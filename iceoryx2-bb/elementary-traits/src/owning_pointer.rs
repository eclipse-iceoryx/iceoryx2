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

//! Represents a normal non-null pointer. It was introduced to distinguish normal pointers from
//! `iceoryx2_bb_elementary::relocatable_ptr::RelocatablePointer`. It implements the [`PointerTrait`].

use alloc::alloc::{alloc, dealloc};
use core::alloc::Layout;
use core::fmt::Debug;

use crate::generic_pointer::GenericPointer;
use crate::pointer_trait::PointerTrait;

#[derive(Debug)]
pub struct GenericOwningPointer;

/// Representation of a pointer which owns its memory.
#[repr(C)]
#[derive(Debug)]
pub struct OwningPointer<T> {
    ptr: *mut T,
    layout: Layout,
}

impl<T> OwningPointer<T> {
    /// Allocates memory for T and number_of_elements. If the number_of_elements is zero it still
    /// allocates memory for one element.
    pub fn new_with_alloc(mut number_of_elements: usize) -> OwningPointer<T> {
        if number_of_elements == 0 {
            number_of_elements = 1;
        }

        let layout = unsafe {
            Layout::from_size_align_unchecked(
                core::mem::size_of::<T>() * number_of_elements,
                core::mem::align_of::<T>(),
            )
        };

        Self {
            ptr: unsafe { alloc(layout) as *mut T },
            layout,
        }
    }
}

impl<T> Drop for OwningPointer<T> {
    fn drop(&mut self) {
        unsafe { dealloc(self.ptr as *mut u8, self.layout) }
    }
}

impl<T> PointerTrait<T> for OwningPointer<T> {
    unsafe fn as_ptr(&self) -> *const T {
        self.ptr as *const T
    }

    unsafe fn as_mut_ptr(&mut self) -> *mut T {
        self.ptr
    }
}

impl GenericPointer for GenericOwningPointer {
    type Type<T: Debug> = OwningPointer<T>;
}
