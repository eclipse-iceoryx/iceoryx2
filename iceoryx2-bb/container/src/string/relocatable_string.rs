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

use crate::string::internal::StringView;
use core::fmt::{Debug, Display};
use core::mem::MaybeUninit;
use iceoryx2_bb_elementary::math::unaligned_mem_size;
use iceoryx2_bb_elementary::relocatable_ptr::RelocatablePointer;
use iceoryx2_bb_elementary_traits::pointer_trait::PointerTrait;
use iceoryx2_bb_elementary_traits::relocatable_container::RelocatableContainer;
use iceoryx2_bb_log::{fail, fatal_panic};
use std::alloc::Layout;
use std::cmp::Ordering;
use std::hash::Hash;
use std::ops::{Deref, DerefMut};

use crate::string::{as_escaped_string, internal, String};

/// **Non-movable** relocatable shared-memory compatible string with runtime fixed size capacity.
#[repr(C)]
pub struct RelocatableString {
    data_ptr: RelocatablePointer<MaybeUninit<u8>>,
    capacity: u64,
    len: u64,
}

impl internal::StringView for RelocatableString {
    fn data(&self) -> &[MaybeUninit<u8>] {
        self.verify_init("data()");
        unsafe { core::slice::from_raw_parts(self.data_ptr.as_ptr(), self.capacity() + 1) }
    }

    unsafe fn data_mut(&mut self) -> &mut [MaybeUninit<u8>] {
        self.verify_init("data_mut()");
        core::slice::from_raw_parts_mut(self.data_ptr.as_mut_ptr(), self.capacity())
    }

    unsafe fn set_len(&mut self, len: u64) {
        self.len = len;
    }
}

impl Debug for RelocatableString {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "RelocatableString {{ capacity: {}, len: {}, data: \"{}\" }}",
            self.capacity,
            self.len,
            as_escaped_string(self.as_bytes())
        )
    }
}

unsafe impl Send for RelocatableString {}

impl PartialOrd<RelocatableString> for RelocatableString {
    fn partial_cmp(&self, other: &RelocatableString) -> Option<Ordering> {
        self.data()[..self.len as usize]
            .iter()
            .zip(other.data()[..other.len as usize].iter())
            .map(|(lhs, rhs)| unsafe { lhs.assume_init_read().cmp(rhs.assume_init_ref()) })
            .find(|&ord| ord != Ordering::Equal)
            .or(Some(self.len.cmp(&other.len)))
    }
}

impl Ord for RelocatableString {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl Hash for RelocatableString {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        state.write(self.as_bytes())
    }
}

impl Deref for RelocatableString {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.as_bytes()
    }
}

impl DerefMut for RelocatableString {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_bytes()
    }
}

impl PartialEq<RelocatableString> for RelocatableString {
    fn eq(&self, other: &RelocatableString) -> bool {
        *self.as_bytes() == *other.as_bytes()
    }
}

impl Eq for RelocatableString {}

impl PartialEq<&[u8]> for RelocatableString {
    fn eq(&self, other: &&[u8]) -> bool {
        *self.as_bytes() == **other
    }
}

impl PartialEq<&str> for RelocatableString {
    fn eq(&self, other: &&str) -> bool {
        *self.as_bytes() == *other.as_bytes()
    }
}

impl PartialEq<RelocatableString> for &str {
    fn eq(&self, other: &RelocatableString) -> bool {
        *self.as_bytes() == *other.as_bytes()
    }
}

impl<const OTHER_CAPACITY: usize> PartialEq<[u8; OTHER_CAPACITY]> for RelocatableString {
    fn eq(&self, other: &[u8; OTHER_CAPACITY]) -> bool {
        *self.as_bytes() == *other
    }
}

impl<const OTHER_CAPACITY: usize> PartialEq<&[u8; OTHER_CAPACITY]> for RelocatableString {
    fn eq(&self, other: &&[u8; OTHER_CAPACITY]) -> bool {
        *self.as_bytes() == **other
    }
}

impl Display for RelocatableString {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", as_escaped_string(self.as_bytes()))
    }
}

impl RelocatableString {
    #[inline(always)]
    fn verify_init(&self, source: &str) {
        debug_assert!(
                self.data_ptr.is_initialized(),
                "From: RelocatableString::{}, Undefined behavior - the object was not initialized with 'init' before.",
                source
            );
    }

    /// Returns the required memory size for a vec with a specified capacity
    pub const fn const_memory_size(capacity: usize) -> usize {
        unaligned_mem_size::<u8>(capacity + 1)
    }
}

impl RelocatableContainer for RelocatableString {
    unsafe fn new_uninit(capacity: usize) -> Self {
        Self {
            data_ptr: RelocatablePointer::new_uninit(),
            capacity: capacity as u64,
            len: 0,
        }
    }

    unsafe fn init<Allocator: iceoryx2_bb_elementary_traits::allocator::BaseAllocator>(
        &mut self,
        allocator: &Allocator,
    ) -> Result<(), iceoryx2_bb_elementary_traits::allocator::AllocationError> {
        let origin = "RelocatableString::init()";
        if self.data_ptr.is_initialized() {
            fatal_panic!(from origin,
                "Memory already initialized, Initializing it twice may lead to undefined behavior.");
        }

        let ptr = match allocator.allocate(Layout::from_size_align_unchecked(
            core::mem::size_of::<u8>() * self.capacity as usize,
            core::mem::align_of::<u8>(),
        )) {
            Ok(ptr) => ptr,
            Err(e) => {
                fail!(from origin, with e,
                    "Failed to initialize since the allocation of the data memory failed.");
            }
        };

        self.data_ptr.init(ptr);

        Ok(())
    }

    fn memory_size(capacity: usize) -> usize {
        Self::const_memory_size(capacity)
    }
}

impl String for RelocatableString {
    fn capacity(&self) -> usize {
        self.capacity as usize
    }

    fn len(&self) -> usize {
        self.len as usize
    }
}
