// Copyright (c) 2025 Contributors to the Eclipse Foundation
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

//! Contains vector variations that are similar to [`std::vec::Vec`].
//!
//!  * [`Vec`](crate::vec::Vec), run-time fixed-size vector that is not shared-memory compatible
//!     since the memory resides in the heap.
//!  * [`RelocatableVec`](crate::vec::RelocatableVec), run-time fixed size vector that uses by default heap memory.
//!
//! # Expert Examples
//!
//! ## Create [`RelocatableVec`](crate::vec::RelocatableVec) inside constructs which provides memory
//!
//! ```
//! use iceoryx2_bb_container::vec::RelocatableVec;
//! use iceoryx2_bb_elementary::math::align_to;
//! use iceoryx2_bb_elementary::bump_allocator::BumpAllocator;
//! use iceoryx2_bb_elementary_traits::relocatable_container::RelocatableContainer;
//! use core::mem::MaybeUninit;
//!
//! const VEC_CAPACITY:usize = 12;
//! struct MyConstruct {
//!     vec: RelocatableVec<u128>,
//!     vec_memory: [MaybeUninit<u128>; VEC_CAPACITY],
//! }
//!
//! impl MyConstruct {
//!     pub fn new() -> Self {
//!         let mut new_self = Self {
//!             vec: unsafe { RelocatableVec::new_uninit(VEC_CAPACITY) },
//!             vec_memory: core::array::from_fn(|_| MaybeUninit::uninit()),
//!         };
//!
//!         let allocator = BumpAllocator::new(new_self.vec_memory.as_mut_ptr().cast());
//!         unsafe {
//!             new_self.vec.init(&allocator).expect("Enough memory provided.")
//!         };
//!         new_self
//!     }
//! }
//! ```
//!
//! ## Create [`RelocatableVec`](crate::vec::RelocatableVec) with allocator
//!
//! ```
//! use iceoryx2_bb_container::vec::RelocatableVec;
//! use iceoryx2_bb_elementary::bump_allocator::BumpAllocator;
//! use iceoryx2_bb_elementary_traits::relocatable_container::RelocatableContainer;
//! use core::ptr::NonNull;
//!
//! const VEC_CAPACITY:usize = 12;
//! const MEM_SIZE: usize = RelocatableVec::<u128>::const_memory_size(VEC_CAPACITY);
//! let mut memory = [0u8; MEM_SIZE];
//!
//! let bump_allocator = BumpAllocator::new(memory.as_mut_ptr());
//!
//! let mut vec = unsafe { RelocatableVec::<u128>::new_uninit(VEC_CAPACITY) };
//! unsafe { vec.init(&bump_allocator).expect("vec init failed") };
//! ```

use core::{
    alloc::Layout,
    fmt::Debug,
    marker::PhantomData,
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
};

use iceoryx2_bb_elementary::relocatable_ptr::GenericRelocatablePointer;
use iceoryx2_bb_elementary_traits::{
    generic_pointer::GenericPointer, owning_pointer::GenericOwningPointer,
    owning_pointer::OwningPointer, pointer_trait::PointerTrait,
    relocatable_container::RelocatableContainer, zero_copy_send::ZeroCopySend,
};

use iceoryx2_bb_elementary::{math::unaligned_mem_size, relocatable_ptr::RelocatablePointer};

use iceoryx2_bb_log::{fail, fatal_panic};

pub(crate) type Vec<T> = MetaVec<T, GenericOwningPointer>;

pub(crate) type RelocatableVec<T> = MetaVec<T, GenericRelocatablePointer>;

#[doc(hidden)]
#[repr(C)]
pub struct MetaVec<T, Ptr: GenericPointer> {
    data_ptr: Ptr::Type<MaybeUninit<T>>,
    capacity: usize,
    len: usize,
    _phantom_data: PhantomData<T>,
}

impl<T: Debug, Ptr: GenericPointer> Debug for MetaVec<T, Ptr> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "MetaVec<{}, {}> {{ capacity: {}, len: {}, is_initialized: {}, content: [ ",
            core::any::type_name::<T>(),
            core::any::type_name::<Ptr>(),
            self.capacity,
            self.len,
            self.data_ptr.is_initialized(),
        )?;

        if self.len > 0 {
            write!(f, "{:?}", self[0])?;
        }

        for n in 1..self.len {
            write!(f, ", {:?}", self[n])?;
        }

        write!(f, " ] }}")
    }
}

unsafe impl<T: Send, Ptr: GenericPointer> Send for MetaVec<T, Ptr> {}

impl<T, Ptr: GenericPointer> Drop for MetaVec<T, Ptr> {
    fn drop(&mut self) {
        if self.data_ptr.is_initialized() {
            unsafe { self.clear_impl() };
        }
    }
}

impl<T> RelocatableContainer for RelocatableVec<T> {
    unsafe fn new_uninit(capacity: usize) -> Self {
        Self {
            data_ptr: RelocatablePointer::new_uninit(),
            capacity,
            len: 0,
            _phantom_data: PhantomData,
        }
    }

    unsafe fn init<Allocator: iceoryx2_bb_elementary_traits::allocator::BaseAllocator>(
        &mut self,
        allocator: &Allocator,
    ) -> Result<(), iceoryx2_bb_elementary_traits::allocator::AllocationError> {
        if self.data_ptr.is_initialized() {
            fatal_panic!(from "Vec::init()", "Memory already initialized, Initializing it twice may lead to undefined behavior.");
        }

        self.data_ptr.init(fail!(from "Vec::init", when allocator
             .allocate(Layout::from_size_align_unchecked(
                 core::mem::size_of::<T>() * self.capacity,
                 core::mem::align_of::<T>(),
             )), "Failed to initialize vec since the allocation of the data memory failed."
        ));

        Ok(())
    }

    fn memory_size(capacity: usize) -> usize {
        Self::const_memory_size(capacity)
    }
}

impl<T, Ptr: GenericPointer> Deref for MetaVec<T, Ptr> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.verify_init("deref()");
        unsafe { self.as_slice_impl() }
    }
}

impl<T, Ptr: GenericPointer> DerefMut for MetaVec<T, Ptr> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.verify_init("deref_mut()");
        unsafe { self.as_mut_slice_impl() }
    }
}

impl<T: PartialEq, Ptr: GenericPointer> PartialEq for MetaVec<T, Ptr> {
    fn eq(&self, other: &Self) -> bool {
        if other.len() != self.len() {
            return false;
        }

        for i in 0..self.len() {
            if other[i] != self[i] {
                return false;
            }
        }

        true
    }
}

impl<T: Eq, Ptr: GenericPointer> Eq for MetaVec<T, Ptr> {}

impl<T, Ptr: GenericPointer> MetaVec<T, Ptr> {
    #[inline(always)]
    fn verify_init(&self, source: &str) {
        debug_assert!(
                self.data_ptr.is_initialized(),
                "From: MetaVec<{}>::{}, Undefined behavior - the object was not initialized with 'init' before.",
                core::any::type_name::<T>(), source
            );
    }

    /// Returns the capacity of the vector
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Returns the number of elements stored inside the vector
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns true if the vector is full, otherwise false
    pub fn is_full(&self) -> bool {
        self.len == self.capacity
    }

    pub(crate) unsafe fn push_impl(&mut self, value: T) -> bool {
        if self.is_full() {
            return false;
        }

        self.verify_init("push()");
        self.push_unchecked(value);
        true
    }

    fn push_unchecked(&mut self, value: T) {
        unsafe {
            self.data_ptr
                .as_mut_ptr()
                .add(self.len)
                .write(MaybeUninit::new(value))
        };

        self.len += 1;
    }

    unsafe fn clear_impl(&mut self) {
        for _ in 0..self.len {
            self.pop_unchecked();
        }
    }

    fn pop_unchecked(&mut self) -> T {
        let value = core::mem::replace(
            unsafe { &mut *self.data_ptr.as_mut_ptr().offset(self.len as isize - 1) },
            MaybeUninit::uninit(),
        );
        self.len -= 1;

        unsafe { value.assume_init() }
    }

    unsafe fn as_slice_impl(&self) -> &[T] {
        unsafe { core::slice::from_raw_parts(self.data_ptr.as_ptr().cast(), self.len) }
    }

    unsafe fn as_mut_slice_impl(&mut self) -> &mut [T] {
        unsafe { core::slice::from_raw_parts_mut(self.data_ptr.as_mut_ptr().cast(), self.len) }
    }
}

impl<T> Vec<T> {
    /// Creates a new [`Vec`] with the provided capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            data_ptr: OwningPointer::<MaybeUninit<T>>::new_with_alloc(capacity),
            capacity,
            len: 0,
            _phantom_data: PhantomData,
        }
    }
}

unsafe impl<T: ZeroCopySend> ZeroCopySend for RelocatableVec<T> {}

impl<T> RelocatableVec<T> {
    pub const fn const_memory_size(capacity: usize) -> usize {
        unaligned_mem_size::<T>(capacity)
    }
}
