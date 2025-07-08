// Copyright (c) 2025 Contributors to the Eclipse Foundation
//
// See the NOTICE file(s) distributed with this work for additional
// information regarding copyright ownership.
//
// This program and the accompanying materials are made available under the
// terms of the Apache Software License 2.0 which is available at
// https://www.apache.org/licenses/LICENSE-2.0, or the MIT license
// which is available at https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

use crate::service::builder::blackboard::BlackboardResources;
use crate::service::dynamic_config::blackboard::WriterDetails;
use crate::service::static_config::message_type_details::{TypeDetail, TypeVariant};
use crate::service::{self, ServiceState};
use core::fmt::Debug;
use core::sync::atomic::Ordering;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use iceoryx2_bb_lock_free::mpmc::container::ContainerHandle;
use iceoryx2_bb_lock_free::spmc::unrestricted_atomic::{Producer, UnrestrictedAtomic};
use iceoryx2_bb_log::fail;
use iceoryx2_cal::dynamic_storage::DynamicStorage;
use iceoryx2_cal::shared_memory::SharedMemory;

extern crate alloc;
use alloc::sync::Arc;

use super::port_identifiers::UniqueWriterId;

#[derive(Debug)]
struct WriterSharedState<
    Service: service::Service,
    KeyType: Send + Sync + Eq + Clone + Debug + 'static,
> {
    dynamic_writer_handle: Option<ContainerHandle>,
    service_state: Arc<ServiceState<Service, BlackboardResources<Service, KeyType>>>,
}

impl<Service: service::Service, KeyType: Send + Sync + Eq + Clone + Debug + 'static> Drop
    for WriterSharedState<Service, KeyType>
{
    fn drop(&mut self) {
        if let Some(handle) = self.dynamic_writer_handle {
            self.service_state
                .dynamic_storage
                .get()
                .blackboard()
                .release_writer_handle(handle)
        }
    }
}

/// Defines a failure that can occur when a [`Writer`] is created with
/// [`crate::service::port_factory::writer::PortFactoryWriter`].
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum WriterCreateError {
    /// The maximum amount of [`Writer`]s that can connect to a
    /// [`Service`](crate::service::Service) is
    /// defined in [`crate::config::Config`]. When this is exceeded no more [`Writer`]s
    /// can be created for a specific [`Service`](crate::service::Service).
    ExceedsMaxSupportedWriters,
    /// Errors that indicate either an implementation issue or a wrongly configured system.
    InternalFailure,
}

impl core::fmt::Display for WriterCreateError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        std::write!(f, "WriterCreateError::{:?}", self)
    }
}

impl core::error::Error for WriterCreateError {}

/// Producing endpoint of a blackboard based communication.
#[derive(Debug)]
pub struct Writer<Service: service::Service, KeyType: Send + Sync + Eq + Clone + Debug + 'static> {
    shared_state: Arc<WriterSharedState<Service, KeyType>>,
    writer_id: UniqueWriterId,
}

impl<Service: service::Service, KeyType: Send + Sync + Eq + Clone + Debug + 'static>
    Writer<Service, KeyType>
{
    pub(crate) fn new(
        service: Arc<ServiceState<Service, BlackboardResources<Service, KeyType>>>,
    ) -> Result<Self, WriterCreateError> {
        let origin = "Writer::new()";
        let msg = "Unable to create Writer port";

        let writer_id = UniqueWriterId::new();
        let mut new_self = Self {
            shared_state: Arc::new(WriterSharedState {
                service_state: service.clone(),
                dynamic_writer_handle: None,
            }),
            writer_id,
        };

        core::sync::atomic::compiler_fence(Ordering::SeqCst);

        // !MUST! be the last task otherwise a writer is added to the dynamic config without the
        // creation of all required resources
        let dynamic_writer_handle = match service.dynamic_storage.get().blackboard().add_writer_id(
            WriterDetails {
                writer_id,
                node_id: *service.shared_node.id(),
            },
        ) {
            Some(unique_index) => unique_index,
            None => {
                fail!(from origin, with WriterCreateError::ExceedsMaxSupportedWriters,
                            "{} since it would exceed the maximum supported amount of writers of {}.",
                            msg, service.static_config.blackboard().max_writers);
            }
        };

        match Arc::get_mut(&mut new_self.shared_state) {
            None => {
                fail!(from origin, with WriterCreateError::InternalFailure,
                    "{} due to an internal failure.", msg);
            }
            Some(writer_state) => writer_state.dynamic_writer_handle = Some(dynamic_writer_handle),
        }
        Ok(new_self)
    }

    /// Returns the [`UniqueWriterId`] of the [`Writer`]
    pub fn id(&self) -> UniqueWriterId {
        self.writer_id
    }

    /// Creates a [`WriterHandle`] for direct write access to the value. There can be only one
    /// [`WriterHandle`] per value.
    pub fn entry<ValueType: Copy + ZeroCopySend>(
        &self,
        key: &KeyType,
    ) -> Result<WriterHandle<Service, KeyType, ValueType>, WriterHandleError> {
        let msg = "Unable to create writer handle";

        // check if key exists
        let index = unsafe {
            self.shared_state
                .service_state
                .additional_resource
                .mgmt
                .get()
                .map
                .get(key)
        };
        if index.is_none() {
            fail!(from self, with WriterHandleError::EntryDoesNotExist,
                "{} since no entry with the given key exists.", msg);
        }

        let entry = &self
            .shared_state
            .service_state
            .additional_resource
            .mgmt
            .get()
            .entries[index.unwrap()];

        // check if ValueType matches
        if TypeDetail::__internal_new::<ValueType>(TypeVariant::FixedSize) != entry.type_details {
            fail!(from self, with WriterHandleError::EntryDoesNotExist,
                "{} since no entry with the given key and value type exists.", msg);
        }

        let offset = entry.offset.load(core::sync::atomic::Ordering::Relaxed);
        match WriterHandle::new(self.shared_state.clone(), offset) {
            Ok(handle) => Ok(handle),
            Err(e) => {
                fail!(from self, with e,
                    "{} since a handle for the passed key and value type already exists.", msg);
            }
        }
    }
}

/// Defines a failure that can occur when a [`WriterHandle`] is created with [`Writer::entry()`].
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum WriterHandleError {
    /// The entry with the given key and value type does not exist.
    EntryDoesNotExist,
    /// The [`WriterHandle`] already exists.
    HandleAlreadyExists,
}

impl core::fmt::Display for WriterHandleError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        std::write!(f, "WriterHandleError::{:?}", self)
    }
}

impl core::error::Error for WriterHandleError {}

/// A handle for direct write access to a specific blackboard value.
pub struct WriterHandle<
    Service: service::Service,
    KeyType: Send + Sync + Eq + Clone + Debug + 'static,
    ValueType: Copy + 'static,
> {
    producer: Producer<'static, ValueType>,
    offset: u64,
    _shared_state: Arc<WriterSharedState<Service, KeyType>>,
}

// Safe since the UnrestrictedAtomic the producer belongs to implements Send + Sync, and
// shared_state ensures the lifetime of the UnrestrictedAtomic (struct fields are dropped in the
// same order as declared)
unsafe impl<
        Service: service::Service,
        KeyType: Send + Sync + Eq + Clone + Debug + 'static,
        ValueType: Copy + 'static,
    > Send for WriterHandle<Service, KeyType, ValueType>
{
}

impl<
        Service: service::Service,
        KeyType: Send + Sync + Eq + Clone + Debug + 'static,
        ValueType: Copy + 'static,
    > WriterHandle<Service, KeyType, ValueType>
{
    fn new(
        writer_state: Arc<WriterSharedState<Service, KeyType>>,
        offset: u64,
    ) -> Result<Self, WriterHandleError> {
        let atomic = (writer_state
            .service_state
            .additional_resource
            .data
            .payload_start_address() as u64
            + offset) as *mut UnrestrictedAtomic<ValueType>;
        match unsafe { (*atomic).acquire_producer() } {
            None => Err(WriterHandleError::HandleAlreadyExists),
            Some(producer) => {
                // change to static lifetime is safe since shared_state owns the service state and
                // the dynamic writer handle + the struct fields are dropped in the same order as
                // declared
                let p: Producer<'static, ValueType> = unsafe { core::mem::transmute(producer) };
                Ok(Self {
                    producer: p,
                    _shared_state: writer_state.clone(),
                    offset,
                })
            }
        }
    }

    /// Updates the value by copying the passed value into it.
    pub fn update_with_copy(&self, value: ValueType) {
        self.producer.store(value);
    }
}
