// Copyright (c) 2024 Contributors to the Eclipse Foundation
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

use alloc::alloc::{alloc, dealloc};
use core::alloc::Layout;

/// Allocates uninitialized memory on the heap. When it goes out of scope the memory is released.
/// The user has to ensure that the memory is initialized.
pub struct RawMemory<T> {
    memory: *mut T,
}

impl<T> Drop for RawMemory<T> {
    fn drop(&mut self) {
        unsafe { dealloc(self.memory.cast(), Layout::new::<T>()) }
    }
}

impl<T> RawMemory<T> {
    /// Allocates memory to fit in a T and fills the memory with values provided by the fill
    /// callback.
    pub fn new<F: FnMut(usize) -> u8>(mut fill: F) -> Self {
        let layout = Layout::new::<T>();
        let memory = unsafe { alloc(layout) };

        for i in 0..layout.size() {
            unsafe { memory.add(i).write(fill(i)) }
        }

        Self {
            memory: memory.cast(),
        }
    }

    /// Allocates zeroed memory to fit in a T.
    pub fn new_zeroed() -> Self {
        Self::new(|_| 0u8)
    }

    /// Allocates memory filled with a given value to fit in a T.
    pub fn new_filled(value: u8) -> Self {
        Self::new(|_| value)
    }

    /// Returns a const pointer to T
    pub fn as_ptr(&self) -> *const T {
        self.memory
    }

    /// Returns a mutable pointer to T
    pub fn as_mut_ptr(&self) -> *mut T {
        self.memory
    }

    /// Returns a const reference to T
    ///
    /// # Safety
    ///
    /// * T must have been constructed manually before calling this function.
    pub unsafe fn assume_init(&self) -> &T {
        &*self.memory
    }

    /// Returns a mutable reference to T
    ///
    /// # Safety
    ///
    /// * T must have been constructed manually before calling this function.
    pub unsafe fn assume_init_mut(&mut self) -> &mut T {
        &mut *self.memory
    }

    /// Returns a slice to the underlying memory
    pub fn as_slice(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.memory.cast(), core::mem::size_of::<T>()) }
    }

    /// Returns a mutable slice to the underlying memory
    pub fn as_slice_mut(&mut self) -> &mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(self.memory.cast(), core::mem::size_of::<T>()) }
    }
}
