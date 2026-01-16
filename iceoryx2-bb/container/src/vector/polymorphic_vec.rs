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

//! Vector implementation with a polymorphic stateful allocator.
//!
//! # Example
//!
//! ```no_run
//! # extern crate iceoryx2_bb_loggers;
//!
//! use iceoryx2_bb_elementary_traits::allocator::*;
//! use iceoryx2_bb_container::vector::*;
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
//! let mut my_vec =
//!     PolymorphicVec::<usize, SomeAllocator>::new(&allocator, capacity)?;
//!
//! my_vec.push(456); // returns false, when capacity is exceeded
//! # Ok(())
//! # }
//! ```

use alloc::format;
use core::{
    alloc::Layout,
    fmt::Debug,
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

use iceoryx2_bb_elementary_traits::allocator::{AllocationError, BaseAllocator};
use iceoryx2_log::fail;

use crate::vector::internal;
pub use crate::vector::Vector;

/// Runtime fixed-size vector variant with a polymorphic allocator, meaning an
/// allocator with a state can be attached to the vector instead of using a
/// stateless allocator like the heap-allocator.
pub struct PolymorphicVec<'a, T, Allocator: BaseAllocator> {
    data_ptr: *mut MaybeUninit<T>,
    len: u64,
    capacity: u64,
    allocator: &'a Allocator,
}

impl<T: Debug, Allocator: BaseAllocator> Debug for PolymorphicVec<'_, T, Allocator> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "PolymorphicVec<{}, {}> {{ capacity: {}, len: {} content: [ ",
            core::any::type_name::<T>(),
            core::any::type_name::<Allocator>(),
            self.capacity,
            self.len,
        )?;

        if !self.is_empty() {
            write!(f, "{:?}", self[0])?;
        }

        for idx in 1..self.len() {
            write!(f, ", {:?}", self[idx])?;
        }

        write!(f, " ] }}")
    }
}

impl<T, Allocator: BaseAllocator> Drop for PolymorphicVec<'_, T, Allocator> {
    fn drop(&mut self) {
        self.clear();
        unsafe {
            self.allocator.deallocate(
                NonNull::new_unchecked(self.data_ptr.cast()),
                Layout::array::<MaybeUninit<T>>(self.capacity as _).unwrap(),
            )
        };
    }
}

impl<T, Allocator: BaseAllocator> Deref for PolymorphicVec<'_, T, Allocator> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T, Allocator: BaseAllocator> DerefMut for PolymorphicVec<'_, T, Allocator> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}

impl<T: PartialEq, Allocator: BaseAllocator> PartialEq for PolymorphicVec<'_, T, Allocator> {
    fn eq(&self, other: &Self) -> bool {
        if self.len != other.len {
            return false;
        }

        for i in 0..self.len() {
            if other[i] != self[i] {
                return false;
            }
        }

        true
    }
}

impl<T: Eq, Allocator: BaseAllocator> Eq for PolymorphicVec<'_, T, Allocator> {}

unsafe impl<T: Send, Allocator: BaseAllocator> Send for PolymorphicVec<'_, T, Allocator> {}

impl<'a, T, Allocator: BaseAllocator> PolymorphicVec<'a, T, Allocator> {
    /// Creates a new [`PolymorphicVec`].
    pub fn new(allocator: &'a Allocator, capacity: usize) -> Result<Self, AllocationError> {
        let layout = Layout::array::<MaybeUninit<T>>(capacity as _)
            .expect("Memory size for the array is smaller than isize::MAX");
        let mut data_ptr = match allocator.allocate(layout) {
            Ok(ptr) => ptr,
            Err(e) => {
                let origin = format!(
                    "PolymorphicVec::<{}, {}>::new(.., {})",
                    core::any::type_name::<T>(),
                    core::any::type_name::<Allocator>(),
                    capacity
                );
                fail!(from origin, with e,
                    "Failed to create new PolymorphicVec due to a failure while allocating memory ({e:?}).");
            }
        };

        Ok(Self {
            data_ptr: unsafe { data_ptr.as_mut() }.as_mut_ptr().cast(),
            len: 0,
            capacity: capacity as _,
            allocator,
        })
    }

    /// Creates a new [`PolymorphicVec`] with the provided capacity and fills it by using the provided callback
    pub fn from_fn<F: FnMut(usize) -> T>(
        allocator: &'a Allocator,
        capacity: usize,
        mut func: F,
    ) -> Result<Self, AllocationError> {
        let mut new_self = Self::new(allocator, capacity)?;

        for n in 0..capacity {
            unsafe { new_self.push_unchecked(func(n)) };
        }

        Ok(new_self)
    }
}

impl<'a, T: Clone, Allocator: BaseAllocator> PolymorphicVec<'a, T, Allocator> {
    /// Same as clone but it can fail when the required memory could not be
    /// allocated from the [`BaseAllocator`].
    pub fn try_clone(&self) -> Result<Self, AllocationError> {
        let layout = Layout::array::<MaybeUninit<T>>(self.capacity as _)
            .expect("Memory size for the array is smaller than isize::MAX");

        let mut data_ptr = match self.allocator.allocate(layout) {
            Ok(ptr) => ptr,
            Err(e) => {
                let origin = format!(
                    "PolymorphicVec::<{}, {}>::try_clone()",
                    core::any::type_name::<T>(),
                    core::any::type_name::<Allocator>(),
                );
                fail!(from origin, with e,
                    "Failed to clone PolymorphicVec due to a failure while allocating memory ({e:?}).");
            }
        };

        let mut new_self = Self {
            data_ptr: unsafe { data_ptr.as_mut() }.as_mut_ptr().cast(),
            len: 0,
            capacity: self.capacity,
            allocator: self.allocator,
        };

        unsafe { new_self.extend_from_slice_unchecked(self.as_slice()) };
        Ok(new_self)
    }
}

impl<T, Allocator: BaseAllocator> internal::VectorView<T> for PolymorphicVec<'_, T, Allocator> {
    fn data(&self) -> &[MaybeUninit<T>] {
        unsafe { core::slice::from_raw_parts(self.data_ptr, self.capacity()) }
    }

    unsafe fn data_mut(&mut self) -> &mut [MaybeUninit<T>] {
        unsafe { core::slice::from_raw_parts_mut(self.data_ptr, self.capacity()) }
    }

    unsafe fn set_len(&mut self, len: u64) {
        self.len = len
    }
}

impl<T, Allocator: BaseAllocator> Vector<T> for PolymorphicVec<'_, T, Allocator> {
    fn capacity(&self) -> usize {
        self.capacity as _
    }

    fn len(&self) -> usize {
        self.len as usize
    }
}
