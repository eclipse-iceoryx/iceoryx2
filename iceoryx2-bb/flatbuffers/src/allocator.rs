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
    fmt::Display,
    ops::{Deref, DerefMut},
};

use flatbuffers::Allocator;
use iceoryx2_bb_elementary::relocatable_ptr::PointerTrait;

#[derive(Clone, Copy, Eq, PartialEq, Debug, Default)]
pub enum ContentPlacement {
    #[default]
    Front,
    Back,
}

pub trait DataSegment<P: PointerTrait<u8>> {
    fn grow(
        &self,
        ptr: &P,
        old_layout: Layout,
        new_layout: Layout,
        placement: ContentPlacement,
    ) -> Result<P, ResizableMemoryError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResizableMemoryError {}

impl Display for ResizableMemoryError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ResizableMemoryError::{:?}", self)
    }
}

impl core::error::Error for ResizableMemoryError {}

pub struct ResizableMemory<P: PointerTrait<u8>, D: DataSegment<P>> {
    ptr: P,
    data_segment: D,
    current_layout: Layout,
}

impl<P: PointerTrait<u8>, D: DataSegment<P>> Deref for ResizableMemory<P, D> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        unsafe { core::slice::from_raw_parts(self.ptr.as_ptr(), self.len()) }
    }
}

impl<P: PointerTrait<u8>, D: DataSegment<P>> DerefMut for ResizableMemory<P, D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { core::slice::from_raw_parts_mut(self.ptr.as_mut_ptr(), self.len()) }
    }
}

unsafe impl<P: PointerTrait<u8>, D: DataSegment<P>> Allocator for ResizableMemory<P, D> {
    type Error = ResizableMemoryError;

    fn grow_downwards(&mut self) -> Result<(), Self::Error> {
        self.ptr = self.data_segment.grow(
            &self.ptr,
            self.current_layout,
            self.current_layout,
            ContentPlacement::Back,
        )?;

        Ok(())
    }

    fn len(&self) -> usize {
        self.current_layout.size()
    }
}
