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

//! A **threadsafe** **lock-free** single producer single consumer queue which can store [`usize`]
//! integers or indices with overflow behavior. When the queue is full the oldest element is
//! returned to the producer and replaced with the newest.
//!
//! # Example
//!
//! ```
//! use iceoryx2_bb_lock_free::spsc::safely_overflowing_index_queue::*;
//!
//! const QUEUE_CAPACITY: usize = 128;
//! let queue = FixedSizeSafelyOverflowingIndexQueue::<QUEUE_CAPACITY>::new();
//!
//! let mut producer = match queue.acquire_producer() {
//!     None => panic!("a producer has been already acquired."),
//!     Some(p) => p,
//! };
//!
//! match producer.push(1234) {
//!     Some(e) => println!("queue is full, recycled element {}", e),
//!     None => println!("add element to queue")
//! }
//!
//! let mut consumer = match queue.acquire_consumer() {
//!     None => panic!("a consumer has been already acquired."),
//!     Some(p) => p,
//! };
//!
//! match consumer.pop() {
//!     None => println!("queue is empty"),
//!     Some(v) => println!("got {}", v)
//! }
//! ```

use core::{alloc::Layout, cell::UnsafeCell, fmt::Debug, sync::atomic::Ordering};
use iceoryx2_bb_elementary::math::unaligned_mem_size;
use iceoryx2_bb_elementary::{bump_allocator::BumpAllocator, relocatable_ptr::RelocatablePointer};
use iceoryx2_bb_elementary_traits::{
    owning_pointer::OwningPointer, pointer_trait::PointerTrait,
    relocatable_container::RelocatableContainer,
};
use iceoryx2_bb_log::{fail, fatal_panic};
use iceoryx2_pal_concurrency_sync::iox_atomic::IoxAtomicBool;
use iceoryx2_pal_concurrency_sync::iox_atomic::IoxAtomicU64;

/// The [`Producer`] of the [`SafelyOverflowingIndexQueue`]/[`FixedSizeSafelyOverflowingIndexQueue`]
/// which can add values to it via [`Producer::push()`].
#[derive(Debug)]
pub struct Producer<'a, PointerType: PointerTrait<UnsafeCell<u64>>> {
    queue: &'a details::SafelyOverflowingIndexQueue<PointerType>,
}

impl<PointerType: PointerTrait<UnsafeCell<u64>> + Debug> Producer<'_, PointerType> {
    /// Adds a new value to the [`SafelyOverflowingIndexQueue`]/[`FixedSizeSafelyOverflowingIndexQueue`].
    /// If the queue is full it returns false, otherwise true.
    pub fn push(&mut self, t: u64) -> Option<u64> {
        unsafe { self.queue.push(t) }
    }
}

impl<PointerType: PointerTrait<UnsafeCell<u64>>> Drop for Producer<'_, PointerType> {
    fn drop(&mut self) {
        self.queue.has_producer.store(true, Ordering::Relaxed);
    }
}

/// The [`Consumer`] of the [`SafelyOverflowingIndexQueue`]/[`FixedSizeSafelyOverflowingIndexQueue`]
/// which can acquire values from it via [`Consumer::pop()`].
#[derive(Debug)]
pub struct Consumer<'a, PointerType: PointerTrait<UnsafeCell<u64>>> {
    queue: &'a details::SafelyOverflowingIndexQueue<PointerType>,
}

impl<PointerType: PointerTrait<UnsafeCell<u64>> + Debug> Consumer<'_, PointerType> {
    /// Acquires a value from the [`SafelyOverflowingIndexQueue`]/[`FixedSizeSafelyOverflowingIndexQueue`].
    /// If the queue is empty it returns [`None`] otherwise the value.
    pub fn pop(&mut self) -> Option<u64> {
        unsafe { self.queue.pop() }
    }
}

impl<PointerType: PointerTrait<UnsafeCell<u64>>> Drop for Consumer<'_, PointerType> {
    fn drop(&mut self) {
        self.queue.has_consumer.store(true, Ordering::Relaxed);
    }
}

/// Non-relocatable version of the safely overflowing index queue
pub type SafelyOverflowingIndexQueue =
    details::SafelyOverflowingIndexQueue<OwningPointer<UnsafeCell<u64>>>;

/// Relocatable version of the safely overflowing index queue
pub type RelocatableSafelyOverflowingIndexQueue =
    details::SafelyOverflowingIndexQueue<RelocatablePointer<UnsafeCell<u64>>>;

pub mod details {
    use super::*;

    /// A threadsafe lock-free safely overflowing index queue with a capacity which can be set up at runtime,
    /// when the queue is created. When the queue is full the oldest element is returned to the producer
    /// and overridden with the newest element.
    #[derive(Debug)]
    #[repr(C)]
    pub struct SafelyOverflowingIndexQueue<PointerType: PointerTrait<UnsafeCell<u64>>> {
        data_ptr: PointerType,
        capacity: usize,
        write_position: IoxAtomicU64,
        read_position: IoxAtomicU64,
        pub(super) has_producer: IoxAtomicBool,
        pub(super) has_consumer: IoxAtomicBool,
        is_memory_initialized: IoxAtomicBool,
    }

    unsafe impl<PointerType: PointerTrait<UnsafeCell<u64>>> Sync
        for SafelyOverflowingIndexQueue<PointerType>
    {
    }
    unsafe impl<PointerType: PointerTrait<UnsafeCell<u64>>> Send
        for SafelyOverflowingIndexQueue<PointerType>
    {
    }

    impl SafelyOverflowingIndexQueue<OwningPointer<UnsafeCell<u64>>> {
        pub fn new(capacity: usize) -> Self {
            let mut data_ptr = OwningPointer::<UnsafeCell<u64>>::new_with_alloc(capacity + 1);

            for i in 0..capacity + 1 {
                unsafe { data_ptr.as_mut_ptr().add(i).write(UnsafeCell::new(0)) };
            }

            Self {
                data_ptr,
                capacity,
                write_position: IoxAtomicU64::new(0),
                read_position: IoxAtomicU64::new(0),
                has_producer: IoxAtomicBool::new(true),
                has_consumer: IoxAtomicBool::new(true),
                is_memory_initialized: IoxAtomicBool::new(true),
            }
        }
    }

    impl RelocatableContainer for SafelyOverflowingIndexQueue<RelocatablePointer<UnsafeCell<u64>>> {
        unsafe fn new_uninit(capacity: usize) -> Self {
            Self {
                data_ptr: RelocatablePointer::new_uninit(),
                capacity,
                write_position: IoxAtomicU64::new(0),
                read_position: IoxAtomicU64::new(0),
                has_producer: IoxAtomicBool::new(true),
                has_consumer: IoxAtomicBool::new(true),
                is_memory_initialized: IoxAtomicBool::new(false),
            }
        }

        unsafe fn init<T: iceoryx2_bb_elementary_traits::allocator::BaseAllocator>(
            &mut self,
            allocator: &T,
        ) -> Result<(), iceoryx2_bb_elementary_traits::allocator::AllocationError> {
            if self.is_memory_initialized.load(Ordering::Relaxed) {
                fatal_panic!(from self, "Memory already initialized. Initializing it twice may lead to undefined behavior.");
            }

            self.data_ptr.init(fail!(from self, when allocator
            .allocate( Layout::from_size_align_unchecked(
                    core::mem::size_of::<u64>() * (self.capacity + 1),
                    core::mem::align_of::<u64>())),
            "Failed to initialize since the allocation of the data memory failed."));

            for i in 0..self.capacity + 1 {
                (self.data_ptr.as_ptr() as *mut UnsafeCell<usize>)
                    .add(i)
                    .write(UnsafeCell::new(0));
            }

            self.is_memory_initialized.store(true, Ordering::Relaxed);
            Ok(())
        }

        fn memory_size(capacity: usize) -> usize {
            Self::const_memory_size(capacity)
        }
    }

    impl<PointerType: PointerTrait<UnsafeCell<u64>> + Debug> SafelyOverflowingIndexQueue<PointerType> {
        #[inline(always)]
        fn verify_init(&self, source: &str) {
            debug_assert!(
                self.is_memory_initialized.load(Ordering::Relaxed),
                "Undefined behavior when calling SafelyOverflowingIndexQueue::{source} and the object is not initialized."
            );
        }

        /// Returns the amount of memory required to create a [`SafelyOverflowingIndexQueue`] with
        /// the provided capacity.
        pub const fn const_memory_size(capacity: usize) -> usize {
            unaligned_mem_size::<UnsafeCell<u64>>(capacity + 1)
        }

        fn at(&self, position: u64) -> *mut u64 {
            unsafe {
                (*self
                    .data_ptr
                    .as_ptr()
                    .add((position % (self.capacity as u64 + 1)) as usize))
                .get()
            }
        }
        /// Acquires the [`Producer`] of the [`SafelyOverflowingIndexQueue`]. This is threadsafe and
        /// lock-free without restrictions but when another thread has already acquired the [`Producer`]
        /// it returns [`None`] since it is a single producer single consumer
        /// [`SafelyOverflowingIndexQueue`].
        /// ```
        /// use iceoryx2_bb_lock_free::spsc::safely_overflowing_index_queue::*;
        ///
        /// const QUEUE_CAPACITY: usize = 128;
        /// let queue = FixedSizeSafelyOverflowingIndexQueue::<QUEUE_CAPACITY>::new();
        ///
        /// let mut producer = match queue.acquire_producer() {
        ///     None => panic!("a producer has been already acquired."),
        ///     Some(p) => p,
        /// };
        ///
        /// match producer.push(1234) {
        ///     Some(e) => println!("queue is full, recycle element {}", e),
        ///     None => println!("added element to queue")
        /// }
        /// ```
        pub fn acquire_producer(&self) -> Option<Producer<'_, PointerType>> {
            self.verify_init("acquire_producer()");
            match self.has_producer.compare_exchange(
                true,
                false,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => Some(Producer { queue: self }),
                Err(_) => None,
            }
        }
        /// Acquires the [`Consumer`] of the [`SafelyOverflowingIndexQueue`]. This is threadsafe and
        /// lock-free without restrictions but when another thread has already acquired the [`Consumer`]
        /// it returns [`None`] since it is a single producer single consumer
        /// [`SafelyOverflowingIndexQueue`].
        /// ```
        /// use iceoryx2_bb_lock_free::spsc::safely_overflowing_index_queue::*;
        ///
        /// const QUEUE_CAPACITY: usize = 128;
        /// let queue = FixedSizeSafelyOverflowingIndexQueue::<QUEUE_CAPACITY>::new();
        ///
        /// let mut consumer = match queue.acquire_consumer() {
        ///     None => panic!("a consumer has been already acquired."),
        ///     Some(p) => p,
        /// };
        ///
        /// match consumer.pop() {
        ///     None => println!("queue is empty"),
        ///     Some(v) => println!("got {}", v)
        /// }
        /// ```
        pub fn acquire_consumer(&self) -> Option<Consumer<'_, PointerType>> {
            self.verify_init("acquire_consumer()");
            match self.has_consumer.compare_exchange(
                true,
                false,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => Some(Consumer { queue: self }),
                Err(_) => None,
            }
        }

        /// Push an index into the [`SafelyOverflowingIndexQueue`]. If the queue is full the oldest
        /// index is returned and replaced with the new value.
        ///
        /// # Safety
        ///
        ///  * [`SafelyOverflowingIndexQueue::push()`] cannot be called concurrently. The user has
        ///    to ensure that at most one thread access this method.
        ///  * It has to be ensured that the memory is initialized with
        ///    [`SafelyOverflowingIndexQueue::init()`].
        pub unsafe fn push(&self, value: u64) -> Option<u64> {
            ////////////////
            // SYNC POINT R
            ////////////////
            // required when push in overflow case is called non-concurrently from a different
            // thread
            let write_position = self.write_position.load(Ordering::Acquire);
            let read_position = self.read_position.load(Ordering::Relaxed);
            let is_full = write_position == read_position + self.capacity as u64;

            unsafe { self.at(write_position).write(value) };

            ////////////////
            // SYNC POINT W
            ////////////////
            self.write_position
                .store(write_position + 1, Ordering::Release);

            if is_full
                && self
                    .read_position
                    .compare_exchange(
                        read_position,
                        read_position + 1,
                        ////////////////
                        // SYNC POINT R
                        ////////////////
                        Ordering::AcqRel,
                        Ordering::Relaxed,
                    )
                    .is_ok()
            {
                let value = unsafe { *self.at(read_position) };
                Some(value)
            } else {
                None
            }
        }

        /// Acquires an index from the [`SafelyOverflowingIndexQueue`]. If the queue is empty
        /// [`None`] is returned.
        ///
        /// # Safety
        ///
        ///  * [`SafelyOverflowingIndexQueue::pop()`] cannot be called concurrently. The user has
        ///    to ensure that at most one thread access this method.
        ///  * It has to be ensured that the memory is initialized with
        ///    [`SafelyOverflowingIndexQueue::init()`].
        pub unsafe fn pop(&self) -> Option<u64> {
            let mut read_position = self.read_position.load(Ordering::Relaxed);
            ////////////////
            // SYNC POINT W
            ////////////////
            let is_empty = read_position == self.write_position.load(Ordering::Acquire);

            if is_empty {
                return None;
            }

            let mut value;
            loop {
                value = unsafe { *self.at(read_position) };

                match self.read_position.compare_exchange(
                    read_position,
                    read_position + 1,
                    Ordering::Relaxed,
                    ////////////////
                    // SYNC POINT R
                    ////////////////
                    Ordering::Acquire,
                ) {
                    Ok(_) => break,
                    Err(v) => read_position = v,
                }
            }

            Some(value)
        }

        fn acquire_read_and_write_position(&self) -> (u64, u64) {
            loop {
                let write_position = self.write_position.load(Ordering::Relaxed);
                let read_position = self.read_position.load(Ordering::Relaxed);

                if write_position == self.write_position.load(Ordering::Relaxed)
                    && read_position == self.read_position.load(Ordering::Relaxed)
                {
                    return (write_position, read_position);
                }
            }
        }

        /// Returns true when the [`SafelyOverflowingIndexQueue`] is empty, otherwise false.
        /// Note: This method may make only sense in a non-concurrent setup since the information
        ///       could be out-of-date as soon as it is acquired.
        pub fn is_empty(&self) -> bool {
            let (write_position, read_position) = self.acquire_read_and_write_position();
            write_position == read_position
        }

        /// Returns the length of the [`SafelyOverflowingIndexQueue`].
        /// Note: This method may make only sense in a non-concurrent setup since the information
        ///       could be out-of-date as soon as it is acquired.
        pub fn len(&self) -> usize {
            let (write_position, read_position) = self.acquire_read_and_write_position();
            (write_position - read_position) as usize
        }

        /// Returns the capacity of the [`SafelyOverflowingIndexQueue`].
        pub const fn capacity(&self) -> usize {
            self.capacity
        }

        /// Returns true when the [`SafelyOverflowingIndexQueue`] is full, otherwise false.
        /// Note: This method may make only sense in a non-concurrent setup since the information
        ///       could be out-of-date as soon as it is acquired.
        pub fn is_full(&self) -> bool {
            let (write_position, read_position) = self.acquire_read_and_write_position();
            write_position == read_position + self.capacity as u64
        }
    }
}

/// The compile-time fixed size version of the [`SafelyOverflowingIndexQueue`].
#[derive(Debug)]
#[repr(C)]
pub struct FixedSizeSafelyOverflowingIndexQueue<const CAPACITY: usize> {
    state: RelocatableSafelyOverflowingIndexQueue,
    data: [UnsafeCell<u64>; CAPACITY],
    data_plus_one: UnsafeCell<u64>,
}

unsafe impl<const CAPACITY: usize> Sync for FixedSizeSafelyOverflowingIndexQueue<CAPACITY> {}
unsafe impl<const CAPACITY: usize> Send for FixedSizeSafelyOverflowingIndexQueue<CAPACITY> {}

impl<const CAPACITY: usize> Default for FixedSizeSafelyOverflowingIndexQueue<CAPACITY> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const CAPACITY: usize> FixedSizeSafelyOverflowingIndexQueue<CAPACITY> {
    /// Creates a new empty [`FixedSizeSafelyOverflowingIndexQueue`].
    pub fn new() -> Self {
        let mut new_self = Self {
            state: unsafe { RelocatableSafelyOverflowingIndexQueue::new_uninit(CAPACITY) },
            data: core::array::from_fn(|_| UnsafeCell::new(0)),
            data_plus_one: UnsafeCell::new(0),
        };

        let allocator = BumpAllocator::new(new_self.data.as_mut_ptr().cast());
        unsafe {
            new_self
                .state
                .init(&allocator)
                .expect("All required memory is preallocated.")
        };

        new_self
    }

    /// See [`SafelyOverflowingIndexQueue::acquire_producer()`]
    pub fn acquire_producer(&self) -> Option<Producer<'_, RelocatablePointer<UnsafeCell<u64>>>> {
        self.state.acquire_producer()
    }

    /// See [`SafelyOverflowingIndexQueue::acquire_consumer()`]
    pub fn acquire_consumer(&self) -> Option<Consumer<'_, RelocatablePointer<UnsafeCell<u64>>>> {
        self.state.acquire_consumer()
    }

    /// See [`SafelyOverflowingIndexQueue::is_empty()`]
    pub fn is_empty(&self) -> bool {
        self.state.is_empty()
    }

    /// See [`SafelyOverflowingIndexQueue::len()`]
    pub fn len(&self) -> usize {
        self.state.len()
    }

    /// See [`SafelyOverflowingIndexQueue::push()`]
    ///
    /// # Safety
    ///
    /// * It must be ensured that no other thread/process calls this method concurrently
    ///
    pub unsafe fn push(&self, value: u64) -> Option<u64> {
        self.state.push(value)
    }

    /// See [`SafelyOverflowingIndexQueue::pop()`]
    ///
    /// # Safety
    ///
    /// * It must be ensured that no other thread/process calls this method concurrently
    ///
    pub unsafe fn pop(&self) -> Option<u64> {
        self.state.pop()
    }

    /// See [`SafelyOverflowingIndexQueue::capacity()`]
    pub const fn capacity(&self) -> usize {
        self.state.capacity()
    }

    /// See [`SafelyOverflowingIndexQueue::is_full()`]
    pub fn is_full(&self) -> bool {
        self.state.is_full()
    }
}
