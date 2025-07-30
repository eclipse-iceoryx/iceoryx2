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

//! Building block to handle inter process communication with multiple shared memory object. Every
//! process has mapped them to a different virtual memory location therefore pointer inside that
//! memory region should be distances starting from a fix point which maybe different in every
//! process.
//!
//! # Example
//!
//! ```
//! use iceoryx2_bb_elementary::relocatable_ptr::*;
//! use iceoryx2_bb_elementary::math::align_to;
//!
//! #[repr(C)]
//! pub struct Container {
//!     data_ptr: RelocatablePointer<u128>,
//!     capacity: usize,
//! }
//!
//! impl Container {
//!     pub fn new(capacity: usize, distance: isize) -> Self {
//!         Self {
//!             data_ptr: RelocatablePointer::new(distance),
//!             capacity
//!         }
//!     }
//!
//!     pub fn get_mut(&mut self, index: usize) -> &mut u128 {
//!         unsafe { &mut *self.data_ptr.as_mut_ptr() }
//!     }
//! }
//!
//! #[repr(C)]
//! pub struct FixedSizeContainer {
//!     base: Container,
//!     data: [u128; 128],
//! }
//!
//! impl FixedSizeContainer {
//!     pub fn new() -> Self {
//!         Self {
//!             base: Container::new(128,
//!                         // the data_ptr is the first member of container. The distance from
//!                         // the memory location of the RelocatablePointer `data_ptr` is
//!                         // therefore the size of `Container` aligned to the type `u128`
//!                         align_to::<u128>(core::mem::size_of::<Container>()) as isize),
//!             data: [0; 128]
//!         }
//!     }
//!
//!     pub fn get_mut(&mut self, index: usize) -> &mut u128 {
//!         self.base.get_mut(index)
//!     }
//! }
//! ```

use core::{fmt::Debug, marker::PhantomData, ptr::NonNull};
use iceoryx2_bb_elementary_traits::generic_pointer::GenericPointer;
pub use iceoryx2_bb_elementary_traits::pointer_trait::PointerTrait;
use iceoryx2_pal_concurrency_sync::iox_atomic::IoxAtomicIsize;

#[derive(Debug)]
pub struct GenericRelocatablePointer;

/// A [`RelocatablePointer`] stores only the distance from its memory starting position to the
/// memory location it is pointing to. When the [`RelocatablePointer`] is now shared between
/// processes its virtual memory starting position changes but the distance to the object it is
/// pointing to is the same.
///
/// **Important:**
///   1. Every construct which relies on a [`RelocatablePointer`] must be declared with
///      `[repr(C)]`. Otherwise different compilation units may have a different structural layout of
///      the data type which is shared between processes which leads to undefined behavior.
///   2. The construct which is using the [`RelocatablePointer`] and the pointee must be stored in
///      the same shared memory object. Pointing to a different shared memory segment most likely
///      leads to crashes since it can be mapped in a different order, at a different position and
///      the distance to the memory destination is off.
#[repr(C)]
#[derive(Debug)]
pub struct RelocatablePointer<T> {
    distance: IoxAtomicIsize,
    _phantom: PhantomData<T>,
}

impl<T> RelocatablePointer<T> {
    /// Creates a new [`RelocatablePointer`]. The distance is the relative distance to the memory
    /// destination starting from the memory location of this [`RelocatablePointer`].
    pub fn new(distance: isize) -> Self {
        Self {
            distance: IoxAtomicIsize::new(distance),
            _phantom: PhantomData,
        }
    }

    /// Creates a new uninitialized [`RelocatablePointer`].
    ///
    /// # Safety
    ///
    ///  * [`RelocatablePointer::init()`] must be called once before use
    ///
    pub unsafe fn new_uninit() -> Self {
        Self::new(0)
    }

    /// Initializes the [`RelocatablePointer`] by setting the distance to the memory destination
    /// by providing an absolut pointer to it. The distance can be calculated from the
    /// [`RelocatablePointer`] memory location and the absolut position of the destination.
    /// **Important:** The pointer must point into the same shared memory object.
    ///
    /// # Safety
    ///
    ///  * [`RelocatablePointer`] was created with [`RelocatablePointer::new_uninit()`]
    ///  * ptr has an alignment of [`core::mem::align_of<T>()`]
    ///  * ptr has a size which is an multiple of [`core::mem::size_of<T>()`]
    ///  * It must be called exactly once before using the [`RelocatablePointer`]
    ///
    pub unsafe fn init(&self, ptr: NonNull<[u8]>) {
        self.distance.store(
            (ptr.as_ptr() as *const u8) as isize - (self as *const Self) as isize,
            core::sync::atomic::Ordering::Relaxed,
        );
    }
}

impl<T> PointerTrait<T> for RelocatablePointer<T> {
    unsafe fn as_ptr(&self) -> *const T {
        ((self as *const Self) as isize + self.distance.load(core::sync::atomic::Ordering::Relaxed))
            as *const T
    }

    unsafe fn as_mut_ptr(&mut self) -> *mut T {
        self.as_ptr() as *mut T
    }

    fn is_initialized(&self) -> bool {
        self.distance.load(core::sync::atomic::Ordering::Relaxed) != 0
    }
}

impl GenericPointer for GenericRelocatablePointer {
    type Type<T: Debug> = RelocatablePointer<T>;
}
