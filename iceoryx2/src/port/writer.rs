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

use crate::service::builder::blackboard::{BlackboardResources, Mgmt};
use crate::service::config_scheme::{blackboard_data_config, blackboard_mgmt_config};
use crate::service::dynamic_config::blackboard::WriterDetails;
use crate::service::static_config::message_type_details::{TypeDetail, TypeVariant};
use crate::service::{self, ServiceState};
use core::fmt::Debug;
use core::sync::atomic::Ordering;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use iceoryx2_bb_lock_free::mpmc::container::ContainerHandle;
use iceoryx2_bb_lock_free::spmc::unrestricted_atomic::{Producer, UnrestrictedAtomic};
use iceoryx2_bb_log::fail;
use iceoryx2_cal::dynamic_storage::{DynamicStorage, DynamicStorageBuilder};
use iceoryx2_cal::event::{NamedConcept, NamedConceptBuilder};
use iceoryx2_cal::shared_memory::{SharedMemory, SharedMemoryBuilder};
use iceoryx2_cal::shm_allocator::bump_allocator::BumpAllocator;

extern crate alloc;
use alloc::sync::Arc;

use super::port_identifiers::UniqueWriterId;

/// Defines a failure that can occur when a [`Writer`] is created with
/// [`crate::service::port_factory::writer::PortFactoryWriter`].
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum WriterCreateError {
    /// The maximum amount of [`Writer`]s that can connect to a
    /// [`Service`](crate::service::Service) is
    /// defined in [`crate::config::Config`]. When this is exceeded no more [`Writer`]s
    /// can be created for a specific [`Service`](crate::service::Service).
    ExceedsMaxSupportedWriters,
    /// The data segment could not be opened.
    UnableToOpenDataSegment,
}

impl core::fmt::Display for WriterCreateError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        std::write!(f, "WriterCreateError::{:?}", self)
    }
}

impl core::error::Error for WriterCreateError {}

/// Producing endpoint of a blackboard based communication.
#[derive(Debug)]
pub struct Writer<
    Service: service::Service,
    T: Send + Sync + Debug + 'static + Eq + ZeroCopySend + Clone,
> {
    service_state: Arc<ServiceState<Service, BlackboardResources<Service, T>>>,
    mgmt: Service::BlackboardMgmt<Mgmt<T>>,
    payload: Service::BlackboardPayload,
    dynamic_writer_handle: Option<ContainerHandle>,
}

impl<Service: service::Service, T: Send + Sync + Debug + 'static + Eq + ZeroCopySend + Clone> Drop
    for Writer<Service, T>
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

impl<Service: service::Service, T: Send + Sync + Debug + 'static + Eq + ZeroCopySend + Clone>
    Writer<Service, T>
{
    pub(crate) fn new(
        service: Arc<ServiceState<Service, BlackboardResources<Service, T>>>,
    ) -> Result<Self, WriterCreateError> {
        let origin = "Writer::new()";
        let msg = "Unable to create Writer port";

        // open payload data segment
        let name = service.additional_resource.mgmt.name();
        let shm_config = blackboard_data_config::<Service, Mgmt<T>>(service.shared_node.config());
        let payload_shm = fail!(from origin,
            when <<Service::BlackboardPayload as SharedMemory<BumpAllocator>
                >::Builder as NamedConceptBuilder<Service::BlackboardPayload>>::new(&name)
                .config(&shm_config)
                .has_ownership(false)
                .open(),
            with WriterCreateError::UnableToOpenDataSegment,
            "{} since the payload data segment could not be opened.", msg);

        // open management segment
        let mgmt_config = blackboard_mgmt_config::<Service, Mgmt<T>>(service.shared_node.config());
        let mgmt_storage = fail!(from origin,
            when <Service::BlackboardMgmt<Mgmt<T>> as DynamicStorage<Mgmt<T>>>::Builder::new(&name)
                .config(&mgmt_config)
                .has_ownership(false)
                .open(),
            with WriterCreateError::UnableToOpenDataSegment,
            "{} since the management data segment could not be opened.", msg);

        let mut new_self = Self {
            service_state: service.clone(),
            mgmt: mgmt_storage,
            payload: payload_shm,
            dynamic_writer_handle: None,
        };

        core::sync::atomic::compiler_fence(Ordering::SeqCst);

        // !MUST! be the last task otherwise a writer is added to the dynamic config without the
        // creation of all required resources
        let dynamic_writer_handle = match service.dynamic_storage.get().blackboard().add_writer_id(
            WriterDetails {
                writer_id: UniqueWriterId::new(),
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

        new_self.dynamic_writer_handle = Some(dynamic_writer_handle);

        Ok(new_self)
    }

    /// Creates a [`WriterHandle`] for direct write access to the value. There can be only one
    /// [`WriterHandle`] per value.
    pub fn entry<ValueType: Copy + ZeroCopySend>(
        &self,
        key: &T,
    ) -> Result<WriterHandle<ValueType>, WriterHandleError> {
        let msg = "Unable to create writer handle";

        // check if key exists
        let index = unsafe { self.mgmt.get().map.get(key) };
        if index.is_none() {
            fail!(from self, with WriterHandleError::EntryDoesNotExist,
                "{} since no entry with the given key exists.", msg);
        }

        let entry = &self.mgmt.get().entries[index.unwrap()];

        // check if ValueType matches
        if TypeDetail::__internal_new::<ValueType>(TypeVariant::FixedSize) != entry.type_details {
            fail!(from self, with WriterHandleError::EntryDoesNotExist,
                "{} since no entry with the given key and value type exists.", msg);
        }

        let offset = entry.offset.load(core::sync::atomic::Ordering::Relaxed);
        let atomic = (self.payload.payload_start_address() as u64 + offset)
            as *mut UnrestrictedAtomic<ValueType>;
        match unsafe { (*atomic).acquire_producer() } {
            None => {
                fail!(from self, with WriterHandleError::HandleAlreadyExists,
                    "{} since a handle for the passed key and value type already exists.", msg);
            }
            Some(producer) => Ok(WriterHandle::new(producer)),
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
pub struct WriterHandle<'handle, ValueType: Copy> {
    producer: Producer<'handle, ValueType>,
}

impl<'handle, ValueType: Copy> WriterHandle<'handle, ValueType> {
    fn new(producer: Producer<'handle, ValueType>) -> Self {
        Self { producer }
    }

    /// Updates the value by copying the passed value into it.
    pub fn update_with_copy(&self, value: ValueType) {
        self.producer.store(value);
    }
}

// TODO:
// 1) allow slow write without handle?
// 2) loan API?
