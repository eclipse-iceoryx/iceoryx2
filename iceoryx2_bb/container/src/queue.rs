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

//! Two relocatable (inter process shared memory compatible) queues.
//!
//! The [`Queue`] which has a
//! fixed capacity defined at runtime and the [`FixedSizeQueue`] which has a fixed capacity at
//! compile time.
//!
//! # Examples
//!
//! ## Create [`Queue`] inside constructs which provides memory
//!
//! ```
//! use iceoryx2_bb_container::queue::RelocatableQueue;
//! use iceoryx2_bb_elementary::math::align_to;
//! use iceoryx2_bb_elementary::relocatable_container::RelocatableContainer;
//!
//! const QUEUE_CAPACITY:usize = 12;
//! struct MyConstruct {
//!     queue: RelocatableQueue<u128>,
//!     data: [u128; QUEUE_CAPACITY],
//! }
//!
//! impl MyConstruct {
//!     pub fn new() -> Self {
//!         Self {
//!             queue: unsafe { RelocatableQueue::new(QUEUE_CAPACITY,
//!                             align_to::<u128>(std::mem::size_of::<u128>()) as isize) },
//!             data: [0; QUEUE_CAPACITY]
//!         }
//!     }
//! }
//! ```
//!
//! ## Create [`Queue`] with allocator
//!
//! ```
//! use iceoryx2_bb_container::queue::RelocatableQueue;
//! use iceoryx2_bb_memory::bump_allocator::BumpAllocator;
//! use iceoryx2_bb_elementary::relocatable_container::RelocatableContainer;
//! use std::ptr::NonNull;
//!
//! const QUEUE_CAPACITY:usize = 12;
//! const MEM_SIZE: usize = RelocatableQueue::<u128>::const_memory_size(QUEUE_CAPACITY);
//! let mut memory = [0u8; MEM_SIZE];
//!
//! let bump_allocator = BumpAllocator::new(
//!                         unsafe { NonNull::new_unchecked(memory.as_mut_ptr() as *mut u8) },
//!                         MEM_SIZE);
//!
//! let queue = unsafe { RelocatableQueue::<u128>::new_uninit(QUEUE_CAPACITY) };
//! unsafe { queue.init(&bump_allocator).expect("queue init failed") };
//! ```

use iceoryx2_bb_elementary::allocator::{AllocationError, BaseAllocator};
use iceoryx2_bb_elementary::math::align_to;
use iceoryx2_bb_elementary::owning_pointer::OwningPointer;
use iceoryx2_bb_elementary::pointer_trait::PointerTrait;
use iceoryx2_bb_elementary::relocatable_ptr::RelocatablePointer;
use iceoryx2_bb_log::{fail, fatal_panic};
use std::sync::atomic::AtomicBool;
use std::{alloc::Layout, fmt::Debug, mem::MaybeUninit};

pub use iceoryx2_bb_elementary::relocatable_container::RelocatableContainer;

pub type Queue<T> = details::Queue<T, OwningPointer<MaybeUninit<T>>>;
pub type RelocatableQueue<T> = details::Queue<T, RelocatablePointer<MaybeUninit<T>>>;

mod details {
    use std::marker::PhantomData;

    use super::*;
    /// **Non-movable** relocatable queue with runtime fixed size capacity.
    #[repr(C)]
    #[derive(Debug)]
    pub struct Queue<T, PointerType: PointerTrait<MaybeUninit<T>>> {
        data_ptr: PointerType,
        start: usize,
        len: usize,
        capacity: usize,
        is_initialized: AtomicBool,
        _phantom_data: PhantomData<T>,
    }

    unsafe impl<T: Send, PointerType: PointerTrait<MaybeUninit<T>>> Send for Queue<T, PointerType> {}

    impl<T> Queue<T, OwningPointer<MaybeUninit<T>>> {
        pub fn new(capacity: usize) -> Self {
            Self {
                data_ptr: OwningPointer::<MaybeUninit<T>>::new_with_alloc(capacity),
                start: 0,
                len: 0,
                capacity,
                is_initialized: AtomicBool::new(true),
                _phantom_data: PhantomData,
            }
        }
    }

    impl<T: Copy + Debug, PointerType: PointerTrait<MaybeUninit<T>> + Debug> Queue<T, PointerType> {
        /// Returns a copy of the element stored at index. The index is starting by 0 for the first
        /// element until [`Queue::len()`].
        pub unsafe fn get_unchecked(&self, index: usize) -> T {
            unsafe {
                (*self
                    .data_ptr
                    .as_ptr()
                    .add((self.start - self.len + index) % self.capacity))
                .assume_init()
            }
        }

        /// Returns a copy of the element stored at index. The index is starting by 0 for the first
        /// element until [`Queue::len()`].
        pub fn get(&self, index: usize) -> T {
            if self.len() <= index {
                fatal_panic!(from self, "Unable to copy content since the index {} is out of range.", index);
            }

            unsafe { self.get_unchecked(index) }
        }
    }

    impl<T> RelocatableContainer for Queue<T, RelocatablePointer<MaybeUninit<T>>> {
        unsafe fn new(capacity: usize, distance_to_data: isize) -> Self {
            Self {
                data_ptr: RelocatablePointer::new(distance_to_data),
                start: 0,
                len: 0,
                capacity,
                is_initialized: AtomicBool::new(true),
                _phantom_data: PhantomData,
            }
        }

        unsafe fn new_uninit(capacity: usize) -> Self {
            Self {
                data_ptr: RelocatablePointer::new_uninit(),
                start: 0,
                len: 0,
                capacity,
                is_initialized: AtomicBool::new(false),
                _phantom_data: PhantomData,
            }
        }

        unsafe fn init<Allocator: BaseAllocator>(
            &self,
            allocator: &Allocator,
        ) -> Result<(), AllocationError> {
            if self
                .is_initialized
                .load(std::sync::atomic::Ordering::Relaxed)
            {
                fatal_panic!(
                    from "Queue::init()",
                    "Memory already initialized. Initializing it twice may lead to undefined behavior."
                );
            }

            self.data_ptr.init(fail!(from "Queue::init", when allocator
                 .allocate(Layout::from_size_align_unchecked(
                     std::mem::size_of::<T>() * self.capacity,
                     std::mem::align_of::<T>(),
                 )), "Failed to initialize queue since the allocation of the data memory failed."
            ));
            self.is_initialized
                .store(true, std::sync::atomic::Ordering::Relaxed);

            Ok(())
        }

        fn memory_size(capacity: usize) -> usize {
            Self::const_memory_size(capacity)
        }
    }

    impl<T, PointerType: PointerTrait<MaybeUninit<T>>> Queue<T, PointerType> {
        fn verify_init(&self, source: &str) {
            if !self
                .is_initialized
                .load(std::sync::atomic::Ordering::Relaxed)
            {
                fatal_panic!(from source, "Undefined behavior - the object was not initialized with 'init' before.");
            }
        }

        /// Returns the required memory size for a queue with a specified capacity
        pub const fn const_memory_size(capacity: usize) -> usize {
            std::mem::size_of::<T>() * capacity + std::mem::align_of::<T>() - 1
        }

        /// Returns true if the queue is empty, otherwise false
        pub fn is_empty(&self) -> bool {
            self.len == 0
        }

        /// Removes all elements from the queue
        ///
        /// # Safety
        ///
        ///  * Only use this method when [`Queue::init()`] was called before
        ///
        pub unsafe fn clear(&mut self) {
            while self.pop().is_some() {}
        }

        /// Returns the capacity of the queue
        pub fn capacity(&self) -> usize {
            self.capacity
        }

        /// Returns the number of elements inside the queue
        pub fn len(&self) -> usize {
            self.len
        }

        /// Returns true if the queue is full, otherwise false
        pub fn is_full(&self) -> bool {
            self.len() == self.capacity()
        }

        /// Acquire an element from the queue. If the queue is empty it returns [`None`].
        pub unsafe fn pop(&mut self) -> Option<T> {
            if self.is_empty() {
                return None;
            }

            self.verify_init(&format!("Queue<{}>::pop()", std::any::type_name::<T>()));
            let index = (self.start - self.len) % self.capacity;
            self.len -= 1;
            let value = std::mem::replace(
                &mut *self.data_ptr.as_mut_ptr().add(index),
                MaybeUninit::uninit(),
            );
            Some(value.assume_init())
        }

        /// Adds an element to the queue. If the queue is full it returns false, otherwise true.
        pub unsafe fn push(&mut self, value: T) -> bool {
            if self.len == self.capacity {
                return false;
            }

            self.verify_init(&format!("Queue<{}>::push()", std::any::type_name::<T>()));

            self.unchecked_push(value);
            true
        }

        /// Adds an element to the queue. If the queue is full it returns the oldest element,
        /// otherwise [`None`].
        pub unsafe fn push_with_overflow(&mut self, value: T) -> Option<T> {
            let overridden_value = if self.len() == self.capacity() {
                self.pop()
            } else {
                None
            };

            self.verify_init(&format!(
                "Queue<{}>::push_with_overflow()",
                std::any::type_name::<T>()
            ));
            self.unchecked_push(value);
            overridden_value
        }

        unsafe fn unchecked_push(&mut self, value: T) {
            let index = (self.start) % self.capacity;
            self.data_ptr
                .as_mut_ptr()
                .add(index)
                .write(MaybeUninit::new(value));
            self.start += 1;
            self.len += 1;
        }
    }

    impl<T, PointerType: PointerTrait<MaybeUninit<T>>> Drop for Queue<T, PointerType> {
        fn drop(&mut self) {
            unsafe { self.clear() }
        }
    }
}

/// Relocatable queue with compile time fixed size capacity. In contrast to its counterpart the
/// [`Queue`] it is movable.
#[repr(C)]
#[derive(Debug)]
pub struct FixedSizeQueue<T, const CAPACITY: usize> {
    state: RelocatableQueue<T>,
    _data: [MaybeUninit<T>; CAPACITY],
}

impl<T, const CAPACITY: usize> Default for FixedSizeQueue<T, CAPACITY> {
    fn default() -> Self {
        Self {
            state: unsafe {
                RelocatableQueue::new(
                    CAPACITY,
                    align_to::<MaybeUninit<T>>(std::mem::size_of::<RelocatableQueue<T>>()) as isize,
                )
            },
            _data: unsafe { MaybeUninit::uninit().assume_init() },
        }
    }
}

unsafe impl<T: Send, const CAPACITY: usize> Send for FixedSizeQueue<T, CAPACITY> {}
unsafe impl<T: Sync, const CAPACITY: usize> Sync for FixedSizeQueue<T, CAPACITY> {}

impl<T, const CAPACITY: usize> FixedSizeQueue<T, CAPACITY> {
    /// Creates a new queue.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns true if the queue is empty, otherwise false
    pub fn is_empty(&self) -> bool {
        self.state.is_empty()
    }

    /// Removes all elements from the queue
    pub fn clear(&mut self) {
        unsafe { self.state.clear() }
    }

    /// Returns the capacity of the queue
    pub fn capacity(&self) -> usize {
        self.state.capacity()
    }

    /// Returns the number of elements inside the queue
    pub fn len(&self) -> usize {
        self.state.len()
    }

    /// Returns true if the queue is full, otherwise false
    pub fn is_full(&self) -> bool {
        self.state.is_full()
    }

    /// Acquire an element from the queue. If the queue is empty it returns [`None`].
    pub fn pop(&mut self) -> Option<T> {
        unsafe { self.state.pop() }
    }

    /// Adds an element to the queue. If the queue is full it returns false, otherwise true.
    pub fn push(&mut self, value: T) -> bool {
        unsafe { self.state.push(value) }
    }

    /// Adds an element to the queue. If the queue is full it returns the oldest element,
    /// otherwise [`None`].
    pub fn push_with_overflow(&mut self, value: T) -> Option<T> {
        unsafe { self.state.push_with_overflow(value) }
    }
}

impl<T: Copy + Debug, const CAPACITY: usize> FixedSizeQueue<T, CAPACITY> {
    /// Returns a copy of the element stored at index. The index is starting by 0 for the first
    /// element until [`FixedSizeQueue::len()`].
    ///
    /// # Safety
    ///
    ///  * The index must be not out of bounds
    ///
    pub unsafe fn get_unchecked(&self, index: usize) -> T {
        self.state.get_unchecked(index)
    }

    /// Returns a copy of the element stored at index. The index is starting by 0 for the first
    /// element until [`FixedSizeQueue::len()`].
    pub fn get(&self, index: usize) -> T {
        self.state.get(index)
    }
}
