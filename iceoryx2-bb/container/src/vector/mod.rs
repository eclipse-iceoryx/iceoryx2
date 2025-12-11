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

use alloc::format;
use core::mem::MaybeUninit;
use core::ops::{Deref, DerefMut};

/// Runtime fixed-capacity vector where the user can provide a stateful allocator.
pub mod polymorphic_vec;
/// Runtime fixed-capacity shared-memory compatible vector
pub mod relocatable_vec;
/// Compile-time fixed-capacity shared-memory compatible vector
pub mod static_vec;

use iceoryx2_log::fail;
pub use polymorphic_vec::*;
pub use relocatable_vec::*;
pub use static_vec::*;

/// Error which can occur when a [`Vector`] is modified.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum VectorModificationError {
    /// An element shall be modified that is not contained in the vector.
    OutOfBounds,
    /// The content that shall be added would exceed the maximum capacity of the
    /// [`Vector`].
    InsertWouldExceedCapacity,
}

impl core::fmt::Display for VectorModificationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "VectorModificationError::{self:?}")
    }
}

impl core::error::Error for VectorModificationError {}

pub(crate) mod internal {
    use super::*;

    #[doc(hidden)]
    pub trait VectorView<T> {
        fn data(&self) -> &[MaybeUninit<T>];

        /// # Safety
        ///
        /// * user must ensure that any modification keeps the initialized data contiguous
        /// * user must update len with [`VectorView::set_len()`] when adding/removing elements
        unsafe fn data_mut(&mut self) -> &mut [MaybeUninit<T>];

        /// # Safety
        ///
        /// * user must ensure that the len defines the number of initialized contiguous
        ///   elements in [`VectorView::data_mut()`] and [`VectorView::data()`]
        unsafe fn set_len(&mut self, len: u64);
    }
}

/// Defines the interface of a vector.
pub trait Vector<T>: Deref<Target = [T]> + DerefMut + internal::VectorView<T> {
    /// Returns a mutable slice to the contents of the vector
    fn as_mut_slice(&mut self) -> &mut [T] {
        let len = self.len();
        unsafe {
            core::mem::transmute::<&mut [MaybeUninit<T>], &mut [T]>(&mut self.data_mut()[0..len])
        }
    }

    /// Returns a slice to the contents of the vector
    fn as_slice(&self) -> &[T] {
        let len = self.len();
        unsafe { core::mem::transmute::<&[MaybeUninit<T>], &[T]>(&self.data()[0..len]) }
    }

    /// Returns the capacity of the vector
    fn capacity(&self) -> usize;

    /// Removes all elements from the vector
    fn clear(&mut self) {
        let len = self.len();
        let data = unsafe { self.data_mut() };
        for idx in (0..len).rev() {
            unsafe { data[idx].assume_init_drop() };
        }

        unsafe { self.set_len(0) };
    }

    /// Append all elements from other via [`Clone`].
    fn extend_from_slice(&mut self, other: &[T]) -> Result<(), VectorModificationError>
    where
        T: Clone,
    {
        if self.capacity() < self.len() + other.len() {
            let origin = format!(
                "Vector::<{}>::extend_from_slice()",
                core::any::type_name::<T>()
            );
            fail!(from origin, with VectorModificationError::InsertWouldExceedCapacity,
                "Unable to extend vector from slice with length {} since it would exceed the vectors capacity of {}.",
                other.len(), self.capacity());
        }

        let len = self.len();
        let data = unsafe { self.data_mut() };
        for (i, element) in other.iter().enumerate() {
            data[i + len].write(element.clone());
        }

        unsafe { self.set_len(self.len() as u64 + other.len() as u64) };

        Ok(())
    }

    /// Append all elements from other via [`Clone`].
    ///
    /// # Safety
    ///
    /// * [`Vector::capacity()`] < [`Vector::len()`] + `other.len()`
    ///
    unsafe fn extend_from_slice_unchecked(&mut self, other: &[T])
    where
        T: Clone,
    {
        let len = self.len();
        let data = unsafe { self.data_mut() };
        for (i, element) in other.iter().enumerate() {
            data[i + len].write(element.clone());
        }

        unsafe { self.set_len(self.len() as u64 + other.len() as u64) };
    }

    /// Inserts an element at the provided index and shifting all elements
    /// after the index to the right.
    fn insert(&mut self, index: usize, element: T) -> Result<(), VectorModificationError> {
        if self.is_full() {
            let origin = format!("Vector::<{}>::insert()", core::any::type_name::<T>());

            fail!(from origin, with VectorModificationError::InsertWouldExceedCapacity,
                "Failed to insert element into vector since it would exceed the vectors capacity of {}.",
                self.capacity());
        }

        let len = self.len();
        if index > len {
            let origin = format!("Vector::<{}>::insert()", core::any::type_name::<T>());

            fail!(from origin, with VectorModificationError::OutOfBounds,
                "Failed to insert element into vector of length {} since the index {} is out-of-bounds.",
                self.len(), index);
        }

        let data = unsafe { self.data_mut() };
        if index != len {
            let ptr = data.as_mut_ptr();
            unsafe { core::ptr::copy(ptr.add(index), ptr.add(index + 1), len - index) };
        }

        data[index].write(element);
        unsafe { self.set_len(len as u64 + 1) };

        Ok(())
    }

    /// Returns true if the vector is empty, otherwise false
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns true if the vector is full, otherwise false
    fn is_full(&self) -> bool {
        self.len() == self.capacity()
    }

    /// Returns the number of elements stored inside the vector
    fn len(&self) -> usize;

    /// Removes the last element of the vector and returns it to the user. If the vector is empty
    /// it returns [`None`].
    fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }

        let len = self.len();
        let data = unsafe { self.data_mut() };
        let value = core::mem::replace(&mut data[len - 1], MaybeUninit::uninit());
        unsafe { self.set_len(len as u64 - 1) };
        Some(unsafe { value.assume_init() })
    }

    /// Adds an element at the end of the vector. If the vector is full and the element cannot be
    /// added it returns [`VectorModificationError::InsertWouldExceedCapacity`].
    fn push(&mut self, value: T) -> Result<(), VectorModificationError> {
        if self.is_full() {
            let origin = format!("Vector::<{}>::push()", core::any::type_name::<T>());

            fail!(from origin, with VectorModificationError::InsertWouldExceedCapacity,
                "Failed to push element into vector since it would exceed the vectors capacity of {}.",
                self.capacity());
        }

        unsafe { self.push_unchecked(value) };
        Ok(())
    }

    /// Adds an element at the end of the vector.
    ///
    /// # Safety
    ///
    /// * [`Vector::len()`] < [`Vector::capacity()`]
    ///
    unsafe fn push_unchecked(&mut self, value: T) {
        let len = self.len();
        unsafe { self.data_mut()[len].write(value) };
        unsafe { self.set_len(len as u64 + 1) };
    }

    /// Removes the element at the provided index and returns it.
    fn remove(&mut self, index: usize) -> Option<T> {
        let len = self.len();
        if len <= index {
            return None;
        }

        let data = unsafe { self.data_mut() };
        let value = unsafe { core::ptr::read(data[index].as_ptr()) };

        let ptr = data.as_mut_ptr();
        unsafe { core::ptr::copy(ptr.add(index + 1), ptr.add(index), len - index - 1) };

        unsafe { self.set_len(len as u64 - 1) };

        Some(value)
    }

    /// Fill the remaining space of the vector with value.
    fn resize(&mut self, new_len: usize, value: T) -> Result<(), VectorModificationError>
    where
        T: Clone,
    {
        self.resize_with(new_len, || value.clone())
    }

    /// Fill the remaining space of the vector with value.
    fn resize_with<F: FnMut() -> T>(
        &mut self,
        new_len: usize,
        mut f: F,
    ) -> Result<(), VectorModificationError> {
        let capacity = self.capacity();
        if capacity < new_len {
            let origin = format!("Vector::<{}>::resize_with()", core::any::type_name::<T>());

            fail!(from origin, with VectorModificationError::InsertWouldExceedCapacity,
                "Failed to resize vector to {} since it would exceed the vectors capacity of {}.",
                new_len, self.capacity());
        }

        if new_len < self.len() {
            self.truncate(new_len);
        } else {
            let len = self.len();
            let data = unsafe { self.data_mut() };
            for item in data.iter_mut().take(new_len).skip(len) {
                item.write(f());
            }

            unsafe { self.set_len(new_len as u64) };
        }

        Ok(())
    }

    /// Truncates the vector to `len` and drops all elements right of `len`
    /// in reverse order.
    fn truncate(&mut self, new_len: usize) {
        let len = self.len();
        if len <= new_len {
            return;
        }

        let data = unsafe { self.data_mut() };
        for idx in (new_len..len).rev() {
            unsafe { data[idx].assume_init_drop() };
        }

        unsafe { self.set_len(new_len as u64) };
    }
}
