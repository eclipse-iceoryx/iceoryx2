// Copyright (c) 2023 Contributors to the Eclipse Foundation
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

//! Two relocatable (inter-process shared memory compatible) vector implementations.
//!
//!
//! The [`Vec`] which has a
//! fixed capacity defined at runtime and the [`FixedSizeVec`] which has a fixed capacity at
//! compile time.
//!
//! # Examples
//!
//! ## Create [`Vec`] inside constructs which provides memory
//!
//! ```
//! use iceoryx2_bb_container::vec::Vec;
//! use iceoryx2_bb_elementary::math::align_to;
//! use iceoryx2_bb_elementary::relocatable_container::RelocatableContainer;
//!
//! const VEC_CAPACITY:usize = 12;
//! struct MyConstruct {
//!     vec: Vec<u128>,
//!     data: [u128; VEC_CAPACITY],
//! }
//!
//! impl MyConstruct {
//!     pub fn new() -> Self {
//!         Self {
//!             vec: unsafe { Vec::new(VEC_CAPACITY,
//!                             align_to::<u128>(std::mem::size_of::<u128>()) as isize) },
//!             data: [0; VEC_CAPACITY]
//!         }
//!     }
//! }
//! ```
//!
//! ## Create [`Vec`] with allocator
//!
//! ```
//! use iceoryx2_bb_container::vec::Vec;
//! use iceoryx2_bb_elementary::bump_allocator::BumpAllocator;
//! use iceoryx2_bb_elementary::relocatable_container::RelocatableContainer;
//! use std::ptr::NonNull;
//!
//! const VEC_CAPACITY:usize = 12;
//! const MEM_SIZE: usize = Vec::<u128>::const_memory_size(VEC_CAPACITY);
//! let mut memory = [0u8; MEM_SIZE];
//!
//! let bump_allocator = BumpAllocator::new(memory.as_mut_ptr() as usize);
//!
//! let vec = unsafe { Vec::<u128>::new_uninit(VEC_CAPACITY) };
//! unsafe { vec.init(&bump_allocator).expect("vec init failed") };
//! ```

use std::{
    alloc::Layout,
    mem::MaybeUninit,
    ops::Deref,
    ops::DerefMut,
    sync::atomic::{AtomicBool, Ordering},
};

use iceoryx2_bb_elementary::{
    math::align_to, pointer_trait::PointerTrait, relocatable_container::RelocatableContainer,
    relocatable_ptr::RelocatablePointer,
};
use iceoryx2_bb_log::{fail, fatal_panic};

/// **Non-movable** relocatable vector with runtime fixed size capacity.
#[repr(C)]
#[derive(Debug)]
pub struct Vec<T> {
    data_ptr: RelocatablePointer<MaybeUninit<T>>,
    capacity: usize,
    len: usize,
    is_initialized: AtomicBool,
}

unsafe impl<T: Send> Send for Vec<T> {}
unsafe impl<T: Sync> Sync for Vec<T> {}

impl<T> Drop for Vec<T> {
    fn drop(&mut self) {
        unsafe { self.clear() };
    }
}

impl<T> RelocatableContainer for Vec<T> {
    unsafe fn new(capacity: usize, distance_to_data: isize) -> Self {
        Self {
            data_ptr: RelocatablePointer::new(distance_to_data),
            capacity,
            len: 0,
            is_initialized: AtomicBool::new(true),
        }
    }

    unsafe fn new_uninit(capacity: usize) -> Self {
        Self {
            data_ptr: RelocatablePointer::new_uninit(),
            capacity,
            len: 0,
            is_initialized: AtomicBool::new(false),
        }
    }

    unsafe fn init<Allocator: iceoryx2_bb_elementary::allocator::BaseAllocator>(
        &self,
        allocator: &Allocator,
    ) -> Result<(), iceoryx2_bb_elementary::allocator::AllocationError> {
        if self.is_initialized.load(Ordering::Relaxed) {
            fatal_panic!(from "Vec::init()", "Memory already initialized, Initializing it twice may lead to undefined behavior.");
        }

        self.data_ptr.init(fail!(from "Queue::init", when allocator
             .allocate(Layout::from_size_align_unchecked(
                 std::mem::size_of::<T>() * self.capacity,
                 std::mem::align_of::<T>(),
             )), "Failed to initialize queue since the allocation of the data memory failed."
        ));
        self.is_initialized
            .store(true, std::sync::atomic::Ordering::Relaxed);

        Ok(())
    }

    fn memory_size(capacity: usize) -> usize {
        Self::const_memory_size(capacity)
    }
}

impl<T> Deref for Vec<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.verify_init(&format!("Vec<{}>::push()", std::any::type_name::<T>()));
        unsafe { core::slice::from_raw_parts((*self.data_ptr.as_ptr()).as_ptr(), self.len) }
    }
}

impl<T> DerefMut for Vec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.verify_init(&format!("Vec<{}>::push()", std::any::type_name::<T>()));
        unsafe {
            core::slice::from_raw_parts_mut((*self.data_ptr.as_mut_ptr()).as_mut_ptr(), self.len)
        }
    }
}

impl<T: PartialEq> PartialEq for Vec<T> {
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

impl<T: Eq> Eq for Vec<T> {}

impl<T> Vec<T> {
    fn verify_init(&self, source: &str) {
        if !self
            .is_initialized
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            fatal_panic!(from source, "Undefined behavior - the object was not initialized with 'init' before.");
        }
    }

    /// Returns the required memory size for a vec with a specified capacity
    pub const fn const_memory_size(capacity: usize) -> usize {
        std::mem::size_of::<T>() * capacity + std::mem::align_of::<T>() - 1
    }

    /// Returns the capacity of the vector
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Returns the number of elements stored inside the vector
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns true if the vector is empty, otherwise false
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns true if the vector is full, otherwise false
    pub fn is_full(&self) -> bool {
        self.len == self.capacity
    }

    /// Adds an element at the end of the vector. If the vector is full and the element cannot be
    /// added it returns false, otherwise true.
    ///
    /// # Safety
    ///
    ///  * Only use this method when [`Vec::init()`] was called before
    ///
    pub unsafe fn push(&mut self, value: T) -> bool {
        if self.is_full() {
            return false;
        }

        self.verify_init(&format!("Vec<{}>::push()", std::any::type_name::<T>()));
        self.push_unchecked(value);
        true
    }

    /// Fill the remaining space of the vector with value.
    ///
    /// # Safety
    ///
    ///  * Only use this method when [`Vec::init()`] was called before
    ///
    pub unsafe fn fill(&mut self, value: T)
    where
        T: Clone,
    {
        for _ in self.len..self.capacity {
            self.push_unchecked(value.clone());
        }
    }

    unsafe fn push_unchecked(&mut self, value: T) {
        self.data_ptr
            .as_mut_ptr()
            .add(self.len)
            .write(MaybeUninit::new(value));

        self.len += 1;
    }

    /// Append all elements from other via [`Clone`].
    ///
    /// # Safety
    ///
    ///  * Only use this method when [`Vec::init()`] was called before
    ///
    pub unsafe fn extend_from_slice(&mut self, other: &[T]) -> bool
    where
        T: Clone,
    {
        if self.capacity < self.len + other.len() {
            return false;
        }

        for element in other {
            self.push_unchecked(element.clone());
        }

        true
    }

    /// Removes the last element of the vector and returns it to the user. If the vector is empty
    /// it returns [`None`].
    ///
    /// # Safety
    ///
    ///  * Only use this method when [`Vec::init()`] was called before
    ///
    pub unsafe fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }

        self.verify_init(&format!("Vec<{}>::pop()", std::any::type_name::<T>()));
        Some(self.pop_unchecked())
    }

    /// Removes all elements from the vector
    ///
    /// # Safety
    ///
    ///  * Only use this method when [`Vec::init()`] was called before
    ///
    pub unsafe fn clear(&mut self) {
        for _ in 0..self.len {
            self.pop_unchecked();
        }
    }

    unsafe fn pop_unchecked(&mut self) -> T {
        let value = std::mem::replace(
            &mut *self.data_ptr.as_mut_ptr().offset(self.len as isize - 1),
            MaybeUninit::uninit(),
        );
        self.len -= 1;

        value.assume_init()
    }
}

/// Relocatable vector with compile time fixed size capacity. In contrast to its counterpart the
/// [`Vec`] it is movable.
#[repr(C)]
#[derive(Debug)]
pub struct FixedSizeVec<T, const CAPACITY: usize> {
    state: Vec<T>,
    _data: [MaybeUninit<T>; CAPACITY],
}

impl<T, const CAPACITY: usize> Default for FixedSizeVec<T, CAPACITY> {
    fn default() -> Self {
        Self {
            state: unsafe {
                Vec::new(
                    CAPACITY,
                    align_to::<MaybeUninit<T>>(std::mem::size_of::<Vec<T>>()) as isize,
                )
            },
            _data: unsafe { MaybeUninit::uninit().assume_init() },
        }
    }
}

impl<T, const CAPACITY: usize> Deref for FixedSizeVec<T, CAPACITY> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.state.deref()
    }
}

impl<T, const CAPACITY: usize> DerefMut for FixedSizeVec<T, CAPACITY> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.state.deref_mut()
    }
}

impl<T: PartialEq, const CAPACITY: usize> PartialEq for FixedSizeVec<T, CAPACITY> {
    fn eq(&self, other: &Self) -> bool {
        self.state.eq(&other.state)
    }
}

impl<T: Eq, const CAPACITY: usize> Eq for FixedSizeVec<T, CAPACITY> {}

impl<T: Clone, const CAPACITY: usize> Clone for FixedSizeVec<T, CAPACITY> {
    fn clone(&self) -> Self {
        let mut new_self = Self::new();
        new_self.extend_from_slice(self.deref());
        new_self
    }
}

unsafe impl<T: Send, const CAPACITY: usize> Send for FixedSizeVec<T, CAPACITY> {}
unsafe impl<T: Sync, const CAPACITY: usize> Sync for FixedSizeVec<T, CAPACITY> {}

impl<T, const CAPACITY: usize> FixedSizeVec<T, CAPACITY> {
    /// Creates a new vector.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns true if the vector is empty, otherwise false
    pub fn is_empty(&self) -> bool {
        self.state.is_empty()
    }

    /// Returns true if the vector is full, otherwise false
    pub fn is_full(&self) -> bool {
        self.state.is_full()
    }

    /// Returns the capacity of the vector
    pub fn capacity(&self) -> usize {
        self.state.capacity()
    }

    /// Returns the number of elements stored inside the vector
    pub fn len(&self) -> usize {
        self.state.len()
    }

    /// Adds an element at the end of the vector. If the vector is full and the element cannot be
    /// added it returns false, otherwise true.
    pub fn push(&mut self, value: T) -> bool {
        unsafe { self.state.push(value) }
    }

    /// Fill the remaining space of the vector with value.
    pub fn fill(&mut self, value: T)
    where
        T: Clone,
    {
        unsafe { self.state.fill(value) }
    }

    /// Append all elements from other via [`Clone`].
    pub fn extend_from_slice(&mut self, other: &[T]) -> bool
    where
        T: Clone,
    {
        unsafe { self.state.extend_from_slice(other) }
    }

    /// Removes the last element of the vector and returns it to the user. If the vector is empty
    /// it returns [`None`].
    pub fn pop(&mut self) -> Option<T> {
        unsafe { self.state.pop() }
    }

    /// Removes all elements from the vector
    pub fn clear(&mut self) {
        unsafe { self.state.clear() }
    }
}
