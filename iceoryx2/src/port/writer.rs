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
//! type KeyType = u64;
//! let service = node.service_builder(&"My/Funk/ServiceName".try_into()?)
//!     .blackboard_creator::<KeyType>()
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
//! // loan an uninitialized entry value and write to it without copying
//! let entry_value_uninit = writer_handle.loan_uninit();
//! let entry_value = entry_value_uninit.write(-8);
//! let writer_handle = entry_value.update();
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
use core::hash::Hash;
use core::sync::atomic::Ordering;
use iceoryx2_bb_elementary::math::align;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use iceoryx2_bb_lock_free::mpmc::container::ContainerHandle;
use iceoryx2_bb_lock_free::spmc::unrestricted_atomic::{
    Producer, UnrestrictedAtomic, UnrestrictedAtomicMgmt,
};
use iceoryx2_bb_log::fail;
use iceoryx2_cal::dynamic_storage::DynamicStorage;
use iceoryx2_cal::shared_memory::SharedMemory;

extern crate alloc;
use alloc::sync::Arc;

use super::port_identifiers::UniqueWriterId;

#[derive(Debug)]
struct WriterSharedState<
    Service: service::Service,
    KeyType: Send + Sync + Eq + Clone + Debug + 'static + Hash + ZeroCopySend,
> {
    dynamic_writer_handle: Option<ContainerHandle>,
    service_state: Arc<ServiceState<Service, BlackboardResources<Service, KeyType>>>,
}

impl<
        Service: service::Service,
        KeyType: Send + Sync + Eq + Clone + Debug + 'static + Hash + ZeroCopySend,
    > Drop for WriterSharedState<Service, KeyType>
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
pub struct Writer<
    Service: service::Service,
    KeyType: Send + Sync + Eq + Clone + Debug + 'static + Hash + ZeroCopySend,
> {
    shared_state: Arc<WriterSharedState<Service, KeyType>>,
    writer_id: UniqueWriterId,
}

impl<
        Service: service::Service,
        KeyType: Send + Sync + Eq + Clone + Debug + 'static + Hash + ZeroCopySend,
    > Writer<Service, KeyType>
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

        let offset = self.get_entry_offset(
            key,
            &TypeDetail::new::<ValueType>(TypeVariant::FixedSize),
            msg,
        )?;

        match WriterHandle::new(self.shared_state.clone(), offset) {
            Ok(handle) => Ok(handle),
            Err(e) => {
                fail!(from self, with e,
                    "{} since a handle for the passed key and value type already exists.", msg);
            }
        }
    }

    fn get_entry_offset(
        &self,
        key: &KeyType,
        type_details: &TypeDetail,
        msg: &str,
    ) -> Result<u64, WriterHandleError> {
        // check if key exists
        let index = match unsafe {
            self.shared_state
                .service_state
                .additional_resource
                .mgmt
                .get()
                .map
                .get(key)
        } {
            Some(i) => i,
            None => {
                fail!(from self, with WriterHandleError::EntryDoesNotExist,
                "{} since no entry with the given key exists.", msg);
            }
        };

        let entry = &self
            .shared_state
            .service_state
            .additional_resource
            .mgmt
            .get()
            .entries[index];

        // check if ValueType matches
        if *type_details != entry.type_details {
            fail!(from self, with WriterHandleError::EntryDoesNotExist,
                "{} since no entry with the given key and value type exists.", msg);
        }

        let offset = entry.offset.load(core::sync::atomic::Ordering::Relaxed);

        Ok(offset)
    }
}

// TODO: replace u64 with CustomKeyMarker
impl<Service: service::Service> Writer<Service, u64> {
    #[doc(hidden)]
    pub fn __internal_entry(
        &self,
        key: &u64,
        type_details: &TypeDetail,
    ) -> Result<__InternalWriterHandle<Service>, WriterHandleError> {
        let msg = "Unable to create writer handle";
        let offset = self.get_entry_offset(key, type_details, msg)?;

        let atomic_mgmt_ptr = (self
            .shared_state
            .service_state
            .additional_resource
            .data
            .payload_start_address() as u64
            + offset) as *const UnrestrictedAtomicMgmt;

        let data_ptr = atomic_mgmt_ptr as usize + core::mem::size_of::<UnrestrictedAtomicMgmt>();
        let data_ptr = align(data_ptr, type_details.alignment);

        match __InternalWriterHandle::new(
            atomic_mgmt_ptr,
            data_ptr as *mut u8,
            EventId::new(offset as _),
            self.shared_state.clone(),
        ) {
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
        std::write!(f, "WriterHandleError::{self:?}")
    }
}

impl core::error::Error for WriterHandleError {}

/// A handle for direct write access to a specific blackboard value.
pub struct WriterHandle<
    Service: service::Service,
    KeyType: Send + Sync + Eq + Clone + Debug + 'static + Hash + ZeroCopySend,
    ValueType: Copy + 'static,
> {
    producer: Producer<'static, ValueType>,
    entry_id: EventId,
    _shared_state: Arc<WriterSharedState<Service, KeyType>>,
}

// Safe since the producer implements Send + Sync and shared_state ensures the lifetime of the
// producer (struct fields are dropped in the same order as declared)
unsafe impl<
        Service: service::Service,
        KeyType: Send + Sync + Eq + Clone + Debug + 'static + Hash + ZeroCopySend,
        ValueType: Copy + 'static,
    > Send for WriterHandle<Service, KeyType, ValueType>
{
}
unsafe impl<
        Service: service::Service,
        KeyType: Send + Sync + Eq + Clone + Debug + 'static + Hash + ZeroCopySend,
        ValueType: Copy + 'static,
    > Sync for WriterHandle<Service, KeyType, ValueType>
{
}

impl<
        Service: service::Service,
        KeyType: Send + Sync + Eq + Clone + Debug + 'static + Hash + ZeroCopySend,
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
                    entry_id: EventId::new(offset as _),
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
        self.producer.store(value);
    }

    /// Consumes the [`WriterHandle`] and loans an uninitialized entry value that can be used to update without copy.
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
    /// let entry_value_uninit = writer_handle.loan_uninit();
    /// let entry_value = entry_value_uninit.write(-8);
    /// entry_value.update();
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn loan_uninit(self) -> EntryValueUninit<Service, KeyType, ValueType> {
        EntryValueUninit::new(self)
    }

    /// Returns an ID corresponding to the entry which can be used in an event based communication
    /// setup.
    pub fn entry_id(&self) -> EventId {
        self.entry_id
    }
}

// TODO: documentation
#[doc(hidden)]
pub struct __InternalWriterHandle<Service: service::Service> {
    atomic_mgmt_ptr: *const UnrestrictedAtomicMgmt,
    data_ptr: *mut u8,
    entry_id: EventId,
    _shared_state: Arc<WriterSharedState<Service, u64>>,
}

impl<Service: service::Service> Drop for __InternalWriterHandle<Service> {
    fn drop(&mut self) {
        unsafe { (*self.atomic_mgmt_ptr).__internal_release_producer() };
    }
}

impl<Service: service::Service> __InternalWriterHandle<Service> {
    pub fn new(
        atomic_mgmt_ptr: *const UnrestrictedAtomicMgmt,
        data_ptr: *mut u8,
        entry_id: EventId,
        writer_state: Arc<WriterSharedState<Service, u64>>,
    ) -> Result<Self, WriterHandleError> {
        match unsafe { (*atomic_mgmt_ptr).acquire_producer() } {
            Ok(_) => Ok(Self {
                atomic_mgmt_ptr,
                data_ptr,
                entry_id,
                _shared_state: writer_state.clone(),
            }),
            Err(_) => Err(WriterHandleError::HandleAlreadyExists),
        }
    }

    pub fn loan_uninit(
        self,
        value_size: usize,
        value_alignment: usize,
    ) -> __InternalEntryValueUninit<Service> {
        __InternalEntryValueUninit::new(self, value_size, value_alignment)
    }

    pub unsafe fn __internal_get_ptr_to_write_cell(
        &self,
        value_size: usize,
        value_alignment: usize,
    ) -> *mut u8 {
        unsafe {
            (*self.atomic_mgmt_ptr).__internal_get_ptr_to_write_cell(
                value_size,
                value_alignment,
                self.data_ptr,
            )
        }
    }

    pub unsafe fn __internal_update_write_cell(&self) {
        unsafe { (*self.atomic_mgmt_ptr).__internal_update_write_cell() };
    }

    pub fn entry_id(&self) -> EventId {
        self.entry_id
    }
}

/// Wrapper around an uninitiaized entry value that can be used for a zero-copy update.
pub struct EntryValueUninit<
    Service: service::Service,
    KeyType: Send + Sync + Eq + Clone + Debug + 'static + Hash + ZeroCopySend,
    ValueType: Copy + 'static,
> {
    ptr: *mut ValueType,
    writer_handle: WriterHandle<Service, KeyType, ValueType>,
}

// Safe since the WriterHandle implements Send + Sync and the WriterHandle's shared_state ensures that
// the memory address ptr is pointing to remains valid, and all methods of EntryValueUninit are
// consuming.
unsafe impl<
        Service: service::Service,
        KeyType: Send + Sync + Eq + Clone + Debug + 'static + Hash + ZeroCopySend,
        ValueType: Copy + 'static,
    > Send for EntryValueUninit<Service, KeyType, ValueType>
{
}
unsafe impl<
        Service: service::Service,
        KeyType: Send + Sync + Eq + Clone + Debug + 'static + Hash + ZeroCopySend,
        ValueType: Copy + 'static,
    > Sync for EntryValueUninit<Service, KeyType, ValueType>
{
}

impl<
        Service: service::Service,
        KeyType: Send + Sync + Eq + Clone + Debug + 'static + Hash + ZeroCopySend,
        ValueType: Copy + 'static,
    > EntryValueUninit<Service, KeyType, ValueType>
{
    fn new(writer_handle: WriterHandle<Service, KeyType, ValueType>) -> Self {
        let ptr = unsafe { writer_handle.producer.__internal_get_ptr_to_write_cell() };
        Self { ptr, writer_handle }
    }

    /// Consumes the [`EntryValueUninit`], writes value to the entry value and returns the
    /// initialized [`EntryValue`].
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
    /// let entry_value_uninit = writer_handle.loan_uninit();
    /// let entry_value = entry_value_uninit.write(-8);
    /// # entry_value.update();
    /// # Ok(())
    /// # }
    /// ```
    pub fn write(self, value: ValueType) -> EntryValue<Service, KeyType, ValueType> {
        unsafe { self.ptr.write(value) };
        EntryValue::new(self)
    }

    /// Discard the [`EntryValueUninit`] and returns the original [`WriterHandle`].
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
    /// let entry_value_uninit = writer_handle.loan_uninit();
    /// let writer_handle = entry_value_uninit.discard();
    /// # Ok(())
    /// # }
    /// ```
    pub fn discard(self) -> WriterHandle<Service, KeyType, ValueType> {
        self.writer_handle
    }
}

// TODO: test on Rust side
#[doc(hidden)]
pub struct __InternalEntryValueUninit<Service: service::Service> {
    write_cell_ptr: *mut u8,
    writer_handle: __InternalWriterHandle<Service>,
}

impl<Service: service::Service> __InternalEntryValueUninit<Service> {
    pub fn new(
        writer_handle: __InternalWriterHandle<Service>,
        value_size: usize,
        value_alignment: usize,
    ) -> Self {
        let write_cell_ptr = unsafe {
            (*writer_handle.atomic_mgmt_ptr).__internal_get_ptr_to_write_cell(
                value_size,
                value_alignment,
                writer_handle.data_ptr,
            )
        };
        Self {
            write_cell_ptr,
            writer_handle,
        }
    }

    pub fn write_cell(&self) -> *mut u8 {
        self.write_cell_ptr
    }

    pub fn update(self) -> __InternalWriterHandle<Service> {
        unsafe {
            (*self.writer_handle.atomic_mgmt_ptr).__internal_update_write_cell();
        }
        self.writer_handle
    }

    pub fn discard(self) -> __InternalWriterHandle<Service> {
        self.writer_handle
    }

    // cleanup
    // update call
    // __InternalEntryValueUninit can probably be renamed to __InternalEntryValue
}

/// Wrapper around an initialized entry value that can be used for a zero-copy update.
pub struct EntryValue<
    Service: service::Service,
    KeyType: Send + Sync + Eq + Clone + Debug + 'static + Hash + ZeroCopySend,
    ValueType: Copy + 'static,
> {
    writer_handle: WriterHandle<Service, KeyType, ValueType>,
}

// Safe since the WriterHandle implements Send + Sync and all methods of EntryValueUninit are
// consuming.
unsafe impl<
        Service: service::Service,
        KeyType: Send + Sync + Eq + Clone + Debug + 'static + Hash + ZeroCopySend,
        ValueType: Copy + 'static,
    > Send for EntryValue<Service, KeyType, ValueType>
{
}
unsafe impl<
        Service: service::Service,
        KeyType: Send + Sync + Eq + Clone + Debug + 'static + Hash + ZeroCopySend,
        ValueType: Copy + 'static,
    > Sync for EntryValue<Service, KeyType, ValueType>
{
}

impl<
        Service: service::Service,
        KeyType: Send + Sync + Eq + Clone + Debug + 'static + Hash + ZeroCopySend,
        ValueType: Copy + 'static,
    > EntryValue<Service, KeyType, ValueType>
{
    fn new(entry_value_uninit: EntryValueUninit<Service, KeyType, ValueType>) -> Self {
        Self {
            writer_handle: entry_value_uninit.writer_handle,
        }
    }

    /// Makes new value readable for [`Reader`](crate::port::reader::Reader)s, consumes the
    /// [`EntryValue`] and returns the original [`WriterHandle`].
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
    /// let entry_value_uninitialized = writer_handle.loan_uninit();
    /// let entry_value = entry_value_uninitialized.write(-8);
    /// let writer_handle = entry_value.update();
    /// # Ok(())
    /// # }
    /// ```
    pub fn update(self) -> WriterHandle<Service, KeyType, ValueType> {
        unsafe { self.writer_handle.producer.__internal_update_write_cell() };
        self.writer_handle
    }

    /// Discards the [`EntryValue`] and returns the original [`WriterHandle`].
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
    /// let entry_value_uninit = writer_handle.loan_uninit();
    /// let entry_value = entry_value_uninit.write(-8);
    /// let writer_handle = entry_value.discard();
    /// # Ok(())
    /// # }
    /// ```
    pub fn discard(self) -> WriterHandle<Service, KeyType, ValueType> {
        self.writer_handle
    }
}
