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
    allocation_strategy::AllocationStrategy, relocatable_pointer::Pointer,
};
use iceoryx2_bb_elementary_traits::allocator::{
    AllocationGrowError, ContentPlacement, ReallocGrow,
};
use iceoryx2_log::fail;

pub struct ResizableMemory<P: Pointer<u8>, A: ReallocGrow<P>> {
    ptr: P,
    strategy: AllocationStrategy,
    allocator: A,
    current_layout: Layout,
}

impl<P: Pointer<u8>, A: ReallocGrow<P>> Debug for ResizableMemory<P, A> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "ResizableMemory<{}, {}> {{ ptr: {:?}, strategy: {:?}, current_layout: {:?} }}",
            core::any::type_name::<P>(),
            core::any::type_name::<A>(),
            self.ptr,
            self.strategy,
            self.current_layout
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
            self.allocator.grow(
                self.ptr.clone(),
                self.current_layout,
                new_layout,
                ContentPlacement::Back,
            )?
        };

        self.current_layout = new_layout;
        Ok(())
    }

    fn len(&self) -> usize {
        self.current_layout.size()
    }
}
