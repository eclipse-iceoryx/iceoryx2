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

//! # Examples
//!
//! ```
//! # use iceoryx2::prelude::*;
//! # fn main() -> Result<(), Box<dyn core::error::Error>> {
//! let node = NodeBuilder::new().create::<ipc::Service>()?;
//! let service = node.service_builder(&"My/Funk/ServiceName".try_into()?)
//!     .blackboard_creator::<u64>()
//!     .add::<i32>(1, -1)
//!     .add::<u32>(9, 17)
//!     .create()?;
//!
//! let writer = service.writer_builder().create()?;
//!
//! // create a handle for direct write access to a value
//! let writer_handle = writer.entry::<i32>(&1)?;
//!
//! // update the value with a copy
//! writer_handle.update_with_copy(8);
//!
//! // loan an uninitialized entry and write to it without copying
//! let entry = writer_handle.loan_uninit()?;
//! entry.write(-8);
//! entry.update();
//!
//! # Ok(())
//! # }
//! ```

use crate::prelude::EventId;
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
use iceoryx2_pal_concurrency_sync::iox_atomic::IoxAtomicBool;

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
        std::write!(f, "WriterCreateError::{self:?}")
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
    ///
    /// # Example
    ///
    /// ```
    /// # use iceoryx2::prelude::*;
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// # let service = node.service_builder(&"My/Funk/ServiceName".try_into()?)
    /// #     .blackboard_creator::<u64>()
    /// #     .add::<i32>(1, -1)
    /// #     .create()?;
    /// #
    /// # let writer = service.writer_builder().create()?;
    /// let writer_handle = writer.entry::<i32>(&1)?;
    /// # Ok(())
    /// # }
    /// ```
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

struct WriterHandleSharedState<ValueType: Copy + 'static> {
    producer: Producer<'static, ValueType>,
    loaned_entry: IoxAtomicBool,
}

/// Defines a failure that can occur when a [`WriterHandle`] is created with [`Writer::entry()`] or
/// an entry is loaned with [`WriterHandle::loan_uninit()`].
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum WriterHandleError {
    /// The entry with the given key and value type does not exist.
    EntryDoesNotExist,
    /// The [`WriterHandle`] already exists.
    HandleAlreadyExists,
    /// The [`WriterHandle`] already loans an entry.
    HandleAlreadyLoansEntry,
}

impl core::fmt::Display for WriterHandleError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        std::write!(f, "WriterHandleError::{self:?}")
    }
}

impl core::error::Error for WriterHandleError {}

/// A handle for direct write access to a specific blackboard value.
pub struct WriterHandle<
    Service: service::Service,
    KeyType: Send + Sync + Eq + Clone + Debug + 'static,
    ValueType: Copy + 'static,
> {
    handle_shared_state: Arc<WriterHandleSharedState<ValueType>>,
    value_id: EventId,
    _shared_state: Arc<WriterSharedState<Service, KeyType>>,
}

impl<
        Service: service::Service,
        KeyType: Send + Sync + Eq + Clone + Debug + 'static,
        ValueType: Copy + 'static,
    > Debug for WriterHandle<Service, KeyType, ValueType>
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "WriterHandle: {{ value_type: {}, has_loaned_entry: {} }}",
            core::any::type_name::<ValueType>(),
            self.handle_shared_state
                .loaned_entry
                .load(Ordering::Relaxed)
        )
    }
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
                    handle_shared_state: Arc::new(WriterHandleSharedState {
                        producer: p,
                        loaned_entry: IoxAtomicBool::new(false),
                    }),
                    _shared_state: writer_state.clone(),
                    value_id: EventId::new(offset as _),
                })
            }
        }
    }

    /// Updates the value by copying the passed value into it.
    ///
    /// # Example
    ///
    /// ```
    /// # use iceoryx2::prelude::*;
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// # let service = node.service_builder(&"My/Funk/ServiceName".try_into()?)
    /// #     .blackboard_creator::<u64>()
    /// #     .add::<i32>(1, -1)
    /// #     .create()?;
    ///
    /// # let writer = service.writer_builder().create()?;
    /// # let writer_handle = writer.entry::<i32>(&1)?;
    /// writer_handle.update_with_copy(8);
    /// # Ok(())
    /// # }
    /// ```
    pub fn update_with_copy(&self, value: ValueType) {
        self.handle_shared_state.producer.store(value);
    }

    /// Loans an entry that can be used to update the value without copy. Only one entry can be
    /// loaned per [`WriterHandle`].
    ///
    /// # Example
    ///
    /// ```
    /// # use iceoryx2::prelude::*;
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// # let service = node.service_builder(&"My/Funk/ServiceName".try_into()?)
    /// #     .blackboard_creator::<u64>()
    /// #     .add::<i32>(1, -1)
    /// #     .create()?;
    ///
    /// # let writer = service.writer_builder().create()?;
    /// # let writer_handle = writer.entry::<i32>(&1)?;
    /// let entry = writer_handle.loan_uninit()?;
    /// entry.write(-8);
    /// entry.update();
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn loan_uninit(&self) -> Result<Entry<ValueType>, WriterHandleError> {
        match Entry::new(self.handle_shared_state.clone()) {
            Ok(ptr) => Ok(ptr),
            Err(_) => {
                fail!(from self, with WriterHandleError::HandleAlreadyLoansEntry,
                    "Entry cannot be loaned since the WriterHandle already loans an entry.");
            }
        }
    }

    /// Returns an ID corresponding to the value which can be used in an event based communication
    /// setup.
    pub fn value_id(&self) -> EventId {
        self.value_id
    }
}

/// Wrapper around a value entry that can be used a for zero-copy uodate.
pub struct Entry<ValueType: Copy + 'static> {
    ptr: *mut ValueType,
    writer_handle_state: Arc<WriterHandleSharedState<ValueType>>,
}

impl<ValueType: Copy + 'static> Drop for Entry<ValueType> {
    fn drop(&mut self) {
        self.writer_handle_state
            .loaned_entry
            .store(false, Ordering::Relaxed);
    }
}

impl<ValueType: Copy + 'static> Entry<ValueType> {
    fn new(
        writer_handle_state: Arc<WriterHandleSharedState<ValueType>>,
    ) -> Result<Self, WriterHandleError> {
        if writer_handle_state
            .loaned_entry
            .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
            .is_err()
        {
            return Err(WriterHandleError::HandleAlreadyLoansEntry);
        }
        let ptr = unsafe { writer_handle_state.producer.get_ptr_to_write_cell() };
        Ok(Self {
            writer_handle_state: writer_handle_state.clone(),
            ptr,
        })
    }

    /// Writes value to the entry.
    ///
    /// # Example
    ///
    /// ```
    /// # use iceoryx2::prelude::*;
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// # let service = node.service_builder(&"My/Funk/ServiceName".try_into()?)
    /// #     .blackboard_creator::<u64>()
    /// #     .add::<i32>(1, -1)
    /// #     .create()?;
    ///
    /// # let writer = service.writer_builder().create()?;
    /// # let writer_handle = writer.entry::<i32>(&1)?;
    /// let entry = writer_handle.loan_uninit()?;
    /// entry.write(-8);
    /// # Ok(())
    /// # }
    /// ```
    pub fn write(&self, value: ValueType) {
        unsafe { self.ptr.write(value) };
    }

    /// Makes new value readable for [`Reader`](crate::port::reader::Reader)s and consumes the
    /// entry, i.e. it cannot be used anymore.
    ///
    /// # Example
    ///
    /// ```
    /// # use iceoryx2::prelude::*;
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// # let service = node.service_builder(&"My/Funk/ServiceName".try_into()?)
    /// #     .blackboard_creator::<u64>()
    /// #     .add::<i32>(1, -1)
    /// #     .create()?;
    ///
    /// # let writer = service.writer_builder().create()?;
    /// # let writer_handle = writer.entry::<i32>(&1)?;
    /// let entry = writer_handle.loan_uninit()?;
    /// entry.write(-8);
    /// entry.update();
    /// # Ok(())
    /// # }
    /// ```
    pub fn update(self) {
        unsafe { self.writer_handle_state.producer.update_write_cell() };
        self.writer_handle_state
            .loaned_entry
            .store(false, Ordering::Relaxed);
    }
}
