// Copyright (c) 2026 Contributors to the Eclipse Foundation
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

extern crate alloc;

use crate::constants::{MAX_BLACKBOARD_KEY_ALIGNMENT, MAX_BLACKBOARD_KEY_SIZE};
use crate::service::builder::{self, ServiceCreateError};
use crate::service::config_scheme::{blackboard_data_config, blackboard_mgmt_config};
use crate::service::naming_scheme::blackboard_name;
use crate::service::static_config::StaticConfig;
use crate::service::{
    self, resource::ServiceResource, static_config::message_type_details::TypeDetail,
};
use alloc::sync::Arc;
use core::alloc::Layout;
use core::mem::MaybeUninit;
use core::{fmt::Debug, ptr::NonNull};
use iceoryx2_bb_concurrency::atomic::AtomicU64;
use iceoryx2_bb_container::queue::RelocatableContainer;
use iceoryx2_bb_container::string::String;
use iceoryx2_bb_container::vector::Vector;
use iceoryx2_bb_container::{flatmap::RelocatableFlatMap, vector::RelocatableVec};
use iceoryx2_bb_derive_macros::ZeroCopySend;
use iceoryx2_bb_elementary::static_assert::static_assert_eq;
use iceoryx2_bb_elementary_traits::{
    non_null::NonNullCompat, testing::abandonable::Abandonable, zero_copy_send::ZeroCopySend,
};
use iceoryx2_bb_memory::bump_allocator::BumpAllocator;
use iceoryx2_bb_posix::file::AccessMode;
use iceoryx2_cal::dynamic_storage::{DynamicStorage, DynamicStorageBuilder};
use iceoryx2_cal::named_concept::NamedConceptBuilder;
use iceoryx2_cal::shared_memory::SharedMemory;
use iceoryx2_cal::shared_memory::SharedMemoryBuilder;
use iceoryx2_log::error;
use iceoryx2_log::fail;

#[doc(hidden)]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum KeyMemoryError {
    ValueTooLarge,
    ValueAlignmentTooLarge,
}

#[doc(hidden)]
#[repr(C)]
#[repr(align(8))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, ZeroCopySend)]
pub struct KeyMemory<const CAPACITY: usize> {
    pub data: [u8; CAPACITY],
}

impl<const CAPACITY: usize> KeyMemory<CAPACITY> {
    pub fn try_from<T: Copy>(value: &T) -> Result<Self, KeyMemoryError> {
        static_assert_eq::<{ align_of::<KeyMemory<1>>() }, MAX_BLACKBOARD_KEY_ALIGNMENT>();

        let origin = "KeyMemory::try_from()";
        let msg = "Unable to create KeyMemory";

        // Replace if block with below compile-time assertion once available for generic parameters
        // static_assert_le::<{ size_of::<T>() }, CAPACITY>();
        if size_of::<T>() > CAPACITY {
            fail!(from origin, with KeyMemoryError::ValueTooLarge,
                "{} since the passed value is too large. Its size must be <= {}.", msg, CAPACITY);
        }
        // Replace if block with below compile-time assertion once available for generic parameters
        // static_assert_le::<{ align_of::<T>() }, MAX_BLACKBOARD_KEY_ALIGNMENT>();
        if align_of::<T>() > MAX_BLACKBOARD_KEY_ALIGNMENT {
            fail!(from origin, with KeyMemoryError::ValueAlignmentTooLarge,
                "{} since the alignment of the passed value is too large. The alignment must be <= {}.",
                msg, MAX_BLACKBOARD_KEY_ALIGNMENT);
        }

        let mut new_self = Self {
            data: [0; CAPACITY],
        };
        unsafe { core::ptr::copy_nonoverlapping(value, new_self.data.as_mut_ptr() as *mut T, 1) };
        Ok(new_self)
    }

    /// # Safety
    ///
    ///   * see Safety section of core::ptr::copy_nonoverlapping
    pub unsafe fn try_from_ptr(ptr: *const u8, key_layout: Layout) -> Result<Self, KeyMemoryError> {
        static_assert_eq::<{ align_of::<KeyMemory<1>>() }, MAX_BLACKBOARD_KEY_ALIGNMENT>();

        let origin = "KeyMemory::try_from_ptr()";
        let msg = "Unable to create KeyMemory";

        if key_layout.size() > CAPACITY {
            fail!(from origin, with KeyMemoryError::ValueTooLarge,
                "{} since the passed key size is too large. The size must be <= {}.", msg, CAPACITY);
        }
        if key_layout.align() > MAX_BLACKBOARD_KEY_ALIGNMENT {
            fail!(from origin, with KeyMemoryError::ValueAlignmentTooLarge,
                "{} since the alignment of the passed key is too large. The alignment must be <= {}.",
                msg, MAX_BLACKBOARD_KEY_ALIGNMENT);
        }

        let mut new_self = Self {
            data: [0; CAPACITY],
        };
        unsafe {
            core::ptr::copy_nonoverlapping(ptr, new_self.data.as_mut_ptr(), key_layout.size())
        };
        Ok(new_self)
    }

    /// This function compares two KeyMemory<CAPACITY> for equality and is only for blackboard internal
    /// usage. It is passed to functions that require a Fn(*const u8, *const u8) -> bool so
    /// default_key_eq_comparison cannot be marked unsafe. Still, there are safety requirements which are
    /// guaranteed by the by the blackboard implementation:
    ///
    /// # Safety
    ///
    ///   * lhs and rhs must be valid pointers to valid KeyMemory<CAPACITY>
    pub fn default_key_eq_comparison<T: Eq>(lhs: *const u8, rhs: *const u8) -> bool {
        let lhs = unsafe { *(lhs as *const KeyMemory<CAPACITY>) };
        let rhs = unsafe { *(rhs as *const KeyMemory<CAPACITY>) };
        unsafe { *(lhs.data.as_ptr() as *const T) == *(rhs.data.as_ptr() as *const T) }
    }

    /// This function compares two KeyMemory<CAPACITY> for equality using the given compare function.
    /// It is passed to functions that require a Fn(*const u8, *const u8) -> bool so key_eq_comparison
    /// cannot be unsafe. Still, there are safety requirements:
    ///
    /// # Safety
    ///
    ///   * lhs and rhs must be valid pointers to valid KeyMemory<CAPACITY>
    pub fn key_eq_comparison<F: Fn(*const u8, *const u8) -> bool>(
        lhs: *const u8,
        rhs: *const u8,
        eq_func: &F,
    ) -> bool {
        let lhs = unsafe { *(lhs as *const KeyMemory<CAPACITY>) };
        let rhs = unsafe { *(rhs as *const KeyMemory<CAPACITY>) };
        eq_func(lhs.data.as_ptr(), rhs.data.as_ptr())
    }
}

#[repr(C)]
#[derive(Debug, ZeroCopySend)]
pub(crate) struct Entry {
    pub(crate) type_details: TypeDetail,
    pub(crate) offset: AtomicU64,
}

#[repr(C)]
#[derive(Debug, ZeroCopySend)]
pub(crate) struct Mgmt {
    pub(crate) map: RelocatableFlatMap<KeyMemory<MAX_BLACKBOARD_KEY_SIZE>, usize>,
    pub(crate) entries: RelocatableVec<Entry>,
}

pub(crate) struct BlackboardResources<ServiceType: service::Service> {
    pub(crate) mgmt: ServiceType::BlackboardMgmt<Mgmt>,
    pub(crate) data: ServiceType::BlackboardPayload,
    pub(crate) key_eq_func: Arc<dyn Fn(*const u8, *const u8) -> bool + Send + Sync>,
}

impl<ServiceType: service::Service> Debug for BlackboardResources<ServiceType> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "BlackboardResource<{}> {{ mgmt: {:?}, data: {:?} }}",
            core::any::type_name::<ServiceType>(),
            self.mgmt,
            self.data,
        )
    }
}

impl<ServiceType: service::Service> Abandonable for BlackboardResources<ServiceType> {
    unsafe fn abandon_in_place(mut this: NonNull<Self>) {
        let this = unsafe { this.as_mut() };
        unsafe {
            ServiceType::BlackboardMgmt::<Mgmt>::abandon_in_place(NonNull::iox2_from_mut(
                &mut this.mgmt,
            ))
        };
        unsafe {
            ServiceType::BlackboardPayload::abandon_in_place(NonNull::iox2_from_mut(&mut this.data))
        };
    }
}

impl<ServiceType: service::Service> ServiceResource for BlackboardResources<ServiceType> {
    type Config = builder::blackboard::BuilderConfig<ServiceType>;

    fn acquire_ownership(&self) {
        self.data.acquire_ownership();
        self.mgmt.acquire_ownership();
    }

    fn open(
        static_config: &StaticConfig,
        resource_config: &Self::Config,
    ) -> Result<Self, builder::ServiceOpenError> {
        let origin = format!(
            "BlackboardResource<{}>::open()",
            core::any::type_name::<ServiceType>()
        );
        let msg = "Failed to open blackboard service resources";
        let shared_node = &resource_config.base.shared_node;
        let blackboard_config = resource_config.config_details();
        let key_eq_func = resource_config.key_eq_func.clone();
        let name = blackboard_name(static_config.unique_service_id());
        let mut mgmt_config = blackboard_mgmt_config::<ServiceType, Mgmt>(shared_node.config());
        let mgmt_name = blackboard_config.type_details.type_name.as_str();
        // The name was set in create_impl to be able to remove the concept when a node
        // dies. Safe since the same name is set in
        // ServiceInternal::__internal_remove_node_from_service.
        unsafe {
            <ServiceType::BlackboardMgmt<Mgmt> as DynamicStorage<
                Mgmt,
            >>::__internal_set_type_name_in_config(
                &mut mgmt_config, mgmt_name
            )
        };
        let mgmt_storage = fail!(from origin, when
            <ServiceType::BlackboardMgmt<Mgmt> as DynamicStorage<Mgmt>
            >::Builder::new(&name)
                .config(&mgmt_config)
                .has_ownership(false)
                .open(AccessMode::ReadWrite),
            with builder::ServiceOpenError::ServiceInCorruptedState,
            "{} since the blackboard management information could not be opened. This could indicate a corrupted system.", msg);

        let shm_config = blackboard_data_config::<ServiceType>(shared_node.config());
        let payload_shm = match <<ServiceType::BlackboardPayload as SharedMemory<
            iceoryx2_cal::shm_allocator::shm_bump_allocator::BumpAllocator,
        >>::Builder as NamedConceptBuilder<ServiceType::BlackboardPayload>>::new(
            &name
        )
        .config(&shm_config)
        .open(AccessMode::ReadWrite)
        {
            Ok(v) => v,
            Err(_) => {
                fail!(from origin, with builder::ServiceOpenError::ServiceInCorruptedState,
                    "{} since the blackboard payload data segment could not be opened. This could indicate a corrupted system.",
                    msg);
            }
        };

        Ok(BlackboardResources {
            mgmt: mgmt_storage,
            data: payload_shm,
            key_eq_func,
        })
    }

    fn create(
        service_config: &StaticConfig,
        resource_config: &Self::Config,
    ) -> Result<BlackboardResources<ServiceType>, ServiceCreateError> {
        let origin = format!(
            "BlackboardResource<{}>::create()",
            core::any::type_name::<ServiceType>()
        );
        let msg = "Failed to create blackboard service resources";
        let blackboard_config = *resource_config.config_details();
        let key_eq_func = resource_config.key_eq_func.clone();
        let builder_internals = resource_config.internals.as_slice();
        let shared_node = &resource_config.base.shared_node;
        // create the payload data segment for the writer
        let name = blackboard_name(service_config.unique_service_id());
        let shm_config = blackboard_data_config::<ServiceType>(shared_node.config());
        let mut payload_size = 0;
        for i in builder_internals.iter() {
            payload_size += i.internal_value_size + i.internal_value_alignment - 1;
        }
        let payload_shm = match <<ServiceType::BlackboardPayload as SharedMemory<
            iceoryx2_cal::shm_allocator::shm_bump_allocator::BumpAllocator,
        >>::Builder as NamedConceptBuilder<ServiceType::BlackboardPayload>>::new(
            &name
        )
        .config(&shm_config)
        .has_ownership(true)
        .size(payload_size)
        .create(&iceoryx2_cal::shared_memory::shm_bump_allocator::Config::default())
        {
            Ok(v) => v,
            Err(_) => {
                fail!(from origin, with ServiceCreateError::ServiceInCorruptedState,
                    "{} since the blackboard payload data segment could not be created. This could indicate a corrupted system.",
                    msg);
            }
        };

        // create the management segment
        let capacity = builder_internals.len();

        let mut mgmt_config = blackboard_mgmt_config::<ServiceType, Mgmt>(shared_node.config());
        let mgmt_name = blackboard_config.type_details.type_name.as_str();

        // The name is set to be able to remove the concept when a node dies. Safe since the
        // same name is set in ServiceInternal::__internal_remove_node_from_service.
        unsafe {
            <ServiceType::BlackboardMgmt<Mgmt> as DynamicStorage::<
                Mgmt,
            >>::__internal_set_type_name_in_config(&mut mgmt_config, mgmt_name)
        };

        let mgmt_storage = fail!(from origin, when
            <ServiceType::BlackboardMgmt<Mgmt> as DynamicStorage<Mgmt,
            >>::Builder::new(&name)
                .config(&mgmt_config)
                .has_ownership(true)
                .supplementary_size(RelocatableFlatMap::<KeyMemory<MAX_BLACKBOARD_KEY_SIZE>, usize>::const_memory_size(capacity)+RelocatableVec::<Entry>::const_memory_size(capacity))
                .initializer(|mgmt: &mut MaybeUninit<Mgmt>, allocator: &mut BumpAllocator| {
                    mgmt.write(Mgmt {
                        map: unsafe { RelocatableFlatMap::<KeyMemory<MAX_BLACKBOARD_KEY_SIZE>, usize>::new_uninit(capacity) },
                        entries: unsafe { RelocatableVec::<Entry>::new_uninit(capacity) },
                    });
                    let mgmt = unsafe { mgmt.assume_init_mut() };

                    if unsafe {mgmt.map.init(allocator)}.is_err() || unsafe {mgmt.entries.init(allocator).is_err()} {
                        return false
                    }
                    for entry in builder_internals.iter() {
                        // write value passed to add() to payload_shm
                        let mem = match payload_shm.allocate(unsafe { Layout::from_size_align_unchecked(entry.internal_value_size, entry.internal_value_alignment) })
                        {
                            Ok(m) => m,
                            Err(_) => {
                                error!(from origin, "Writing the value to the blackboard data segment failed.");
                                return false
                            }
                        };
                        (*entry.value_writer)(mem.data_ptr);
                        // write offset to value in payload_shm to entries vector
                        let res = mgmt.entries.push(Entry{type_details: entry.value_type_details, offset: AtomicU64::new(mem.offset.offset() as u64)});
                        if res.is_err() {
                            error!(from origin, "Writing the value offset to the blackboard management segment failed.");
                            return false
                        }
                        // write offset index to map
                        let res = unsafe {mgmt.map.__internal_insert(entry.key, mgmt.entries.len() - 1, &*key_eq_func)};
                        if res.is_err() {
                            error!(from origin, "Inserting the key-value pair into the blackboard management segment failed.");
                            return false
                        }
                    }
                    true})
                .create(),
                    with ServiceCreateError::ServiceInCorruptedState, "{} since the blackboard management segment could not be created. This could indicate a corrupted system.",
                    msg);

        Ok(BlackboardResources {
            mgmt: mgmt_storage,
            data: payload_shm,
            key_eq_func,
        })
    }
}
