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

//! Three queue variations that are similar to [`std::collections::VecDeque`].
//!
//!  * [`FixedSizeQueue`](crate::queue::FixedSizeQueue), compile-time fixed size queue that
//!     is self-contained.
//!  * [`RelocatableQueue`](crate::queue::RelocatableQueue), run-time fixed size queue that
//!     acquires the required memory from a custom user-provided allocator.
//!  * [`Queue`](crate::queue::Queue), run-time fixed size queue that uses by default
//!     heap memory.
//!
//! # Basic Examples
//!
//! ## Use the [`FixedSizeQueue`](crate::queue::FixedSizeQueue)
//!
//! ```
//! use iceoryx2_bb_container::queue::FixedSizeQueue;
//!
//! const QUEUE_CAPACITY: usize = 1;
//! let mut queue = FixedSizeQueue::<u64, QUEUE_CAPACITY>::new();
//!
//! queue.push(123);
//!
//! // queue is full, we override the oldest element (123) with the new number (456)
//! queue.push_with_overflow(456);
//!
//! println!("pop from queue {}", queue.pop().unwrap());
//! ```
//!
//! ## Use the [`Queue`](crate::queue::Queue)
//!
//! ```
//! use iceoryx2_bb_container::queue::Queue;
//!
//! let queue_capacity = 1234;
//! let mut queue = Queue::<u64>::new(queue_capacity);
//!
//! queue.push(123);
//!
//! println!("pop from queue {}", queue.pop().unwrap());
//! ```
//!
//! # Advanced Examples
//!
//! ## Create [`RelocatableQueue`](crate::queue::RelocatableQueue) inside constructs which provides memory
//!
//! ```
//! use iceoryx2_bb_container::queue::RelocatableQueue;
//! use iceoryx2_bb_elementary::math::align_to;
//! use iceoryx2_bb_elementary::bump_allocator::BumpAllocator;
//! use iceoryx2_bb_elementary_traits::relocatable_container::RelocatableContainer;
//! use core::mem::MaybeUninit;
//!
//! const QUEUE_CAPACITY:usize = 12;
//! struct MyConstruct {
//!     queue: RelocatableQueue<u128>,
//!     queue_memory: [MaybeUninit<u128>; QUEUE_CAPACITY],
//! }
//!
//! impl MyConstruct {
//!     pub fn new() -> Self {
//!         let mut new_self = Self {
//!             queue: unsafe { RelocatableQueue::new_uninit(QUEUE_CAPACITY) },
//!             queue_memory: core::array::from_fn(|_| MaybeUninit::uninit()),
//!         };
//!
//!         let allocator = BumpAllocator::new(new_self.queue_memory.as_mut_ptr().cast());
//!         unsafe {
//!             new_self.queue.init(&allocator).expect("Enough memory provided.")
//!         };
//!         new_self
//!     }
//! }
//! ```
//!
//! ## Create [`RelocatableQueue`](crate::queue::RelocatableQueue) with allocator
//!
//! ```
//! use iceoryx2_bb_container::queue::RelocatableQueue;
//! use iceoryx2_bb_elementary::bump_allocator::BumpAllocator;
//! use iceoryx2_bb_elementary_traits::relocatable_container::RelocatableContainer;
//! use core::ptr::NonNull;
//!
//! const QUEUE_CAPACITY:usize = 12;
//! const MEM_SIZE: usize = RelocatableQueue::<u128>::const_memory_size(QUEUE_CAPACITY);
//! let mut memory = [0u8; MEM_SIZE];
//!
//! let bump_allocator = BumpAllocator::new(memory.as_mut_ptr());
//!
//! let mut queue = unsafe { RelocatableQueue::<u128>::new_uninit(QUEUE_CAPACITY) };
//! unsafe { queue.init(&bump_allocator).expect("queue init failed") };
//! ```
//!
use core::marker::PhantomData;
use core::{alloc::Layout, fmt::Debug, mem::MaybeUninit};
use iceoryx2_bb_elementary::bump_allocator::BumpAllocator;
use iceoryx2_bb_elementary::math::unaligned_mem_size;
use iceoryx2_bb_elementary::relocatable_ptr::{GenericRelocatablePointer, RelocatablePointer};
use iceoryx2_bb_elementary_traits::allocator::{AllocationError, BaseAllocator};
use iceoryx2_bb_elementary_traits::generic_pointer::GenericPointer;
use iceoryx2_bb_elementary_traits::owning_pointer::{GenericOwningPointer, OwningPointer};
use iceoryx2_bb_elementary_traits::placement_default::PlacementDefault;
use iceoryx2_bb_elementary_traits::pointer_trait::PointerTrait;
pub use iceoryx2_bb_elementary_traits::relocatable_container::RelocatableContainer;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use iceoryx2_bb_log::{fail, fatal_panic};
use iceoryx2_pal_concurrency_sync::iox_atomic::IoxAtomicBool;

/// Queue with run-time fixed size capacity. In contrast to its counterpart the
/// [`RelocatableQueue`] it is movable but is not shared memory compatible.
pub type Queue<T> = MetaQueue<T, GenericOwningPointer>;
/// **Non-movable** relocatable queue with runtime fixed size capacity.
pub type RelocatableQueue<T> = MetaQueue<T, GenericRelocatablePointer>;

#[doc(hidden)]
/// **Non-movable** relocatable queue with runtime fixed size capacity.
#[repr(C)]
#[derive(Debug)]
pub struct MetaQueue<T, Ptr: GenericPointer> {
    data_ptr: Ptr::Type<MaybeUninit<T>>,
    start: usize,
    len: usize,
    capacity: usize,
    is_initialized: IoxAtomicBool,
    _phantom_data: PhantomData<T>,
}

unsafe impl<T: Send, Ptr: GenericPointer> Send for MetaQueue<T, Ptr> {}

impl<T> Queue<T> {
    /// Creates a new [`Queue`] with the provided capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            data_ptr: OwningPointer::<MaybeUninit<T>>::new_with_alloc(capacity),
            start: 0,
            len: 0,
            capacity,
            is_initialized: IoxAtomicBool::new(true),
            _phantom_data: PhantomData,
        }
    }

    /// Removes all elements from the queue
    pub fn clear(&mut self) {
        unsafe { self.clear_impl() }
    }

    /// Returns a reference to the element from the beginning of the queue without removing it.
    /// If the queue is empty it returns [`None`].
    pub fn peek(&self) -> Option<&T> {
        unsafe { self.peek_impl() }
    }

    /// Returns a mutable reference to the element from the beginning of the queue without removing it.
    /// If the queue is empty it returns [`None`].
    pub fn peek_mut(&mut self) -> Option<&mut T> {
        unsafe { self.peek_mut_impl() }
    }

    /// Removes the element from the beginning of the queue. If the queue is empty it returns [`None`].
    pub fn pop(&mut self) -> Option<T> {
        unsafe { self.pop_impl() }
    }

    /// Adds an element at the end of the queue. If the queue is full it returns false, otherwise true.
    pub fn push(&mut self, value: T) -> bool {
        unsafe { self.push_impl(value) }
    }

    /// Adds an element at the end of the queue. If the queue is full it returns the oldest element,
    /// otherwise [`None`].
    pub fn push_with_overflow(&mut self, value: T) -> Option<T> {
        unsafe { self.push_with_overflow_impl(value) }
    }
}

impl<T: Copy + Debug, Ptr: GenericPointer + Debug> MetaQueue<T, Ptr> {
    /// Returns a copy of the element stored at index. The index is starting by 0 for the first
    /// element until [`Queue::len()`].
    ///
    /// # Safety
    ///
    ///   * Must satisfy `index` < [`Queue::len()`]
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
    /// element until [`Queue::len()`]queue_memory
    pub fn get(&self, index: usize) -> T {
        if self.len() <= index {
            fatal_panic!(from self, "Unable to copy content since the index {} is out of range.", index);
        }

        unsafe { self.get_unchecked(index) }
    }
}

impl<T> RelocatableContainer for RelocatableQueue<T> {
    unsafe fn new_uninit(capacity: usize) -> Self {
        Self {
            data_ptr: RelocatablePointer::new_uninit(),
            start: 0,
            len: 0,
            capacity,
            is_initialized: IoxAtomicBool::new(false),
            _phantom_data: PhantomData,
        }
    }

    unsafe fn init<Allocator: BaseAllocator>(
        &mut self,
        allocator: &Allocator,
    ) -> Result<(), AllocationError> {
        if self
            .is_initialized
            .load(core::sync::atomic::Ordering::Relaxed)
        {
            fatal_panic!(
                from "Queue::init()",
                "Memory already initialized. Initializing it twice may lead to undefined behavior."
            );
        }

        self.data_ptr.init(fail!(from "Queue::init", when allocator
             .allocate(Layout::from_size_align_unchecked(
                 core::mem::size_of::<T>() * self.capacity,
                 core::mem::align_of::<T>(),
             )), "Failed to initialize queue since the allocation of the data memory failed."
        ));
        self.is_initialized
            .store(true, core::sync::atomic::Ordering::Relaxed);

        Ok(())
    }

    fn memory_size(capacity: usize) -> usize {
        Self::const_memory_size(capacity)
    }
}

unsafe impl<T: ZeroCopySend> ZeroCopySend for RelocatableQueue<T> {}

impl<T> RelocatableQueue<T> {
    /// Returns the required memory size for a queue with a specified capacity
    pub const fn const_memory_size(capacity: usize) -> usize {
        unaligned_mem_size::<T>(capacity)
    }

    /// Removes all elements from the queue
    ///
    /// # Safety
    ///
    ///  * [`Queue::init()`] must have been called once before
    ///
    pub unsafe fn clear(&mut self) {
        self.clear_impl()
    }

    /// Returns a reference to the element from the beginning of the queue without removing it.
    /// If the queue is empty it returns [`None`].
    ///
    /// # Safety
    ///
    ///  * [`Queue::init()`] must have been called once before
    ///
    pub fn peek(&self) -> Option<&T> {
        unsafe { self.peek_impl() }
    }

    /// Returns a mutable reference to the element from the beginning of the queue without removing it.
    /// If the queue is empty it returns [`None`].
    ///
    /// # Safety
    ///
    ///  * [`Queue::init()`] must have been called once before
    ///
    pub fn peek_mut(&mut self) -> Option<&mut T> {
        unsafe { self.peek_mut_impl() }
    }

    /// Removes the element from the beginning of the queue. If the queue is empty it returns [`None`].
    ///
    /// # Safety
    ///
    ///  * [`Queue::init()`] must have been called once before
    ///
    pub unsafe fn pop(&mut self) -> Option<T> {
        self.pop_impl()
    }

    /// Adds an element at the end of the queue. If the queue is full it returns false, otherwise true.
    ///
    /// # Safety
    ///
    ///  * [`Queue::init()`] must have been called once before
    ///
    pub unsafe fn push(&mut self, value: T) -> bool {
        self.push_impl(value)
    }

    /// Adds an element at the end of the queue. If the queue is full it returns the oldest element,
    /// otherwise [`None`].
    ///
    /// # Safety
    ///
    ///  * [`Queue::init()`] must have been called once before
    ///
    pub unsafe fn push_with_overflow(&mut self, value: T) -> Option<T> {
        self.push_with_overflow_impl(value)
    }
}

impl<T, Ptr: GenericPointer> MetaQueue<T, Ptr> {
    #[inline(always)]
    fn verify_init(&self, source: &str) {
        debug_assert!(
                self.is_initialized
                    .load(core::sync::atomic::Ordering::Relaxed),
                "From: MetaQueue<{}>::{}, Undefined behavior - the object was not initialized with 'init' before.",
                core::any::type_name::<T>(), source
            );
    }

    /// Returns true if the queue is empty, otherwise false
    pub fn is_empty(&self) -> bool {
        self.len == 0
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

    pub(crate) unsafe fn clear_impl(&mut self) {
        while self.pop_impl().is_some() {}
    }

    pub(crate) unsafe fn peek_mut_impl(&mut self) -> Option<&mut T> {
        self.verify_init("peek_mut()");

        if self.is_empty() {
            return None;
        }

        let index = (self.start - self.len) % self.capacity;

        Some((*self.data_ptr.as_mut_ptr().add(index)).assume_init_mut())
    }

    pub(crate) unsafe fn peek_impl(&self) -> Option<&T> {
        self.verify_init("peek()");

        if self.is_empty() {
            return None;
        }

        let index = (self.start - self.len) % self.capacity;

        Some((*self.data_ptr.as_ptr().add(index)).assume_init_ref())
    }

    pub(crate) unsafe fn pop_impl(&mut self) -> Option<T> {
        self.verify_init("pop()");

        if self.is_empty() {
            return None;
        }

        let index = (self.start - self.len) % self.capacity;
        self.len -= 1;
        let value = core::mem::replace(
            &mut *self.data_ptr.as_mut_ptr().add(index),
            MaybeUninit::uninit(),
        );
        Some(value.assume_init())
    }

    pub(crate) unsafe fn push_impl(&mut self, value: T) -> bool {
        self.verify_init("push()");

        if self.len == self.capacity {
            return false;
        }

        self.unchecked_push(value);
        true
    }

    pub(crate) unsafe fn push_with_overflow_impl(&mut self, value: T) -> Option<T> {
        self.verify_init("push_with_overflow()");

        let overridden_value = if self.len() == self.capacity() {
            self.pop_impl()
        } else {
            None
        };

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

impl<T, Ptr: GenericPointer> Drop for MetaQueue<T, Ptr> {
    fn drop(&mut self) {
        if self
            .is_initialized
            .load(core::sync::atomic::Ordering::Relaxed)
        {
            unsafe { self.clear_impl() }
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

unsafe impl<T: ZeroCopySend, const CAPACITY: usize> ZeroCopySend for FixedSizeQueue<T, CAPACITY> {}

impl<T, const CAPACITY: usize> PlacementDefault for FixedSizeQueue<T, CAPACITY> {
    unsafe fn placement_default(ptr: *mut Self) {
        let state_ptr = core::ptr::addr_of_mut!((*ptr).state);
        state_ptr.write(RelocatableQueue::new_uninit(CAPACITY));

        let allocator = BumpAllocator::new((*ptr)._data.as_mut_ptr().cast());
        (*ptr)
            .state
            .init(&allocator)
            .expect("All required memory is preallocated.");
    }
}

impl<T, const CAPACITY: usize> Default for FixedSizeQueue<T, CAPACITY> {
    fn default() -> Self {
        let mut new_self = Self {
            state: unsafe { RelocatableQueue::new_uninit(CAPACITY) },
            _data: unsafe { MaybeUninit::uninit().assume_init() },
        };

        let allocator = BumpAllocator::new(new_self._data.as_mut_ptr().cast());
        unsafe {
            new_self
                .state
                .init(&allocator)
                .expect("All required memory is preallocated.")
        };

        new_self
    }
}

unsafe impl<T: Send, const CAPACITY: usize> Send for FixedSizeQueue<T, CAPACITY> {}

impl<T, const CAPACITY: usize> FixedSizeQueue<T, CAPACITY> {
    /// Creates a new queue.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns true if the queue is empty, otherwise false
    pub fn is_empty(&self) -> bool {
        self.state.is_empty()
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

    /// Removes all elements from the queue
    pub fn clear(&mut self) {
        unsafe { self.state.clear_impl() }
    }

    /// Returns a reference to the element from the beginning of the queue without removing it.
    /// If the queue is empty it returns [`None`].
    pub fn peek(&self) -> Option<&T> {
        unsafe { self.state.peek_impl() }
    }

    /// Returns a mutable reference to the element from the beginning of the queue without removing it.
    /// If the queue is empty it returns [`None`].
    pub fn peek_mut(&mut self) -> Option<&mut T> {
        unsafe { self.state.peek_mut_impl() }
    }

    /// Removes the element from the beginning of the queue. If the queue is empty it returns [`None`].
    pub fn pop(&mut self) -> Option<T> {
        unsafe { self.state.pop_impl() }
    }

    /// Adds an element at the end of the queue. If the queue is full it returns false, otherwise true.
    pub fn push(&mut self, value: T) -> bool {
        unsafe { self.state.push_impl(value) }
    }

    /// Adds an element at the end of the queue. If the queue is full it returns the oldest element,
    /// otherwise [`None`].
    pub fn push_with_overflow(&mut self, value: T) -> Option<T> {
        unsafe { self.state.push_with_overflow_impl(value) }
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
