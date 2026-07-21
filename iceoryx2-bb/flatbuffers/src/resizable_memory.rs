// Copyright (c) 2026 Contributors to the Eclipse Foundation
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

use core::{
    alloc::Layout,
    fmt::Debug,
    ops::{Deref, DerefMut},
};

use flatbuffers::Allocator;
use iceoryx2_bb_elementary::{
    allocation_strategy::AllocationStrategy, enum_gen, relocatable_pointer::Pointer,
};
use iceoryx2_bb_elementary_traits::allocator::{
    AllocationGrowError, ContentPlacement, ReallocGrow,
};
use iceoryx2_log::fail;

enum_gen! {
    ResizableMemoryError
  entry:
    ReservedHeaderLenExceedsInitialSize
}

#[derive(Debug)]
pub struct ResizableMemoryBuilder<P: Pointer<u8>> {
    ptr: P,
    strategy: AllocationStrategy,
    initial_layout: Layout,
    reserved_header_len: usize,
}

impl<P: Pointer<u8>> ResizableMemoryBuilder<P> {
    pub fn new(ptr: P) -> Self {
        Self {
            ptr,
            strategy: AllocationStrategy::Static,
            initial_layout: Layout::new::<u8>(),
            reserved_header_len: 0,
        }
    }

    pub fn allocation_strategy(mut self, value: AllocationStrategy) -> Self {
        self.strategy = value;
        self
    }

    pub fn initial_layout(mut self, layout: Layout) -> Self {
        self.initial_layout = layout;
        self
    }

    pub fn reserved_header_len(mut self, value: usize) -> Self {
        self.reserved_header_len = value;
        self
    }

    pub fn create<A: ReallocGrow<P>>(
        self,
        allocatable: A,
    ) -> Result<ResizableMemory<P, A>, ResizableMemoryError> {
        let msg = "Unable to create ResizableMemory";
        if self.reserved_header_len >= self.initial_layout.size() {
            fail!(from self, with ResizableMemoryError::ReservedHeaderLenExceedsInitialSize,
                "{msg} since the reserved header len {} exceeds the initial memory size {}.",
                self.reserved_header_len, self.initial_layout.size());
        }

        Ok(ResizableMemory {
            ptr: self.ptr,
            strategy: self.strategy,
            allocatable,
            current_layout: self.initial_layout,
            reserved_header_len: self.reserved_header_len,
        })
    }
}

pub struct ResizableMemory<P: Pointer<u8>, A: ReallocGrow<P>> {
    ptr: P,
    strategy: AllocationStrategy,
    allocatable: A,
    current_layout: Layout,
    reserved_header_len: usize,
}

impl<P: Pointer<u8>, A: ReallocGrow<P>> Debug for ResizableMemory<P, A> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "ResizableMemory<{}, {}> {{ ptr: {:?}, strategy: {:?}, current_layout: {:?}, reserved_header_len: {:?} }}",
            core::any::type_name::<P>(),
            core::any::type_name::<A>(),
            self.ptr,
            self.strategy,
            self.current_layout,
            self.reserved_header_len
        )
    }
}

impl<P: Pointer<u8>, A: ReallocGrow<P>> Deref for ResizableMemory<P, A> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        unsafe { core::slice::from_raw_parts(self.ptr.as_ptr(), self.len()) }
    }
}

impl<P: Pointer<u8>, A: ReallocGrow<P>> DerefMut for ResizableMemory<P, A> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { core::slice::from_raw_parts_mut(self.ptr.as_mut_ptr(), self.len()) }
    }
}

unsafe impl<P: Pointer<u8>, A: ReallocGrow<P>> Allocator for ResizableMemory<P, A> {
    type Error = AllocationGrowError;

    fn grow_downwards(&mut self) -> Result<(), Self::Error> {
        let msg = "Unable to grow memory";
        let new_layout = match self.strategy {
            AllocationStrategy::Static => {
                fail!(from self, with AllocationGrowError::OutOfMemory,
                    "{msg} since the allocation strategy is static.");
            }
            AllocationStrategy::PowerOfTwo => unsafe {
                let size = (self.current_layout.size() + 1).next_power_of_two();
                Layout::from_size_align_unchecked(size, self.current_layout.align())
            },
            AllocationStrategy::BestFit => {
                let size =
                    (self.current_layout.size() + 1).next_multiple_of(self.current_layout.align());
                unsafe { Layout::from_size_align_unchecked(size, self.current_layout.align()) }
            }
        };

        self.ptr = unsafe {
            self.allocatable.grow(
                self.ptr.clone(),
                self.current_layout,
                new_layout,
                ContentPlacement::Back,
            )?
        };

        let offset = new_layout.size() - self.current_layout.size();
        unsafe {
            core::ptr::copy(
                self.ptr.as_ptr().add(offset),
                self.ptr.as_mut_ptr(),
                self.reserved_header_len,
            )
        };

        self.current_layout = new_layout;
        Ok(())
    }

    fn len(&self) -> usize {
        self.current_layout.size() - self.reserved_header_len
    }
}
