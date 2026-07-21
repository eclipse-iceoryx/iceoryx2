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
    ops::{Deref, DerefMut},
};

use flatbuffers::Allocator;
use iceoryx2_bb_elementary::relocatable_pointer::Pointer;
use iceoryx2_bb_elementary_traits::{
    allocator::{AllocationGrowError, ContentPlacement, ReallocGrow},
    pointer_family::PointerFamily,
};

pub struct ResizableMemory<P: PointerFamily, A: ReallocGrow<P>> {
    ptr: P::Pointer<u8>,
    allocator: A,
    current_layout: Layout,
}

impl<P: PointerFamily, A: ReallocGrow<P>> Deref for ResizableMemory<P, A> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        unsafe { core::slice::from_raw_parts(self.ptr.as_ptr(), self.len()) }
    }
}

impl<P: PointerFamily, A: ReallocGrow<P>> DerefMut for ResizableMemory<P, A> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { core::slice::from_raw_parts_mut(self.ptr.as_mut_ptr(), self.len()) }
    }
}

unsafe impl<P: PointerFamily, A: ReallocGrow<P>> Allocator for ResizableMemory<P, A> {
    type Error = AllocationGrowError;

    fn grow_downwards(&mut self) -> Result<(), Self::Error> {
        self.ptr = unsafe {
            self.allocator.grow(
                self.ptr.clone(),
                self.current_layout,
                self.current_layout,
                ContentPlacement::Back,
            )?
        };

        Ok(())
    }

    fn len(&self) -> usize {
        self.current_layout.size()
    }
}
