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

//! Provides an interface for low-level heap allocations.

pub use core::alloc::Layout;
pub use core::ptr::NonNull;
use iceoryx2_bb_elementary::math::*;
use iceoryx2_bb_elementary_traits::allocator::{
    AllocationError, AllocationGrowError, AllocationShrinkError,
};

use iceoryx2_pal_posix::posix::errno::Errno;
use iceoryx2_pal_posix::*;

impl From<MemoryError> for AllocationError {
    fn from(v: MemoryError) -> Self {
        match v {
            MemoryError::SizeIsZero => AllocationError::SizeIsZero,
            MemoryError::OutOfMemory => AllocationError::OutOfMemory,
            MemoryError::AlignmentFailure => AllocationError::AlignmentFailure,
            MemoryError::UnknownError(_) => AllocationError::InternalError,
        }
    }
}

impl From<MemoryError> for AllocationGrowError {
    fn from(v: MemoryError) -> Self {
        match v {
            MemoryError::SizeIsZero => AllocationGrowError::SizeIsZero,
            MemoryError::OutOfMemory => AllocationGrowError::OutOfMemory,
            MemoryError::AlignmentFailure => AllocationGrowError::AlignmentFailure,
            MemoryError::UnknownError(_) => AllocationGrowError::InternalError,
        }
    }
}

impl From<MemoryError> for AllocationShrinkError {
    fn from(v: MemoryError) -> Self {
        match v {
            MemoryError::SizeIsZero => AllocationShrinkError::SizeIsZero,
            MemoryError::AlignmentFailure => AllocationShrinkError::AlignmentFailure,
            _ => AllocationShrinkError::InternalError,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum MemoryError {
    OutOfMemory,
    SizeIsZero,
    AlignmentFailure,
    UnknownError(i32),
}

/// Performs heap allocations. Basis for a heap allocator.
pub mod heap {
    use crate::handle_errno;
    use iceoryx2_bb_log::fail;

    use super::*;

    use super::MemoryError;
    const MEMORY_START_STORAGE_SPACE: usize = core::mem::size_of::<usize>();

    fn setup_and_align(
        msg: &str,
        addr: usize,
        layout: Layout,
    ) -> Result<NonNull<[u8]>, MemoryError> {
        if addr == 0 {
            handle_errno!(MemoryError, from "heap::setup_and_align",
                Errno::ENOMEM => (OutOfMemory, "{} a size of {} and an alignment of {} due to insufficient memory.", msg, layout.size(), layout.align() ),
                v => (UnknownError(v as i32), "{} a size of {} and an alignment of {} since an unknown error occurred ({}).", msg, layout.size(), layout.align(), v)

            );
        }

        let aligned_start = align(addr + MEMORY_START_STORAGE_SPACE, layout.align());
        unsafe { *(aligned_start as *mut usize).offset(-1) = addr };

        Ok(NonNull::new(unsafe {
            core::slice::from_raw_parts_mut(aligned_start as *mut u8, layout.size())
        })
        .unwrap())
    }

    fn check_precondition(msg: &str, layout: Layout) -> Result<(), MemoryError> {
        if layout.size() == 0 {
            fail!(from "heap::check_precondition", with MemoryError::SizeIsZero,
                "{} a size of {} and an alignment of {} since the size is 0.",
                msg,
                layout.size(),
                layout.align()
            );
        }
        Ok(())
    }

    unsafe fn extract_address(addr: usize) -> usize {
        let memory = addr as *const usize;
        *memory.offset(-1)
    }

    /// Allocates an aligned piece of memory on the heap with a size specified in layout.
    pub fn allocate(layout: Layout) -> Result<NonNull<[u8]>, MemoryError> {
        let msg = "Unable to allocate heap memory with";
        check_precondition(msg, layout)?;

        let alignment_adjustment_buffer = layout.align() - 1;
        let unaligned_start = unsafe {
            posix::malloc(layout.size() + alignment_adjustment_buffer + MEMORY_START_STORAGE_SPACE)
        } as usize;

        setup_and_align(msg, unaligned_start, layout)
    }

    /// Allocates an aligned and zeroed piece of memory on the heap with a size specified in layout.
    pub fn allocate_zeroed(layout: Layout) -> Result<NonNull<[u8]>, MemoryError> {
        let msg = "Unable to allocate zeroed heap memory with";
        check_precondition(msg, layout)?;

        let alignment_adjustment_buffer = layout.align() - 1;
        let unaligned_start = unsafe {
            posix::calloc(
                layout.size() + alignment_adjustment_buffer + MEMORY_START_STORAGE_SPACE,
                1,
            )
        } as usize;

        setup_and_align(msg, unaligned_start, layout)
    }

    /// Resizes the memory to the new_layout. If the space after the current block does not
    /// suffice resize returns a new pointer and copies to content into the new block. The
    /// old address does not have to be free'd in this case.
    ///
    /// # Safety
    ///
    ///  * `ptr` must be allocated before with [`heap::allocate()`] or [`heap::allocate_zeroed()`]
    ///  * `old_layout` must be the same layout it was either acquired with in [`heap::allocate()`]
    ///    or [`heap::allocate_zeroed()`] or when it was resized the current layout
    pub unsafe fn resize(
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, MemoryError> {
        let msg = "Unable to resize heap memory to";
        check_precondition(msg, new_layout)?;

        if old_layout.align() < new_layout.align() {
            fail!(from "heap::resize", with MemoryError::AlignmentFailure,
                "{} {} since the new layouts alignment was increased from {} to {}.",
                msg,
                new_layout.size(),
                old_layout.align(),
                new_layout.align()
            );
        }

        let alignment_adjustment_buffer = new_layout.align() - 1;

        let current_unaligned_start = extract_address(ptr.as_ptr() as usize);
        let new_unaligned_start = posix::realloc(
            current_unaligned_start as *mut posix::void,
            new_layout.size() + alignment_adjustment_buffer + MEMORY_START_STORAGE_SPACE,
        ) as usize;

        setup_and_align(msg, new_unaligned_start, new_layout)
    }

    /// Deallocates a previously allocated piece of memory.
    ///
    /// # Safety
    ///
    ///  * `ptr` must be allocated before with [`heap::allocate()`] or [`heap::allocate_zeroed()`]
    ///  * `old_layout` must be the same layout it was either acquired with in [`heap::allocate()`]
    ///    or [`heap::allocate_zeroed()`] or when it was resized the current layout
    ///
    pub unsafe fn deallocate(ptr: NonNull<u8>, _layout: Layout) {
        posix::free(extract_address(ptr.as_ptr() as usize) as *mut posix::void)
    }
}
