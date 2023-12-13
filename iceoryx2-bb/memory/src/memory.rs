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

//! A fixed size piece of uninitialized memory. It comes with an allocator so that it can be used
//! in combination with data structures than can use a custom allocator to implement
//! simplistic fixed size versions of them.
//!
//! It is pinned and therefore not movable.
//!
//! # Example
//!
//! ## Stack
//!
//! ```
//! use pin_init::init_stack;
//! use std::alloc::Layout;
//! use iceoryx2_bb_memory::memory::Memory;
//! use iceoryx2_bb_memory::bump_allocator::*;
//!
//! const MEMORY_SIZE: usize = 1024;
//!
//! init_stack!(
//!     memory = Memory::<MEMORY_SIZE, BumpAllocator>::new_filled(0xff)
//! );
//! let memory = memory.unwrap();
//!
//! let chunk = memory.allocate(unsafe{ Layout::from_size_align_unchecked(16, 8) });
//! let another_chunk = memory.allocate(Layout::new::<u64>());
//! ```
//!
//! ## Heap
//! ```
//! use pin_init::PtrPinWith;
//! use pin_init::init_stack;
//! use std::alloc::Layout;
//! use iceoryx2_bb_memory::memory::Memory;
//! use iceoryx2_bb_memory::bump_allocator::*;
//!
//! const MEMORY_SIZE: usize = 1024;
//!
//! let memory = Box::pin_with(Memory::<MEMORY_SIZE, BumpAllocator>::new()).unwrap();
//!
//! let chunk = memory.allocate(Layout::new::<u64>());
//! ```

use std::{mem::MaybeUninit, ptr::NonNull};

pub use iceoryx2_bb_elementary::allocator::BaseAllocator;
use iceoryx2_bb_elementary::allocator::{AllocationError, DeallocationError};

use crate::bump_allocator::BumpAllocator;
use pin_init::*;

#[derive(Debug)]
struct MemoryData<const SIZE: usize, Allocator: BaseAllocator> {
    memory: MaybeUninit<[u8; SIZE]>,
    allocator: Allocator,
}

impl<const SIZE: usize> MemoryData<SIZE, BumpAllocator> {
    fn new() -> impl Init<Self, ()> {
        init_from_closure(
            |mut this: PinUninit<'_, Self>| -> InitResult<'_, Self, ()> {
                let v = this.get_mut().as_mut_ptr();

                unsafe {
                    let memory_ptr = std::ptr::addr_of_mut!((*v).memory);
                    let allocator_ptr = std::ptr::addr_of_mut!((*v).allocator);
                    allocator_ptr.write(BumpAllocator::new(
                        NonNull::new_unchecked(memory_ptr as *mut u8),
                        SIZE,
                    ));

                    Ok(this.init_ok())
                }
            },
        )
    }

    fn new_filled(value: u8) -> impl Init<Self, ()> {
        init_from_closure(
            move |mut this: PinUninit<'_, Self>| -> InitResult<'_, Self, ()> {
                let v = this.get_mut().as_mut_ptr();

                unsafe {
                    let memory_ptr = std::ptr::addr_of_mut!((*v).memory);
                    memory_ptr.write(MaybeUninit::new([value; SIZE]));
                    let allocator_ptr = std::ptr::addr_of_mut!((*v).allocator);
                    allocator_ptr.write(BumpAllocator::new(
                        NonNull::new_unchecked(memory_ptr as *mut u8),
                        SIZE,
                    ));

                    Ok(this.init_ok())
                }
            },
        )
    }
}

#[derive(Debug)]
#[pin_init]
pub struct Memory<const SIZE: usize, Allocator: BaseAllocator> {
    #[pin]
    data: MemoryData<SIZE, Allocator>,
}

impl<const SIZE: usize> Memory<SIZE, BumpAllocator> {
    /// Creates a fixed size [`Memory`] object.
    pub fn new() -> impl Init<Self, ()> {
        init_pin!(Self {
            data: MemoryData::new()
        })
    }

    /// Creates a fixed size [`Memory`] object which is filled with value.
    pub fn new_filled(value: u8) -> impl Init<Self, ()> {
        init_pin!(Self {
            data: MemoryData::new_filled(value)
        })
    }
}

impl<const SIZE: usize, Allocator: BaseAllocator> Memory<SIZE, Allocator> {
    /// Returns a reference to the underlying allocator
    pub fn allocator(&self) -> &Allocator {
        &self.data.allocator
    }
}

impl<const SIZE: usize, Allocator: BaseAllocator> BaseAllocator for Memory<SIZE, Allocator> {
    /// Calls [`BaseAllocator::allocate()`] on the underlying allocator
    fn allocate(&self, layout: std::alloc::Layout) -> Result<NonNull<[u8]>, AllocationError> {
        self.data.allocator.allocate(layout)
    }

    /// Calls [`BaseAllocator::deallocate()`] on the underlying allocator
    unsafe fn deallocate(
        &self,
        ptr: NonNull<u8>,
        layout: std::alloc::Layout,
    ) -> Result<(), DeallocationError> {
        self.data.allocator.deallocate(ptr, layout)
    }
}
