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

//! Contains the [`RelocatableVec`](crate::relocatable_vec::RelocatableVec), a
//! run-time fixed size vector that is shared memory compatible
//!
//! # Expert Examples
//!
//! ## Create [`RelocatableVec`](crate::relocatable_vec::RelocatableVec) inside constructs which provides memory
//!
//! ```
//! use iceoryx2_bb_container::relocatable_vec::*;
//! use iceoryx2_bb_elementary::math::align_to;
//! use iceoryx2_bb_elementary::bump_allocator::BumpAllocator;
//! use core::mem::MaybeUninit;
//!
//! const VEC_CAPACITY:usize = 12;
//! struct MyConstruct {
//!     vec: RelocatableVec<u128>,
//!     vec_memory: [MaybeUninit<u128>; VEC_CAPACITY],
//! }
//!
//! impl MyConstruct {
//!     pub fn new() -> Self {
//!         let mut new_self = Self {
//!             vec: unsafe { RelocatableVec::new_uninit(VEC_CAPACITY) },
//!             vec_memory: core::array::from_fn(|_| MaybeUninit::uninit()),
//!         };
//!
//!         let allocator = BumpAllocator::new(new_self.vec_memory.as_mut_ptr().cast());
//!         unsafe {
//!             new_self.vec.init(&allocator).expect("Enough memory provided.")
//!         };
//!         new_self
//!     }
//! }
//! ```
//!
//! ## Create [`RelocatableVec`](crate::relocatable_vec::RelocatableVec) with allocator
//!
//! ```
//! use iceoryx2_bb_container::relocatable_vec::*;
//! use iceoryx2_bb_elementary::bump_allocator::BumpAllocator;
//! use core::ptr::NonNull;
//!
//! const VEC_CAPACITY:usize = 12;
//! const MEM_SIZE: usize = RelocatableVec::<u128>::const_memory_size(VEC_CAPACITY);
//! let mut memory = [0u8; MEM_SIZE];
//!
//! let bump_allocator = BumpAllocator::new(memory.as_mut_ptr());
//!
//! let mut vec = unsafe { RelocatableVec::<u128>::new_uninit(VEC_CAPACITY) };
//! unsafe { vec.init(&bump_allocator).expect("vec init failed") };
//! ```

use core::{fmt::Debug, mem::MaybeUninit};
use std::{
    alloc::Layout,
    ops::{Deref, DerefMut},
};

use iceoryx2_bb_elementary::{math::unaligned_mem_size, relocatable_ptr::*};
pub use iceoryx2_bb_elementary_traits::relocatable_container::RelocatableContainer;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use iceoryx2_bb_log::{fail, fatal_panic};

pub use crate::vector::Vector;

/// **Non-movable** relocatable shared-memory compatible vector with runtime fixed size capacity.
pub struct RelocatableVec<T> {
    data_ptr: RelocatablePointer<MaybeUninit<T>>,
    capacity: u64,
    len: u64,
}

impl<T> Drop for RelocatableVec<T> {
    fn drop(&mut self) {
        if self.data_ptr.is_initialized() {
            self.clear()
        }
    }
}

unsafe impl<T: Send> Send for RelocatableVec<T> {}

unsafe impl<T: ZeroCopySend> ZeroCopySend for RelocatableVec<T> {}

impl<T: Debug> Debug for RelocatableVec<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "RelocatableVec<{}> {{ capacity: {}, len: {}, content: [ ",
            core::any::type_name::<T>(),
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

impl<T> Deref for RelocatableVec<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.verify_init("deref()");
        self.as_slice()
    }
}

impl<T> DerefMut for RelocatableVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.verify_init("deref_mut()");
        self.as_mut_slice()
    }
}

impl<T: PartialEq> PartialEq for RelocatableVec<T> {
    fn eq(&self, other: &Self) -> bool {
        if other.len() != self.len() {
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

impl<T: Eq> Eq for RelocatableVec<T> {}

impl<T> RelocatableVec<T> {
    #[inline(always)]
    fn verify_init(&self, source: &str) {
        debug_assert!(
                self.data_ptr.is_initialized(),
                "From: MetaVec<{}>::{}, Undefined behavior - the object was not initialized with 'init' before.",
                core::any::type_name::<T>(), source
            );
    }

    /// Returns the required memory size for a vec with a specified capacity
    pub const fn const_memory_size(capacity: usize) -> usize {
        unaligned_mem_size::<T>(capacity)
    }
}

impl<T> RelocatableContainer for RelocatableVec<T> {
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
        if self.data_ptr.is_initialized() {
            let origin = format!("RelocatableVec<{}>::init()", core::any::type_name::<T>());
            fatal_panic!(from origin,
                "Memory already initialized, Initializing it twice may lead to undefined behavior.");
        }

        let ptr = match allocator.allocate(Layout::from_size_align_unchecked(
            core::mem::size_of::<T>() * self.capacity as usize,
            core::mem::align_of::<T>(),
        )) {
            Ok(ptr) => ptr,
            Err(e) => {
                let origin = format!("RelocatableVec<{}>::init()", core::any::type_name::<T>());
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

impl<T> Vector<T> for RelocatableVec<T> {
    fn as_mut_slice(&mut self) -> &mut [T] {
        self.verify_init("as_mut_slice()");
        unsafe { core::slice::from_raw_parts_mut(self.data_ptr.as_mut_ptr().cast(), self.len()) }
    }

    fn as_slice(&self) -> &[T] {
        self.verify_init("as_slice()");
        unsafe { core::slice::from_raw_parts(self.data_ptr.as_ptr().cast(), self.len()) }
    }

    fn capacity(&self) -> usize {
        self.capacity as usize
    }

    fn clear(&mut self) {
        self.verify_init("clear()");
        let ptr = unsafe { self.data_ptr.as_mut_ptr() };
        for idx in (0..self.len()).rev() {
            unsafe { (&mut *ptr.add(idx)).assume_init_drop() };
        }

        self.len = 0;
    }

    fn extend_from_slice(&mut self, other: &[T]) -> bool
    where
        T: Clone,
    {
        self.verify_init("extend_from_slice()");

        if self.capacity() < self.len() + other.len() {
            return false;
        }

        for (i, element) in other.iter().enumerate() {
            unsafe { &mut *self.data_ptr.as_mut_ptr().add(i + self.len()) }.write(element.clone());
        }

        self.len += other.len() as u64;

        true
    }

    fn insert(&mut self, index: usize, element: T) -> bool {
        self.verify_init("insert()");

        if index > self.len() {
            return false;
        }

        if index != self.len() {
            let ptr = unsafe { self.data_ptr.as_mut_ptr() };
            unsafe { core::ptr::copy(ptr.add(index), ptr.add(index + 1), self.len() - index) };
        }

        unsafe { &mut *self.data_ptr.as_mut_ptr().add(index) }.write(element);
        self.len += 1;
        true
    }

    fn is_empty(&self) -> bool {
        self.len == 0
    }

    fn is_full(&self) -> bool {
        self.len == self.capacity
    }

    fn len(&self) -> usize {
        self.len as usize
    }

    fn pop(&mut self) -> Option<T> {
        self.verify_init("pop()");

        if self.is_empty() {
            return None;
        }

        let value = core::mem::replace(
            unsafe { &mut *self.data_ptr.as_mut_ptr().add(self.len() - 1) },
            MaybeUninit::uninit(),
        );
        self.len -= 1;
        Some(unsafe { value.assume_init() })
    }

    fn push(&mut self, value: T) -> bool {
        self.verify_init("push()");

        if self.is_full() {
            return false;
        }

        unsafe { &mut *self.data_ptr.as_mut_ptr().add(self.len()) }.write(value);
        self.len += 1;
        true
    }

    fn remove(&mut self, index: usize) -> Option<T> {
        self.verify_init("remove()");

        if self.len() <= index {
            return None;
        }

        let ptr = unsafe { self.data_ptr.as_mut_ptr() };
        let value = unsafe { core::ptr::read(ptr.add(index)).assume_init() };

        unsafe { core::ptr::copy(ptr.add(index + 1), ptr.add(index), self.len() - index - 1) };

        self.len -= 1;

        Some(value)
    }

    fn resize(&mut self, new_len: usize, value: T) -> bool
    where
        T: Clone,
    {
        self.verify_init("resize()");
        self.resize_with(new_len, || value.clone())
    }

    fn resize_with<F: FnMut() -> T>(&mut self, new_len: usize, mut f: F) -> bool {
        self.verify_init("resize_with()");

        if self.capacity() < new_len {
            return false;
        }

        if new_len < self.len() {
            self.truncate(new_len);
        } else {
            let ptr = unsafe { self.data_ptr.as_mut_ptr() };
            for idx in self.len()..self.capacity() {
                unsafe { &mut *ptr.add(idx) }.write(f());
            }

            self.len = new_len as u64;
        }

        true
    }

    fn truncate(&mut self, len: usize) {
        self.verify_init("truncate()");

        if self.len() <= len {
            return;
        }

        let ptr = unsafe { self.data_ptr.as_mut_ptr() };
        for idx in (len..self.len()).rev() {
            unsafe { (&mut *ptr.add(idx)).assume_init_drop() };
        }

        self.len = len as u64;
    }
}
