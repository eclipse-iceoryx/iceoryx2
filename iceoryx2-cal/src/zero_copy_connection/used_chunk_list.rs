// Copyright (c) 2024 Contributors to the Eclipse Foundation
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

use core::{alloc::Layout, sync::atomic::Ordering};
use iceoryx2_bb_elementary::{
    bump_allocator::BumpAllocator,
    relocatable_ptr::{PointerTrait, RelocatablePointer},
};
use iceoryx2_bb_elementary_traits::{
    owning_pointer::OwningPointer, relocatable_container::RelocatableContainer,
};
use iceoryx2_bb_log::{fail, fatal_panic};
use iceoryx2_pal_concurrency_sync::iox_atomic::IoxAtomicBool;

pub type UsedChunkList = details::UsedChunkList<OwningPointer<IoxAtomicBool>>;
pub type RelocatableUsedChunkList = details::UsedChunkList<RelocatablePointer<IoxAtomicBool>>;

pub mod details {
    use core::fmt::Debug;

    use iceoryx2_bb_elementary::math::unaligned_mem_size;
    use iceoryx2_bb_elementary_traits::owning_pointer::OwningPointer;

    use super::*;

    #[derive(Debug)]
    #[repr(C)]
    pub struct UsedChunkList<PointerType: PointerTrait<IoxAtomicBool>> {
        data_ptr: PointerType,
        capacity: usize,
        is_memory_initialized: IoxAtomicBool,
    }

    unsafe impl<PointerType: PointerTrait<IoxAtomicBool>> Send for UsedChunkList<PointerType> {}
    unsafe impl<PointerType: PointerTrait<IoxAtomicBool>> Sync for UsedChunkList<PointerType> {}

    impl UsedChunkList<OwningPointer<IoxAtomicBool>> {
        pub fn new(capacity: usize) -> Self {
            let mut data_ptr = OwningPointer::<IoxAtomicBool>::new_with_alloc(capacity);

            for i in 0..capacity {
                unsafe {
                    data_ptr
                        .as_mut_ptr()
                        .add(i)
                        .write(IoxAtomicBool::new(false))
                };
            }

            Self {
                data_ptr,
                capacity,
                is_memory_initialized: IoxAtomicBool::new(true),
            }
        }
    }

    impl RelocatableContainer for UsedChunkList<RelocatablePointer<IoxAtomicBool>> {
        unsafe fn new_uninit(capacity: usize) -> Self {
            Self {
                data_ptr: RelocatablePointer::new_uninit(),
                capacity,
                is_memory_initialized: IoxAtomicBool::new(false),
            }
        }

        unsafe fn init<T: iceoryx2_bb_elementary_traits::allocator::BaseAllocator>(
            &mut self,
            allocator: &T,
        ) -> Result<(), iceoryx2_bb_elementary_traits::allocator::AllocationError> {
            if self.is_memory_initialized.load(Ordering::Relaxed) {
                fatal_panic!(from self,
                "Memory already initialized. Initializing it twice may lead to undefined behavior.");
            }

            let memory = fail!(from self, when allocator
            .allocate(Layout::from_size_align_unchecked(
                    core::mem::size_of::<IoxAtomicBool>() * self.capacity,
                    core::mem::align_of::<IoxAtomicBool>())),
            "Failed to initialize since the allocation of the data memory failed.");

            self.data_ptr.init(memory);

            for i in 0..self.capacity {
                unsafe {
                    (self.data_ptr.as_ptr() as *mut IoxAtomicBool)
                        .add(i)
                        .write(IoxAtomicBool::new(false))
                };
            }

            // relaxed is sufficient since no relocatable container can be used
            // before init was called. Meaning, it is not allowed to send or share
            // the container with other threads when it is in an uninitialized state.
            self.is_memory_initialized.store(true, Ordering::Relaxed);

            Ok(())
        }

        fn memory_size(capacity: usize) -> usize {
            Self::const_memory_size(capacity)
        }
    }

    impl<PointerType: PointerTrait<IoxAtomicBool> + Debug> UsedChunkList<PointerType> {
        pub const fn const_memory_size(capacity: usize) -> usize {
            unaligned_mem_size::<IoxAtomicBool>(capacity)
        }

        pub fn capacity(&self) -> usize {
            self.capacity
        }

        #[inline(always)]
        fn verify_init(&self, source: &str) {
            debug_assert!(
                self.is_memory_initialized.load(Ordering::Relaxed),
                "Undefined behavior when calling \"{source}\" and the object is not initialized."
            );
        }

        fn set(&self, idx: usize, value: bool) -> bool {
            self.verify_init("set");
            debug_assert!(
                idx < self.capacity,
                "This should never happen. Out of bounds access with index {idx}."
            );

            unsafe { (*self.data_ptr.as_ptr().add(idx)).swap(value, Ordering::Relaxed) }
        }

        pub fn insert(&self, value: usize) -> bool {
            !self.set(value, true)
        }

        pub fn remove(&self, value: usize) -> bool {
            self.set(value, false)
        }

        pub fn remove_all<F: FnMut(usize)>(&self, mut callback: F) {
            self.verify_init("pop");

            for i in 0..self.capacity {
                if unsafe { (*self.data_ptr.as_ptr().add(i)).swap(false, Ordering::Relaxed) } {
                    callback(i);
                }
            }
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct FixedSizeUsedChunkList<const CAPACITY: usize> {
    list: RelocatableUsedChunkList,
    data: [IoxAtomicBool; CAPACITY],
}

impl<const CAPACITY: usize> Default for FixedSizeUsedChunkList<CAPACITY> {
    fn default() -> Self {
        let mut new_self = Self {
            list: unsafe { RelocatableUsedChunkList::new_uninit(CAPACITY) },
            data: core::array::from_fn(|_| IoxAtomicBool::new(false)),
        };

        let allocator = BumpAllocator::new(new_self.data.as_mut_ptr().cast());
        unsafe {
            new_self
                .list
                .init(&allocator)
                .expect("All required memory is preallocated.")
        };
        new_self
    }
}

impl<const CAPACITY: usize> FixedSizeUsedChunkList<CAPACITY> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&self, value: usize) -> bool {
        self.list.insert(value)
    }

    pub fn remove_all<F: FnMut(usize)>(&mut self, callback: F) {
        self.list.remove_all(callback)
    }

    pub const fn capacity(&self) -> usize {
        CAPACITY
    }

    pub fn remove(&self, value: usize) -> bool {
        self.list.remove(value)
    }
}
