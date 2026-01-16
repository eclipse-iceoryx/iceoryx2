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

//! String implementation with a polymorphic stateful allocator.
//!
//! # Example
//!
//! ```no_run
//! # extern crate iceoryx2_bb_loggers;
//!
//! use iceoryx2_bb_elementary_traits::allocator::*;
//! use iceoryx2_bb_container::string::*;
//!
//! # struct SomeAllocator {}
//!
//! # impl SomeAllocator {
//! #     fn new() -> Self {
//! #          Self {}
//! #     }
//! # }
//! #
//! # impl BaseAllocator for SomeAllocator {
//! #     fn allocate(
//! #         &self,
//! #         layout: core::alloc::Layout,
//! #     ) -> Result<core::ptr::NonNull<[u8]>, AllocationError> {
//! #         todo!()
//! #     }
//! #
//! #     unsafe fn deallocate(&self, _ptr: core::ptr::NonNull<u8>, _layout: core::alloc::Layout) {
//! #         todo!()
//! #     }
//! # }
//!
//! # fn main() -> Result<(), Box<dyn core::error::Error>> {
//! let allocator = SomeAllocator::new();
//! let capacity: usize = 123;
//! let mut my_str =
//!     PolymorphicString::<SomeAllocator>::new(&allocator, capacity)?;
//!
//! my_str.push_bytes(b"all glory to the hypnotoad"); // returns false, when capacity is exceeded
//! # Ok(())
//! # }
//! ```

use alloc::format;
use core::{
    alloc::Layout,
    cmp::Ordering,
    fmt::{Debug, Display},
    hash::Hash,
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
};

use iceoryx2_bb_elementary_traits::allocator::{AllocationError, BaseAllocator};

use crate::string::*;

/// Runtime fixed-size string variant with a polymorphic allocator, meaning an
/// allocator with a state can be attached to the string instead of using a
/// stateless allocator like the heap-allocator.
pub struct PolymorphicString<'a, Allocator: BaseAllocator> {
    data_ptr: *mut MaybeUninit<u8>,
    len: u64,
    capacity: u64,
    allocator: &'a Allocator,
}

impl<Allocator: BaseAllocator> internal::StringView for PolymorphicString<'_, Allocator> {
    fn data(&self) -> &[MaybeUninit<u8>] {
        unsafe { core::slice::from_raw_parts(self.data_ptr, self.capacity() + 1) }
    }

    unsafe fn data_mut(&mut self) -> &mut [MaybeUninit<u8>] {
        core::slice::from_raw_parts_mut(self.data_ptr, self.capacity() + 1)
    }

    unsafe fn set_len(&mut self, len: u64) {
        self.len = len;
    }
}

impl<Allocator: BaseAllocator> Debug for PolymorphicString<'_, Allocator> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "PolymorphicString::<{}> {{ capacity: {}, len: {}, data: \"{}\" }}",
            core::any::type_name::<Allocator>(),
            self.capacity,
            self.len,
            as_escaped_string(self.as_bytes())
        )
    }
}

unsafe impl<Allocator: BaseAllocator> Send for PolymorphicString<'_, Allocator> {}

impl<Allocator: BaseAllocator> PartialOrd<PolymorphicString<'_, Allocator>>
    for PolymorphicString<'_, Allocator>
{
    fn partial_cmp(&self, other: &PolymorphicString<'_, Allocator>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<Allocator: BaseAllocator> Ord for PolymorphicString<'_, Allocator> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_bytes().cmp(other.as_bytes())
    }
}

impl<Allocator: BaseAllocator> Hash for PolymorphicString<'_, Allocator> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        state.write(self.as_bytes())
    }
}

impl<Allocator: BaseAllocator> Deref for PolymorphicString<'_, Allocator> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.as_bytes()
    }
}

impl<Allocator: BaseAllocator> DerefMut for PolymorphicString<'_, Allocator> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_bytes()
    }
}

impl<Allocator: BaseAllocator> PartialEq<PolymorphicString<'_, Allocator>>
    for PolymorphicString<'_, Allocator>
{
    fn eq(&self, other: &PolymorphicString<'_, Allocator>) -> bool {
        *self.as_bytes() == *other.as_bytes()
    }
}

impl<Allocator: BaseAllocator> Eq for PolymorphicString<'_, Allocator> {}

impl<Allocator: BaseAllocator> PartialEq<&[u8]> for PolymorphicString<'_, Allocator> {
    fn eq(&self, other: &&[u8]) -> bool {
        *self.as_bytes() == **other
    }
}

impl<Allocator: BaseAllocator> PartialEq<&str> for PolymorphicString<'_, Allocator> {
    fn eq(&self, other: &&str) -> bool {
        *self.as_bytes() == *other.as_bytes()
    }
}

impl<Allocator: BaseAllocator> PartialEq<PolymorphicString<'_, Allocator>> for &str {
    fn eq(&self, other: &PolymorphicString<'_, Allocator>) -> bool {
        *self.as_bytes() == *other.as_bytes()
    }
}

impl<const OTHER_CAPACITY: usize, Allocator: BaseAllocator> PartialEq<[u8; OTHER_CAPACITY]>
    for PolymorphicString<'_, Allocator>
{
    fn eq(&self, other: &[u8; OTHER_CAPACITY]) -> bool {
        *self.as_bytes() == *other
    }
}

impl<const OTHER_CAPACITY: usize, Allocator: BaseAllocator> PartialEq<&[u8; OTHER_CAPACITY]>
    for PolymorphicString<'_, Allocator>
{
    fn eq(&self, other: &&[u8; OTHER_CAPACITY]) -> bool {
        *self.as_bytes() == **other
    }
}

impl<Allocator: BaseAllocator> Display for PolymorphicString<'_, Allocator> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", as_escaped_string(self.as_bytes()))
    }
}

impl<'a, Allocator: BaseAllocator> PolymorphicString<'a, Allocator> {
    /// Creates a new [`PolymorphicString`].
    pub fn new(allocator: &'a Allocator, capacity: usize) -> Result<Self, AllocationError> {
        let layout = Layout::array::<MaybeUninit<u8>>(capacity + 1)
            .expect("Memory size for the array is smaller than isize::MAX");
        let mut data_ptr = match allocator.allocate(layout) {
            Ok(ptr) => ptr,
            Err(e) => {
                let origin = format!(
                    "PolymorphicString::<{}>::new(.., {})",
                    core::any::type_name::<Allocator>(),
                    capacity
                );
                fail!(from origin, with e,
                    "Failed to create new PolymorphicString due to a failure while allocating memory ({e:?}).");
            }
        };

        Ok(Self {
            data_ptr: unsafe { data_ptr.as_mut() }.as_mut_ptr().cast(),
            len: 0,
            capacity: capacity as _,
            allocator,
        })
    }

    /// Same as clone but it can fail when the required memory could not be
    /// allocated from the [`BaseAllocator`].
    pub fn try_clone(&self) -> Result<Self, AllocationError> {
        let layout = Layout::array::<MaybeUninit<u8>>(self.capacity as usize + 1)
            .expect("Memory size for the array is smaller than isize::MAX");

        let mut data_ptr = match self.allocator.allocate(layout) {
            Ok(ptr) => ptr,
            Err(e) => {
                let origin = format!(
                    "PolymorphicString::<{}>::try_clone()",
                    core::any::type_name::<Allocator>(),
                );
                fail!(from origin, with e,
                    "Failed to clone PolymorphicString due to a failure while allocating memory ({e:?}).");
            }
        };

        let mut new_self = Self {
            data_ptr: unsafe { data_ptr.as_mut() }.as_mut_ptr().cast(),
            len: 0,
            capacity: self.capacity,
            allocator: self.allocator,
        };

        unsafe { new_self.insert_bytes_unchecked(0, self.as_bytes()) };
        Ok(new_self)
    }
}

impl<Allocator: BaseAllocator> String for PolymorphicString<'_, Allocator> {
    fn capacity(&self) -> usize {
        self.capacity as usize
    }

    fn len(&self) -> usize {
        self.len as usize
    }
}
