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

//! A simplistic **threadsafe** and **lock-free** container which contains element in an
//! unspecified order.
//!
//! Elements can be added with [`Container::add()`]. The method returns an [`UniqueIndex`] which
//! removes the element as soon as it goes out of scope. Elements are stored under a fixed index
//! which is guaranteed to never change. The index can be acquired inside the callback of
//! [`ContainerState::for_each()`].
//!
//! To iterate/acquire all container elements
//! a [`ContainerState`] has to be created with [`Container::get_state()`] and can be updated with
//! [`Container::update_state()`].
//!
//! # Example
//!
//! ```
//! use iceoryx2_bb_lock_free::mpmc::container::*;
//!
//! const CAPACITY: usize = 139;
//! let container = FixedSizeContainer::<u32, CAPACITY>::new();
//! let mut stored_indices = vec![];
//!
//! match unsafe { container.add(1234567) } {
//!     Some(index) => stored_indices.push(index),
//!     None => println!("container is full"),
//! };
//!
//! let mut state = container.get_state();
//! state.for_each(|index: u32, value: &u32| println!("index: {}, value: {}", index, value));
//!
//! stored_indices.clear();
//!
//! if unsafe { container.update_state(&mut state) } {
//!     println!("container state has changed");
//!     state.for_each(|index: u32, value: &u32| println!("index: {}, value: {}", index, value));
//! }
//! ```

use iceoryx2_bb_elementary::allocator::AllocationError;
use iceoryx2_bb_elementary::pointer_trait::PointerTrait;
use iceoryx2_bb_elementary::relocatable_container::RelocatableContainer;
use iceoryx2_bb_elementary::relocatable_ptr::RelocatablePointer;
use iceoryx2_bb_elementary::unique_id::UniqueId;
use iceoryx2_bb_elementary::{allocator::BaseAllocator, math::align_to};
use iceoryx2_bb_log::{fail, fatal_panic};

use crate::mpmc::unique_index_set::*;
use std::alloc::Layout;
use std::fmt::Debug;
use std::sync::atomic::AtomicU64;
use std::{
    cell::UnsafeCell,
    mem::MaybeUninit,
    sync::atomic::{AtomicBool, Ordering},
};

/// A handle that corresponds to an element inside the [`Container`]. Will be acquired when using
/// [`Container::add()`] and can be released with [`Container::remove()`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContainerHandle {
    index: u32,
    container_id: u64,
}

impl ContainerHandle {
    /// Returns the underlying index of the container handle
    pub fn index(&self) -> u32 {
        self.index
    }
}

/// Contains a state of the [`Container`]. Can be created with [`Container::get_state()`] and
/// updated when the [`Container`] has changed with [`Container::update_state()`].
#[derive(Debug)]
pub struct ContainerState<T: Copy + Debug> {
    container_id: u64,
    current_change_counter: u64,
    data: Vec<MaybeUninit<T>>,
    active_index: Vec<u64>,
}

impl<T: Copy + Debug> ContainerState<T> {
    fn new(container_id: u64, capacity: usize) -> Self {
        Self {
            container_id,
            current_change_counter: 0,
            data: vec![MaybeUninit::uninit(); capacity],
            active_index: vec![0; capacity],
        }
    }

    /// Iterates over all elements and calls the callback for each of them, providing the
    /// index of the element and a reference to the underlying value.
    /// **Note:** The index of a value never changes as long as it is stored inside the container.
    ///
    /// ```
    /// use iceoryx2_bb_lock_free::mpmc::container::*;
    ///
    /// let container = FixedSizeContainer::<u128, 128>::new();
    ///
    /// let mut state = container.get_state();
    /// state.for_each(|index: u32, value: &u128| println!("index: {}, value: {}", index, value));
    /// ```
    pub fn for_each<F: FnMut(u32, &T)>(&self, mut callback: F) {
        for i in 0..self.data.len() {
            if self.active_index[i] % 2 == 1 {
                callback(i as _, unsafe { self.data[i].assume_init_ref() });
            }
        }
    }
}

/// A **threadsafe** and **lock-free** runtime fixed size container. The compile time fixed size
/// container is called [`FixedSizeContainer`].
///
/// **Restriction:** T is not allowed to implement [`Drop`], it must be trivially dropable!
#[repr(C)]
#[derive(Debug)]
pub struct Container<T: Copy + Debug> {
    // must be first member, otherwise the offset calculations fail
    active_index_ptr: RelocatablePointer<AtomicU64>,
    data_ptr: RelocatablePointer<UnsafeCell<MaybeUninit<T>>>,
    capacity: usize,
    change_counter: AtomicU64,
    is_memory_initialized: AtomicBool,
    container_id: UniqueId,
    // must be the last member, since it is a relocatable container as well and then the offset
    // calculations would again fail
    index_set: UniqueIndexSet,
}

unsafe impl<T: Copy + Debug> Send for Container<T> {}
unsafe impl<T: Copy + Debug> Sync for Container<T> {}

impl<T: Copy + Debug> RelocatableContainer for Container<T> {
    unsafe fn new_uninit(capacity: usize) -> Self {
        let distance_to_active_index =
            (std::mem::size_of::<Self>() + UniqueIndexSet::memory_size(capacity)) as isize;
        Self {
            container_id: UniqueId::new(),
            active_index_ptr: RelocatablePointer::new(distance_to_active_index),
            data_ptr: RelocatablePointer::new(align_to::<MaybeUninit<T>>(
                distance_to_active_index as usize + capacity * std::mem::size_of::<AtomicBool>(),
            ) as isize),
            capacity,
            change_counter: AtomicU64::new(0),
            index_set: UniqueIndexSet::new_uninit(capacity),
            is_memory_initialized: AtomicBool::new(false),
        }
    }

    unsafe fn init<Allocator: BaseAllocator>(
        &self,
        allocator: &Allocator,
    ) -> Result<(), AllocationError> {
        if self.is_memory_initialized.load(Ordering::Relaxed) {
            fatal_panic!(from self, "Memory already initialized. Initializing it twice may lead to undefined behavior.");
        }
        let msg = "Unable to initialize";

        fail!(from self, when self.index_set.init(allocator),
            "{} since the underlying UniqueIndexSet could not be initialized", msg);

        self.active_index_ptr.init(fail!(from self, when allocator.allocate(Layout::from_size_align_unchecked(
                        std::mem::size_of::<AtomicU64>() * self.capacity,
                        std::mem::align_of::<AtomicU64>())), "{} since the allocation of the active index memory failed.",
                msg));
        self.data_ptr.init(
            fail!(from self, when allocator.allocate(Layout::from_size_align_unchecked(
                    std::mem::size_of::<T>() * self.capacity,
                    std::mem::align_of::<T>())),
                "{} since the allocation of the data memory failed.", msg
            ),
        );

        for i in 0..self.capacity {
            (self.active_index_ptr.as_ptr() as *mut AtomicU64)
                .add(i)
                .write(AtomicU64::new(0));
            (self.data_ptr.as_ptr() as *mut UnsafeCell<MaybeUninit<T>>)
                .add(i)
                .write(UnsafeCell::new(MaybeUninit::uninit()));
        }
        self.is_memory_initialized.store(true, Ordering::Relaxed);

        Ok(())
    }

    unsafe fn new(capacity: usize, distance_to_data: isize) -> Self {
        let unique_index_set_distance = distance_to_data
            - align_to::<u32>(std::mem::size_of::<Container<T>>()) as isize
            + align_to::<u32>(std::mem::size_of::<UniqueIndexSet>()) as isize;

        let distance_to_active_index = align_to::<AtomicU64>(
            distance_to_data as usize + (std::mem::size_of::<u32>() * (capacity + 1)),
        ) as isize;
        let distance_to_container_data = align_to::<UnsafeCell<MaybeUninit<T>>>(
            distance_to_active_index as usize + (std::mem::size_of::<AtomicU64>() * capacity),
        ) as isize
            - std::mem::size_of::<RelocatablePointer<AtomicU64>>() as isize;

        Self {
            container_id: UniqueId::new(),
            active_index_ptr: RelocatablePointer::new(distance_to_active_index),
            data_ptr: RelocatablePointer::new(distance_to_container_data),
            capacity,
            change_counter: AtomicU64::new(0),
            index_set: UniqueIndexSet::new(capacity, unique_index_set_distance),
            is_memory_initialized: AtomicBool::new(true),
        }
    }

    fn memory_size(capacity: usize) -> usize {
        Self::const_memory_size(capacity)
    }
}

impl<T: Copy + Debug> Container<T> {
    fn verify_memory_initialization(&self, source: &str) {
        if !self.is_memory_initialized.load(Ordering::Relaxed) {
            fatal_panic!(from self, "Undefined behavior when calling \"{}\" and the object is not initialized with 'initialize_memory'.", source);
        }
    }

    /// Returns the required memory size of the data segment of the [`Container`].
    pub const fn const_memory_size(capacity: usize) -> usize {
        // UniqueIndexSet
        (std::mem::size_of::<u32>() * (capacity + 1) + std::mem::align_of::<u32>() - 1)
        //  ActiveIndexPtr
        + (std::mem::size_of::<AtomicU64>() * capacity + std::mem::align_of::<u64>() - 1)
        // data ptr
        + (std::mem::size_of::<T>() * capacity + std::mem::align_of::<T>() - 1)
    }

    /// Returns the capacity of the container.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Returns the current len of the container
    pub fn len(&self) -> usize {
        self.index_set.borrowed_indices()
    }

    /// Returns true if the container is empty, otherwise false
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Adds a new element to the [`Container`]. If there is no more space available it returns
    /// [`None`], otherwise [`Some`] containing the the index value to the underlying element.
    ///
    /// # Safety
    ///
    ///  * Ensure that the either [`Container::new()`] was used or [`Container::init()`] was used
    ///     before calling this method
    ///  * Use [`Container::remove()`] to release the acquired index again. Otherwise, the
    ///     element will leak.
    ///
    pub unsafe fn add(&self, value: T) -> Option<ContainerHandle> {
        self.verify_memory_initialization("add");

        match self.index_set.acquire_raw_index() {
            Some(index) => {
                core::ptr::copy_nonoverlapping(
                    &value,
                    (*self.data_ptr.as_ptr().add(index as _)).get().cast(),
                    1,
                );

                //////////////////////////////////////
                // SYNC POINT with reading data values
                //////////////////////////////////////
                unsafe { &*self.active_index_ptr.as_ptr().add(index as _) }
                    .fetch_add(1, Ordering::Release);

                // MUST HAPPEN AFTER all other operations
                self.change_counter.fetch_add(1, Ordering::Release);
                Some(ContainerHandle {
                    index,
                    container_id: self.container_id.value(),
                })
            }
            None => None,
        }
    }

    /// Useful in IPC context when an application holding the UniqueIndex has died.
    ///
    /// # Safety
    ///
    ///  * Ensure that the either [`Container::new()`] was used or [`Container::init()`] was used
    ///     before calling this method
    ///  * Ensure that no one else possesses the [`UniqueIndex`] and the index was unrecoverable
    ///     lost
    ///  * Ensure that the `handle` was acquired by the same [`Container`]
    ///     with [`Container::add()`], otherwise the method will panic.
    ///
    /// **Important:** If the UniqueIndex still exists it causes double frees or freeing an index
    /// which was allocated afterwards
    ///
    pub unsafe fn remove(&self, handle: ContainerHandle) {
        self.verify_memory_initialization("remove_with_handle");
        if handle.container_id != self.container_id.value() {
            fatal_panic!(from self,
                "The ContainerHandle used as handle was not created by this Container instance.");
        }

        unsafe { &*self.active_index_ptr.as_ptr().add(handle.index as _) }
            .fetch_add(1, Ordering::Relaxed);
        self.index_set.release_raw_index(handle.index);

        // MUST HAPPEN AFTER all other operations
        self.change_counter.fetch_add(1, Ordering::Release);
    }

    /// Returns [`ContainerState`] which contains all elements of this container. Be aware that
    /// this state can be out of date as soon as it is returned from this function.
    ///
    /// # Safety
    ///
    ///  * Ensure that the either [`Container::new()`] was used or [`Container::init()`] was used
    ///     before calling this method
    ///
    pub unsafe fn get_state(&self) -> ContainerState<T> {
        self.verify_memory_initialization("get_state");

        let mut state = ContainerState::new(self.container_id.value(), self.capacity);
        self.update_state(&mut state);
        state
    }

    /// Syncs the [`ContainerState`] with the current state of the [`Container`]. If the state has
    /// changed it returns true, otherwise false.
    ///
    /// # Safety
    ///
    ///  * Ensure that the either [`Container::new()`] was used or [`Container::init()`] was used
    ///     before calling this method
    ///  * Ensure that the input argument `previous_state` was acquired by the same [`Container`]
    ///     with [`Container::get_state()`], otherwise the method will panic.
    ///
    pub unsafe fn update_state(&self, previous_state: &mut ContainerState<T>) -> bool {
        if previous_state.container_id != self.container_id.value() {
            fatal_panic!(from self,
                "The ContainerState used as previous_state was not created by this Container instance.");
        }

        // MUST HAPPEN BEFORE all other operations
        let current_change_counter = self.change_counter.load(Ordering::Acquire);

        if previous_state.current_change_counter == current_change_counter {
            return false;
        }

        // must be set once here, if the current_change_counter changes in the loop below
        // the previous_state is updated again with the next clone_state iteration
        previous_state.current_change_counter = current_change_counter;

        for i in 0..self.capacity {
            // go through here element by element and do not start the operation from the
            // beginning when the content has changed.
            // only copy single entries otherwise we encounter starvation since this is a
            // heavy-weight operation

            //////////////////////////////////////
            // SYNC POINT with reading data values
            //////////////////////////////////////
            let mut current_index_count =
                unsafe { (*self.active_index_ptr.as_ptr().add(i)).load(Ordering::Acquire) };

            loop {
                if current_index_count == previous_state.active_index[i] {
                    break;
                }

                previous_state.active_index[i] = current_index_count;

                if previous_state.active_index[i] % 2 == 1 {
                    core::ptr::copy_nonoverlapping(
                        (*self.data_ptr.as_ptr().add(i)).get(),
                        previous_state.data.as_mut_ptr().add(i),
                        1,
                    );
                }

                // MUST HAPPEN AFTER all other operations
                if let Err(count) = (*self.active_index_ptr.as_ptr().add(i)).compare_exchange(
                    current_index_count,
                    current_index_count,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                ) {
                    current_index_count = count
                } else {
                    break;
                }
            }
        }

        true
    }
}

/// A **threadsafe** and **lock-free** compile time fixed size container. The runtime time fixed size
/// container is called [`Container`].
///
/// **Restriction:** T is not allowed to implement [`Drop`], it must be trivially dropable!
// T is not allowed to implement Drop - must be trivially dropable
#[repr(C)]
#[derive(Debug)]
pub struct FixedSizeContainer<T: Copy + Debug, const CAPACITY: usize> {
    container: Container<T>,

    // DO NOT CHANGE MEMBER ORDER UniqueIndexSet variable data
    next_free_index: [UnsafeCell<u32>; CAPACITY],
    next_free_index_plus_one: UnsafeCell<u32>,

    // DO NOT CHANGE MEMBER ORDER actual Container variable data
    active_index: [AtomicU64; CAPACITY],
    data: [UnsafeCell<MaybeUninit<T>>; CAPACITY],
}

impl<T: Copy + Debug, const CAPACITY: usize> Default for FixedSizeContainer<T, CAPACITY> {
    fn default() -> Self {
        Self {
            container: unsafe {
                Container::new(
                    CAPACITY,
                    align_to::<u32>(std::mem::size_of::<Container<T>>()) as isize,
                )
            },
            next_free_index: core::array::from_fn(|i| UnsafeCell::new(i as u32 + 1)),
            next_free_index_plus_one: UnsafeCell::new(CAPACITY as u32 + 1),
            active_index: core::array::from_fn(|_| AtomicU64::new(0)),
            data: core::array::from_fn(|_| UnsafeCell::new(MaybeUninit::uninit())),
        }
    }
}

unsafe impl<T: Copy + Debug, const CAPACITY: usize> Send for FixedSizeContainer<T, CAPACITY> {}
unsafe impl<T: Copy + Debug, const CAPACITY: usize> Sync for FixedSizeContainer<T, CAPACITY> {}

impl<T: Copy + Debug, const CAPACITY: usize> FixedSizeContainer<T, CAPACITY> {
    /// Creates a new [`FixedSizeContainer`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the capacity of the container
    pub fn capacity(&self) -> usize {
        self.container.capacity()
    }

    /// Adds a new element to the [`FixedSizeContainer`]. If there is no more space available it returns
    /// [`None`], otherwise [`Some`] containing the the index value to the underlying element.
    ///
    /// ```
    /// use iceoryx2_bb_lock_free::mpmc::container::*;
    ///
    /// const CAPACITY: usize = 139;
    /// let container = FixedSizeContainer::<u128, CAPACITY>::new();
    ///
    /// match unsafe { container.add(1234567) } {
    ///     Some(index) => {
    ///         println!("added at index {:?}", index);
    ///         unsafe { container.remove(index) };
    ///     },
    ///     None => println!("container is full"),
    /// };
    ///
    /// ```
    ///
    /// # Safety
    ///
    ///  * Use [`FixedSizeContainer::remove()`] to release the acquired index again. Otherwise,
    ///     the element will leak.
    ///
    pub unsafe fn add(&self, value: T) -> Option<ContainerHandle> {
        self.container.add(value)
    }

    /// Useful in IPC context when an application holding the UniqueIndex has died.
    ///
    /// # Safety
    ///
    ///  * If the UniqueIndex still exists it causes double frees or freeing an index
    ///    which was allocated afterwards
    pub unsafe fn remove(&self, handle: ContainerHandle) {
        self.container.remove(handle)
    }

    /// Returns [`ContainerState`] which contains all elements of this container. Be aware that
    /// this state can be out of date as soon as it is returned from this function.
    pub fn get_state(&self) -> ContainerState<T> {
        unsafe { self.container.get_state() }
    }

    /// Syncs the [`ContainerState`] with the current state of the [`FixedSizeContainer`].
    /// If the state has changed it returns true, otherwise false.
    ///
    /// ```
    /// use iceoryx2_bb_lock_free::mpmc::container::*;
    ///
    /// let container = FixedSizeContainer::<u128, 128>::new();
    ///
    /// let mut state = container.get_state();
    ///
    /// if unsafe { container.update_state(&mut state) } {
    ///     println!("container has changed");
    /// } else {
    ///     println!("no container changes");
    /// }
    /// ```
    ///
    /// # Safety
    ///
    ///  * Ensure that the input argument `previous_state` was acquired by the same [`Container`]
    ///     with [`Container::get_state()`].
    ///
    pub unsafe fn update_state(&self, previous_state: &mut ContainerState<T>) -> bool {
        unsafe { self.container.update_state(previous_state) }
    }
}
