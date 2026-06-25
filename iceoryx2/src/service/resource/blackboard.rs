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

use std::alloc::Layout;
use std::{fmt::Debug, ptr::NonNull, sync::Arc};

use crate::constants::{MAX_BLACKBOARD_KEY_ALIGNMENT, MAX_BLACKBOARD_KEY_SIZE};
use crate::service::{
    self, resource::ServiceResource, static_config::message_type_details::TypeDetail,
};
use iceoryx2_bb_concurrency::atomic::AtomicU64;
use iceoryx2_bb_container::{flatmap::RelocatableFlatMap, vector::RelocatableVec};
use iceoryx2_bb_derive_macros::ZeroCopySend;
use iceoryx2_bb_elementary::static_assert::static_assert_eq;
use iceoryx2_bb_elementary_traits::{
    non_null::NonNullCompat, testing::abandonable::Abandonable, zero_copy_send::ZeroCopySend,
};
use iceoryx2_cal::dynamic_storage::DynamicStorage;
use iceoryx2_cal::shared_memory::SharedMemory;
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

impl<ServiceType: service::Service> Debug for BlackboardResources<ServiceType> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "BlackboardResources {{ mgmt: {:?}, data: {:?} }}",
            self.mgmt, self.data
        )
    }
}

impl<ServiceType: service::Service> ServiceResource for BlackboardResources<ServiceType> {
    fn acquire_ownership(&self) {
        self.data.acquire_ownership();
        self.mgmt.acquire_ownership();
    }
}
