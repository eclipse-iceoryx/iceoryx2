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
//! let reader = service.reader_builder().create()?;
//!
//! // create a handle for direct read access to a value
//! let entry_handle = reader.entry::<i32>(&1)?;
//!
//! // get a copy of the value
//! let value = entry_handle.get();
//!
//! # Ok(())
//! # }
//! ```

use crate::prelude::EventId;
use crate::service::builder::blackboard::BlackboardResources;
use crate::service::dynamic_config::blackboard::ReaderDetails;
use crate::service::static_config::message_type_details::{TypeDetail, TypeVariant};
use crate::service::{self, ServiceState};
use core::fmt::Debug;
use core::hash::Hash;
use core::sync::atomic::Ordering;
use iceoryx2_bb_elementary::math::align;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use iceoryx2_bb_lock_free::mpmc::container::ContainerHandle;
use iceoryx2_bb_lock_free::spmc::unrestricted_atomic::{
    UnrestrictedAtomic, UnrestrictedAtomicMgmt,
};
use iceoryx2_bb_log::{fail, fatal_panic};
use iceoryx2_cal::dynamic_storage::DynamicStorage;
use iceoryx2_cal::shared_memory::SharedMemory;

extern crate alloc;
use alloc::sync::Arc;

use super::port_identifiers::UniqueReaderId;

#[derive(Debug)]
struct ReaderSharedState<
    Service: service::Service,
    KeyType: Send + Sync + Eq + Clone + Debug + 'static + Hash + ZeroCopySend,
> {
    dynamic_reader_handle: Option<ContainerHandle>,
    service_state: Arc<ServiceState<Service, BlackboardResources<Service, KeyType>>>,
}

impl<
        Service: service::Service,
        KeyType: Send + Sync + Eq + Clone + Debug + 'static + Hash + ZeroCopySend,
    > Drop for ReaderSharedState<Service, KeyType>
{
    fn drop(&mut self) {
        if let Some(handle) = self.dynamic_reader_handle {
            self.service_state
                .dynamic_storage
                .get()
                .blackboard()
                .release_reader_handle(handle)
        }
    }
}

/// Defines a failure that can occur when a [`Reader`] is created with
/// [`PortFactoryReader`](crate::service::port_factory::reader::PortFactoryReader).
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ReaderCreateError {
    /// The maximum amount of [`Reader`]s that can connect to a
    /// [`Service`](crate::service::Service) is defined in
    /// [`Config`](crate::config::Config). When this is exceeded no more [`Reader`]s
    /// can be created for a specific [`Service`](crate::service::Service).
    ExceedsMaxSupportedReaders,
}

impl core::fmt::Display for ReaderCreateError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        std::write!(f, "ReaderCreateError::{self:?}")
    }
}

impl core::error::Error for ReaderCreateError {}

/// Reading endpoint of a blackboard based communication.
#[derive(Debug)]
pub struct Reader<
    Service: service::Service,
    KeyType: Send + Sync + Eq + Clone + Debug + 'static + Hash + ZeroCopySend,
> {
    shared_state: Arc<ReaderSharedState<Service, KeyType>>,
    reader_id: UniqueReaderId,
}

impl<
        Service: service::Service,
        KeyType: Send + Sync + Eq + Clone + Debug + 'static + Hash + ZeroCopySend,
    > Reader<Service, KeyType>
{
    pub(crate) fn new(
        service: Arc<ServiceState<Service, BlackboardResources<Service, KeyType>>>,
    ) -> Result<Self, ReaderCreateError> {
        let origin = "Reader::new()";
        let msg = "Unable to create Reader port";

        let reader_id = UniqueReaderId::new();
        let mut new_self = Self {
            shared_state: Arc::new(ReaderSharedState {
                dynamic_reader_handle: None,
                service_state: service.clone(),
            }),
            reader_id,
        };

        core::sync::atomic::compiler_fence(Ordering::SeqCst);

        // !MUST! be the last task otherwise a reader is added to the dynamic config without the
        // creation of all required resources
        let dynamic_reader_handle = match service.dynamic_storage.get().blackboard().add_reader_id(
            ReaderDetails {
                reader_id,
                node_id: *service.shared_node.id(),
            },
        ) {
            Some(unique_index) => unique_index,
            None => {
                fail!(from origin, with ReaderCreateError::ExceedsMaxSupportedReaders,
                            "{} since it would exceed the maximum supported amount of readers of {}.",
                            msg, service.static_config.blackboard().max_readers);
            }
        };

        match Arc::get_mut(&mut new_self.shared_state) {
            None => {
                fatal_panic!(from origin,
                    "This should never happen! Member has already multiple references while Reader creation is not yet completed.");
            }
            Some(reader_state) => reader_state.dynamic_reader_handle = Some(dynamic_reader_handle),
        }
        Ok(new_self)
    }

    /// Returns the [`UniqueReaderId`] of the [`Reader`]
    pub fn id(&self) -> UniqueReaderId {
        self.reader_id
    }

    /// Creates a [`EntryHandle`] for direct read access to the value.
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
    /// # let reader = service.reader_builder().create()?;
    /// let entry_handle = reader.entry::<i32>(&1)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn entry<ValueType: Copy + ZeroCopySend>(
        &self,
        key: &KeyType,
    ) -> Result<EntryHandle<Service, KeyType, ValueType>, EntryHandleError> {
        let msg = "Unable to create entry handle";

        let offset = self.get_entry_offset(
            key,
            &TypeDetail::new::<ValueType>(TypeVariant::FixedSize),
            msg,
        )?;

        let atomic = (self
            .shared_state
            .service_state
            .additional_resource
            .data
            .payload_start_address() as u64
            + offset) as *const UnrestrictedAtomic<ValueType>;

        Ok(EntryHandle::new(self.shared_state.clone(), atomic, offset))
    }

    fn get_entry_offset(
        &self,
        key: &KeyType,
        type_details: &TypeDetail,
        msg: &str,
    ) -> Result<u64, EntryHandleError> {
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
                fail!(from self, with EntryHandleError::EntryDoesNotExist,
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
            fail!(from self, with EntryHandleError::EntryDoesNotExist,
                "{} since no entry with the given key and value type exists.", msg);
        }

        let offset = entry.offset.load(core::sync::atomic::Ordering::Relaxed);

        Ok(offset)
    }
}

/// Defines a failure that can occur when a [`EntryHandle`] is created with [`Reader::entry()`].
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum EntryHandleError {
    /// The entry with the given key and value type does not exist.
    EntryDoesNotExist,
}

impl core::fmt::Display for EntryHandleError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        std::write!(f, "EntryHandleError::{self:?}")
    }
}

impl core::error::Error for EntryHandleError {}

/// A handle for direct read access to a specific blackboard value.
pub struct EntryHandle<
    Service: service::Service,
    KeyType: Send + Sync + Eq + Clone + Debug + 'static + Hash + ZeroCopySend,
    ValueType: Copy,
> {
    atomic: *const UnrestrictedAtomic<ValueType>,
    entry_id: EventId,
    _shared_state: Arc<ReaderSharedState<Service, KeyType>>,
}

// Safe since the pointer to the UnrestrictedAtomic doesn't change and the UnrestrictedAtomic
// implements Send + Sync, and shared_state ensures the lifetime of the UnrestrictedAtomic (struct
// fields are dropped in the same order as declared)
unsafe impl<
        Service: service::Service,
        KeyType: Send + Sync + Eq + Clone + Debug + 'static + Hash + ZeroCopySend,
        ValueType: Copy + 'static,
    > Send for EntryHandle<Service, KeyType, ValueType>
{
}
unsafe impl<
        Service: service::Service,
        KeyType: Send + Sync + Eq + Clone + Debug + 'static + Hash + ZeroCopySend,
        ValueType: Copy + 'static,
    > Sync for EntryHandle<Service, KeyType, ValueType>
{
}

impl<
        Service: service::Service,
        KeyType: Send + Sync + Eq + Clone + Debug + 'static + Hash + ZeroCopySend,
        ValueType: Copy,
    > EntryHandle<Service, KeyType, ValueType>
{
    fn new(
        reader_state: Arc<ReaderSharedState<Service, KeyType>>,
        atomic: *const UnrestrictedAtomic<ValueType>,
        offset: u64,
    ) -> Self {
        Self {
            atomic,
            entry_id: EventId::new(offset as _),
            _shared_state: reader_state.clone(),
        }
    }

    /// Returns a copy of the value.
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
    /// # let reader = service.reader_builder().create()?;
    /// # let entry_handle = reader.entry::<i32>(&1)?;
    /// let value = entry_handle.get();
    /// # Ok(())
    /// # }
    /// ```
    pub fn get(&self) -> ValueType {
        unsafe { (*self.atomic).load() }
    }

    /// Returns an ID corresponding to the entry which can be used in an event based communication
    /// setup.
    pub fn entry_id(&self) -> EventId {
        self.entry_id
    }
}

// TODO [#817] replace u64 with CustomKeyMarker
impl<Service: service::Service> Reader<Service, u64> {
    #[doc(hidden)]
    pub fn __internal_entry(
        &self,
        key: &u64,
        type_details: &TypeDetail,
    ) -> Result<__InternalEntryHandle<Service>, EntryHandleError> {
        let msg = "Unable to create entry handle";
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

        Ok(__InternalEntryHandle {
            atomic_mgmt_ptr,
            data_ptr: data_ptr as *const u8,
            entry_id: EventId::new(offset as _),
            _shared_state: self.shared_state.clone(),
        })
    }
}

/// A handle for direct read access to a specific blackboard value. Used for the language bindings
/// where key and value type cannot be passed as generic.
#[doc(hidden)]
pub struct __InternalEntryHandle<Service: service::Service> {
    atomic_mgmt_ptr: *const UnrestrictedAtomicMgmt,
    data_ptr: *const u8,
    entry_id: EventId,
    _shared_state: Arc<ReaderSharedState<Service, u64>>,
}

impl<Service: service::Service> __InternalEntryHandle<Service> {
    /// Stores a copy of the value in `value_ptr`.
    ///
    /// # Safety
    ///
    ///   * see Safety section of core::ptr::copy_nonoverlapping
    pub unsafe fn get(&self, value_ptr: *mut u8, value_size: usize, value_alignment: usize) {
        (&*self.atomic_mgmt_ptr).load(value_ptr, value_size, value_alignment, self.data_ptr);
    }

    /// Returns an ID corresponding to the entry which can be used in an event based communication
    /// setup.
    pub fn entry_id(&self) -> EventId {
        self.entry_id
    }
}
