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

//! Describes a container which can shared between processes.

use crate::{allocator::AllocationError, allocator::BaseAllocator};

/// Describes a container which can shared between processes. Since the shared memory is often
/// mapped at a different virtual memory position the underlying constructs must be relocatable in
/// the sense that they should not rely on absolut memory positions.
pub trait RelocatableContainer {
    /// Creates a new uninitialized RelocatableContainer. Before the container can be used the method
    /// [`RelocatableContainer::init()`] must be called.
    ///
    /// # Safety
    ///
    ///  * Before the container can be used [`RelocatableContainer::init()`] must be called exactly
    ///    once.
    ///
    unsafe fn new_uninit(capacity: usize) -> Self;

    /// Initializes an uninitialized RelocatableContainer. It allocates the required memory from
    /// the provided allocator. The allocator must have at least
    /// [`RelocatableContainer::memory_size()`] bytes available.
    ///
    /// # Safety
    ///
    ///  * Must be called exactly once before any other method is called.
    ///  * Shall be only used when the [`RelocatableContainer`] was created with
    ///    [`RelocatableContainer::new_uninit()`]
    ///
    unsafe fn init<T: BaseAllocator>(&mut self, allocator: &T) -> Result<(), AllocationError>;

    /// Returns the amount of additional memory the object requires from the
    /// [`BaseAllocator`] in the [`RelocatableContainer::init()`] call. The returned value
    /// considers the alignment overhead. When implementing this, please use
    /// `iceoryx2_bb_elementary::math::unaligned_mem_size()`.
    /// The whole memory consumption is
    /// `core::mem::size_of::<RelocatableContainer>() + RelocatableContainer::memory_size()`.
    fn memory_size(capacity: usize) -> usize;
}
