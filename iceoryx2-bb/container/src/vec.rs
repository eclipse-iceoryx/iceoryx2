// Copyright (c) 2023 - 2024 Contributors to the Eclipse Foundation
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

//! Contains vector variations that are similar to [`std::vec::Vec`].
//!
//!  * [`Vec`](crate::vec::Vec), run-time fixed-size vector that is not shared-memory compatible
//!     since the memory resides in the heap.
//!  * [`FixedSizeVec`](crate::vec::FixedSizeVec), compile-time fixed size vector that is
//!     self-contained.
//!  * [`RelocatableVec`](crate::vec::RelocatableVec), run-time fixed size vector that uses by default heap memory.
//!
//! # User Examples
//!
//! ```
//! use iceoryx2_bb_container::vec::FixedSizeVec;
//!
//! const VEC_CAPACITY: usize = 123;
//! let mut my_vec = FixedSizeVec::<u64, VEC_CAPACITY>::new();
//!
//! my_vec.push(283);
//! my_vec.push(787);
//!
//! println!("vec contents {:?}", my_vec);
//! ```
//!
//! # Expert Examples
//!
//! ## Create [`RelocatableVec`](crate::vec::RelocatableVec) inside constructs which provides memory
//!
//! ```
//! use iceoryx2_bb_container::vec::RelocatableVec;
//! use iceoryx2_bb_elementary::math::align_to;
//! use iceoryx2_bb_elementary::bump_allocator::BumpAllocator;
//! use iceoryx2_bb_elementary_traits::relocatable_container::RelocatableContainer;
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
//! ## Create [`RelocatableVec`](crate::vec::RelocatableVec) with allocator
//!
//! ```
//! use iceoryx2_bb_container::vec::RelocatableVec;
//! use iceoryx2_bb_elementary::bump_allocator::BumpAllocator;
//! use iceoryx2_bb_elementary_traits::relocatable_container::RelocatableContainer;
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

use core::{
    alloc::Layout,
    fmt::Debug,
    marker::PhantomData,
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
};

use iceoryx2_bb_elementary::{
    bump_allocator::BumpAllocator, relocatable_ptr::GenericRelocatablePointer,
};
use iceoryx2_bb_elementary_traits::{
    generic_pointer::GenericPointer, owning_pointer::GenericOwningPointer,
    owning_pointer::OwningPointer, placement_default::PlacementDefault,
    pointer_trait::PointerTrait, relocatable_container::RelocatableContainer,
    zero_copy_send::ZeroCopySend,
};

use iceoryx2_bb_elementary::{math::unaligned_mem_size, relocatable_ptr::RelocatablePointer};

use iceoryx2_bb_log::{fail, fatal_panic};
use serde::{de::Visitor, Deserialize, Serialize};

/// Vector with run-time fixed size capacity. In contrast to its counterpart the
/// [`RelocatableVec`] it is movable but is not shared memory compatible.
pub type Vec<T> = MetaVec<T, GenericOwningPointer>;

/// **Non-movable** relocatable vector with runtime fixed size capacity.
pub type RelocatableVec<T> = MetaVec<T, GenericRelocatablePointer>;

#[doc(hidden)]
/// **Non-movable** relocatable vector with runtime fixed size capacity.
#[repr(C)]
pub struct MetaVec<T, Ptr: GenericPointer> {
    data_ptr: Ptr::Type<MaybeUninit<T>>,
    capacity: usize,
    len: usize,
    _phantom_data: PhantomData<T>,
}

impl<T: Debug, Ptr: GenericPointer> Debug for MetaVec<T, Ptr> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "MetaVec<{}, {}> {{ capacity: {}, len: {}, is_initialized: {}, content: [ ",
            core::any::type_name::<T>(),
            core::any::type_name::<Ptr>(),
            self.capacity,
            self.len,
            self.data_ptr.is_initialized(),
        )?;

        if self.len > 0 {
            write!(f, "{:?}", self[0])?;
        }

        for n in 1..self.len {
            write!(f, ", {:?}", self[n])?;
        }

        write!(f, " ] }}")
    }
}

unsafe impl<T: Send, Ptr: GenericPointer> Send for MetaVec<T, Ptr> {}

impl<T, Ptr: GenericPointer> Drop for MetaVec<T, Ptr> {
    fn drop(&mut self) {
        if self.data_ptr.is_initialized() {
            unsafe { self.clear_impl() };
        }
    }
}

impl<T> RelocatableContainer for RelocatableVec<T> {
    unsafe fn new_uninit(capacity: usize) -> Self {
        Self {
            data_ptr: RelocatablePointer::new_uninit(),
            capacity,
            len: 0,
            _phantom_data: PhantomData,
        }
    }

    unsafe fn init<Allocator: iceoryx2_bb_elementary_traits::allocator::BaseAllocator>(
        &mut self,
        allocator: &Allocator,
    ) -> Result<(), iceoryx2_bb_elementary_traits::allocator::AllocationError> {
        if self.data_ptr.is_initialized() {
            fatal_panic!(from "Vec::init()", "Memory already initialized, Initializing it twice may lead to undefined behavior.");
        }

        self.data_ptr.init(fail!(from "Vec::init", when allocator
             .allocate(Layout::from_size_align_unchecked(
                 core::mem::size_of::<T>() * self.capacity,
                 core::mem::align_of::<T>(),
             )), "Failed to initialize vec since the allocation of the data memory failed."
        ));

        Ok(())
    }

    fn memory_size(capacity: usize) -> usize {
        Self::const_memory_size(capacity)
    }
}

impl<T, Ptr: GenericPointer> Deref for MetaVec<T, Ptr> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.verify_init("deref()");
        unsafe { self.as_slice_impl() }
    }
}

impl<T, Ptr: GenericPointer> DerefMut for MetaVec<T, Ptr> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.verify_init("deref_mut()");
        unsafe { self.as_mut_slice_impl() }
    }
}

impl<T: PartialEq, Ptr: GenericPointer> PartialEq for MetaVec<T, Ptr> {
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

impl<T: Eq, Ptr: GenericPointer> Eq for MetaVec<T, Ptr> {}

impl<T, Ptr: GenericPointer> MetaVec<T, Ptr> {
    #[inline(always)]
    fn verify_init(&self, source: &str) {
        debug_assert!(
                self.data_ptr.is_initialized(),
                "From: MetaVec<{}>::{}, Undefined behavior - the object was not initialized with 'init' before.",
                core::any::type_name::<T>(), source
            );
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

    pub(crate) unsafe fn push_impl(&mut self, value: T) -> bool {
        if self.is_full() {
            return false;
        }

        self.verify_init("push()");
        self.push_unchecked(value);
        true
    }

    unsafe fn fill_impl(&mut self, value: T)
    where
        T: Clone,
    {
        // NOTE: Since 'self.data_ptr' is a relocatable pointer, converting it
        // to an absolute pointer is expensive and should not be done in the loop
        let data_ptr = self.data_ptr.as_mut_ptr();
        for i in self.len..self.capacity {
            data_ptr.add(i).write(MaybeUninit::new(value.clone()));
        }
        self.len = self.capacity;
    }

    unsafe fn fill_with_impl<F: FnMut() -> T>(&mut self, mut f: F) {
        // NOTE: Since 'self.data_ptr' is a relocatable pointer, converting it
        // to an absolute pointer is expensive and should not be done in the loop
        let data_ptr = self.data_ptr.as_mut_ptr();
        for i in self.len..self.capacity {
            data_ptr.add(i).write(MaybeUninit::new(f()));
        }
        self.len = self.capacity;
    }

    fn push_unchecked(&mut self, value: T) {
        unsafe {
            self.data_ptr
                .as_mut_ptr()
                .add(self.len)
                .write(MaybeUninit::new(value))
        };

        self.len += 1;
    }

    unsafe fn extend_from_slice_impl(&mut self, other: &[T]) -> bool
    where
        T: Clone,
    {
        if self.capacity < self.len + other.len() {
            return false;
        }

        // NOTE: Since 'self.data_ptr' is a relocatable pointer, converting it
        // to an absolute pointer is expensive and should not be done in the loop
        let data_ptr = self.data_ptr.as_mut_ptr().add(self.len);
        for (i, element) in other.iter().enumerate() {
            data_ptr.add(i).write(MaybeUninit::new(element.clone()));
        }

        self.len += other.len();

        true
    }

    unsafe fn remove_impl(&mut self, index: usize) -> T {
        self.verify_init("remove()");
        debug_assert!(index < self.len());

        let ptr = self.as_mut_ptr().add(index);
        let value = core::ptr::read(ptr);
        core::ptr::copy(ptr.add(1), ptr, self.len - index - 1);
        self.len -= 1;
        value
    }

    unsafe fn pop_impl(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }

        self.verify_init("pop()");
        Some(self.pop_unchecked())
    }

    unsafe fn clear_impl(&mut self) {
        for _ in 0..self.len {
            self.pop_unchecked();
        }
    }

    fn pop_unchecked(&mut self) -> T {
        let value = core::mem::replace(
            unsafe { &mut *self.data_ptr.as_mut_ptr().offset(self.len as isize - 1) },
            MaybeUninit::uninit(),
        );
        self.len -= 1;

        unsafe { value.assume_init() }
    }

    unsafe fn as_slice_impl(&self) -> &[T] {
        unsafe { core::slice::from_raw_parts(self.data_ptr.as_ptr().cast(), self.len) }
    }

    unsafe fn as_mut_slice_impl(&mut self) -> &mut [T] {
        unsafe { core::slice::from_raw_parts_mut(self.data_ptr.as_mut_ptr().cast(), self.len) }
    }
}

impl<T> Vec<T> {
    /// Creates a new [`Vec`] with the provided capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            data_ptr: OwningPointer::<MaybeUninit<T>>::new_with_alloc(capacity),
            capacity,
            len: 0,
            _phantom_data: PhantomData,
        }
    }

    /// Creates a new [`Vec`] with the provided capacity and fills it by
    /// using the provided callback
    pub fn from_fn<F: FnMut(usize) -> T>(capacity: usize, mut callback: F) -> Self {
        let mut new_self = Self::new(capacity);
        for n in 0..capacity {
            new_self.push(callback(n));
        }
        new_self
    }

    /// Adds an element at the end of the vector. If the vector is full and the element cannot be
    /// added it returns false, otherwise true.
    pub fn push(&mut self, value: T) -> bool {
        unsafe { self.push_impl(value) }
    }

    /// Fill the remaining space of the vector with value.
    pub fn fill(&mut self, value: T)
    where
        T: Clone,
    {
        unsafe { self.fill_impl(value) }
    }

    /// Fill the remaining space of the vector by calling the provided closure repeatedly
    pub fn fill_with<F: FnMut() -> T>(&mut self, f: F) {
        unsafe { self.fill_with_impl(f) }
    }

    /// Append all elements from other via [`Clone`].
    pub fn extend_from_slice(&mut self, other: &[T]) -> bool
    where
        T: Clone,
    {
        unsafe { self.extend_from_slice_impl(other) }
    }

    /// Removes the last element of the vector and returns it to the user. If the vector is empty
    /// it returns [`None`].
    pub fn pop(&mut self) -> Option<T> {
        unsafe { self.pop_impl() }
    }

    /// Removes the element at the provided index and returns it.
    pub fn remove(&mut self, index: usize) -> T {
        unsafe { self.remove_impl(index) }
    }

    /// Removes all elements from the vector
    pub fn clear(&mut self) {
        unsafe { self.clear_impl() }
    }

    /// Returns a slice to the contents of the vector
    pub fn as_slice(&self) -> &[T] {
        unsafe { self.as_slice_impl() }
    }

    /// Returns a mutable slice to the contents of the vector
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { self.as_mut_slice_impl() }
    }
}

unsafe impl<T: ZeroCopySend> ZeroCopySend for RelocatableVec<T> {}

impl<T> RelocatableVec<T> {
    /// Returns the required memory size for a vec with a specified capacity
    pub const fn const_memory_size(capacity: usize) -> usize {
        unaligned_mem_size::<T>(capacity)
    }

    /// Adds an element at the end of the vector. If the vector is full and the element cannot be
    /// added it returns false, otherwise true.
    ///
    /// # Safety
    ///
    ///  * [`RelocatableVec::init()`] must be called once before
    ///
    pub unsafe fn push(&mut self, value: T) -> bool {
        self.push_impl(value)
    }

    /// Fill the remaining space of the vector with value.
    ///
    /// # Safety
    ///
    ///  * [`RelocatableVec::init()`] must be called once before
    ///
    pub unsafe fn fill(&mut self, value: T)
    where
        T: Clone,
    {
        self.fill_impl(value)
    }

    /// Fill the remaining space of the vector by calling the provided closure repeatedly
    ///
    /// # Safety
    ///
    ///  * [`RelocatableVec::init()`] must be called once before
    ///
    pub unsafe fn fill_with<F: FnMut() -> T>(&mut self, f: F) {
        self.fill_with_impl(f)
    }

    /// Append all elements from other via [`Clone`].
    ///
    /// # Safety
    ///
    ///  * [`RelocatableVec::init()`] must be called once before
    ///
    pub unsafe fn extend_from_slice(&mut self, other: &[T]) -> bool
    where
        T: Clone,
    {
        self.extend_from_slice_impl(other)
    }

    /// Removes the last element of the vector and returns it to the user. If the vector is empty
    /// it returns [`None`].
    ///
    /// # Safety
    ///
    ///  * [`RelocatableVec::init()`] must be called once before
    ///
    pub unsafe fn pop(&mut self) -> Option<T> {
        self.pop_impl()
    }

    /// Removes the element at the provided index and returns it.
    ///
    /// # Safety
    ///
    ///  * [`RelocatableVec::init()`] must be called once before
    ///
    pub unsafe fn remove(&mut self, index: usize) -> T {
        unsafe { self.remove_impl(index) }
    }

    /// Removes all elements from the vector
    ///
    /// # Safety
    ///
    ///  * [`RelocatableVec::init()`] must be called once before
    ///
    pub unsafe fn clear(&mut self) {
        self.clear_impl()
    }

    /// Returns a slice to the contents of the vector
    ///
    /// # Safety
    ///
    ///  * [`RelocatableVec::init()`] must be called once before
    ///
    pub unsafe fn as_slice(&self) -> &[T] {
        self.as_slice_impl()
    }

    /// Returns a mutable slice to the contents of the vector
    ///
    /// # Safety
    ///
    ///  * [`RelocatableVec::init()`] must be called once before
    ///
    pub unsafe fn as_mut_slice(&mut self) -> &mut [T] {
        self.as_mut_slice_impl()
    }
}

/// Relocatable vector with compile time fixed size capacity. In contrast to its counterpart the
/// [`RelocatableVec`] it is movable.
#[repr(C)]
pub struct FixedSizeVec<T, const CAPACITY: usize> {
    state: RelocatableVec<T>,
    _data: [MaybeUninit<T>; CAPACITY],
}

impl<T: Debug, const CAPACITY: usize> Debug for FixedSizeVec<T, CAPACITY> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "FixedSizeVec<{}, {}> {{ {:?} }}",
            core::any::type_name::<T>(),
            CAPACITY,
            self.state
        )
    }
}

unsafe impl<T: ZeroCopySend, const CAPACITY: usize> ZeroCopySend for FixedSizeVec<T, CAPACITY> {}

impl<'de, T: Serialize + Deserialize<'de>, const CAPACITY: usize> Serialize
    for FixedSizeVec<T, CAPACITY>
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.as_slice().serialize(serializer)
    }
}

struct FixedSizeVecVisitor<T, const CAPACITY: usize> {
    _value: PhantomData<T>,
}

impl<'de, T: Deserialize<'de>, const CAPACITY: usize> Visitor<'de>
    for FixedSizeVecVisitor<T, CAPACITY>
{
    type Value = FixedSizeVec<T, CAPACITY>;

    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        let str = format!(
            "an array of at most {} elements of type {}",
            CAPACITY,
            core::any::type_name::<T>()
        );
        formatter.write_str(&str)
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut new_vec = Self::Value::new();

        while let Some(element) = seq.next_element()? {
            if !new_vec.push(element) {
                return Err(<A::Error as serde::de::Error>::custom(format!(
                    "the array can hold at most {CAPACITY} elements"
                )));
            }
        }

        Ok(new_vec)
    }
}

impl<'de, T: Deserialize<'de>, const CAPACITY: usize> Deserialize<'de>
    for FixedSizeVec<T, CAPACITY>
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(FixedSizeVecVisitor::<T, CAPACITY> {
            _value: PhantomData,
        })
    }
}

impl<T, const CAPACITY: usize> PlacementDefault for FixedSizeVec<T, CAPACITY> {
    unsafe fn placement_default(ptr: *mut Self) {
        let state_ptr = core::ptr::addr_of_mut!((*ptr).state);
        state_ptr.write(unsafe { RelocatableVec::new_uninit(CAPACITY) });
        let allocator = BumpAllocator::new((*ptr)._data.as_mut_ptr().cast());
        (*ptr)
            .state
            .init(&allocator)
            .expect("All required memory is preallocated.");
    }
}

impl<T, const CAPACITY: usize> Default for FixedSizeVec<T, CAPACITY> {
    fn default() -> Self {
        let mut new_self = Self {
            state: unsafe { RelocatableVec::new_uninit(CAPACITY) },
            _data: core::array::from_fn(|_| MaybeUninit::uninit()),
        };

        let allocator = BumpAllocator::new(new_self._data.as_mut_ptr().cast());
        unsafe {
            new_self
                .state
                .init(&allocator)
                .expect("All required memory is preallocated.")
        };

        new_self
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
    pub const fn capacity() -> usize {
        CAPACITY
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

    /// Fill the remaining space of the vector with value.
    pub fn fill_with<F: FnMut() -> T>(&mut self, f: F) {
        unsafe { self.state.fill_with(f) }
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

    /// Removes the element at the provided index and returns it.
    pub fn remove(&mut self, index: usize) -> T {
        unsafe { self.state.remove(index) }
    }

    /// Removes all elements from the vector
    pub fn clear(&mut self) {
        unsafe { self.state.clear() }
    }

    /// Returns a slice to the contents of the vector
    pub fn as_slice(&self) -> &[T] {
        unsafe { self.state.as_slice() }
    }

    /// Returns a mutable slice to the contents of the vector
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { self.state.as_mut_slice() }
    }
}
