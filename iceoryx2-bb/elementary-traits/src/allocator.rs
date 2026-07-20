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

//! Contains the traits [`BaseAllocator`] which contains the most basic functionality an allocator
//! requires and [`Allocator`] with more advanced allocation features.

pub use core::{alloc::Layout, ptr::NonNull};

use crate::generic_pointer::PointerFamily;
use crate::pointer::Pointer;

/// Failures caused by [`BaseAllocator::allocate()`] or [`BaseAllocator::allocate_zeroed()`].
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum AllocationError {
    SizeIsZero,
    SizeTooLarge,
    AlignmentFailure,
    OutOfMemory,
    InternalError,
}

impl core::fmt::Display for AllocationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "AllocationError::{self:?}")
    }
}

impl core::error::Error for AllocationError {}

/// Failures caused by [`Allocator::grow()`] or [`Allocator::grow_zeroed()`].
#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
pub enum AllocationGrowError {
    GrowWouldShrink,
    SizeIsZero,
    OutOfMemory,
    AlignmentFailure,
    InternalError,
}

impl core::fmt::Display for AllocationGrowError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "AllocationGrowError::{self:?}")
    }
}

impl core::error::Error for AllocationGrowError {}

/// Failures caused by [`Allocator::shrink()`].
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum AllocationShrinkError {
    ShrinkWouldGrow,
    SizeIsZero,
    AlignmentFailure,
    InternalError,
}

impl core::fmt::Display for AllocationShrinkError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "AllocationShrinkError::{self:?}")
    }
}

impl core::error::Error for AllocationShrinkError {}

/// The most minimalistic requirement for an allocator
pub trait BaseAllocator<P: PointerFamily> {
    /// Allocates a memory chunk with the properties provided in layout and either
    /// returns a slice or an allocation error on failure.
    fn allocate(&self, layout: Layout) -> Result<P::Pointer<u8>, AllocationError>;

    /// Allocates a memory chunk with the properties provided in layout and zeroes the memory
    /// On success it returns a slice or an allocation error on failure.
    fn allocate_zeroed(
        &self,
        layout: core::alloc::Layout,
    ) -> Result<P::Pointer<u8>, AllocationError> {
        let mut ptr = self.allocate(layout)?;
        let raw_ptr = ptr.as_mut_ptr();
        unsafe { core::ptr::write_bytes(raw_ptr, 0, layout.size()) };
        Ok(ptr)
    }

    /// Releases an previously allocated chunk of memory.
    ///
    /// # Safety
    ///
    ///  * `ptr` must be allocated previously with [`BaseAllocator::allocate()`] or
    ///    [`BaseAllocator::allocate_zeroed()`]
    ///  * `layout` must have the same value as in the allocation or, when the memory was
    ///    resized, the same value as it was resized to
    ///
    unsafe fn deallocate(&self, ptr: P::Pointer<u8>, layout: Layout);
}

/// Allocator with grow and shrink features.
pub trait Allocator<P: PointerFamily>: BaseAllocator<P> {
    /// Increases the size of an previously allocated chunk of memory or allocates a new chunk
    /// with the provided properties.
    /// It returns a failure when the size decreases.
    ///
    /// # Safety
    ///
    ///  * `ptr` must be allocated previously with [`BaseAllocator::allocate()`] or
    ///    [`BaseAllocator::allocate_zeroed()`]
    ///  * `old_layout` must have the same value as in the allocation or, when the memory was
    ///    resized, the same value as it was resized to
    ///
    unsafe fn grow(
        &self,
        ptr: P::Pointer<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<P::Pointer<u8>, AllocationGrowError>;

    /// Increases the size of an previously allocated chunk of memory or allocates a new chunk
    /// with the provided properties. If the chunk can be resized only the difference in size
    /// will be zeroed.
    /// It returns a failure when the size decreases.
    ///
    /// # Safety
    ///
    ///  * `ptr` must be allocated previously with [`BaseAllocator::allocate()`] or
    ///    [`BaseAllocator::allocate_zeroed()`]
    ///  * `old_layout` must have the same value as in the allocation or, when the memory was
    ///    resized, the same value as it was resized to
    ///
    unsafe fn grow_zeroed(
        &self,
        ptr: P::Pointer<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<P::Pointer<u8>, AllocationGrowError> {
        let mut ptr = unsafe { self.grow(ptr, old_layout, new_layout)? };
        let raw_ptr = ptr.as_mut_ptr();
        unsafe {
            core::ptr::write_bytes(
                raw_ptr.add(old_layout.size()),
                0,
                new_layout.size() - old_layout.size(),
            )
        };
        Ok(ptr)
    }

    /// Decreases the size of an previously allocated chunk of memory. If the size increases it
    /// fails.
    ///
    /// # Safety
    ///
    ///  * `ptr` must be allocated previously with [`BaseAllocator::allocate()`] or
    ///    [`BaseAllocator::allocate_zeroed()`]
    ///  * `old_layout` must have the same value as in the allocation or, when the memory was
    ///    resized, the same value as it was resized to
    ///
    unsafe fn shrink(
        &self,
        ptr: P::Pointer<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<P::Pointer<u8>, AllocationShrinkError>;
}
