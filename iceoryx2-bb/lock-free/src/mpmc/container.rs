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
//! [`ContainerState::update()`].
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
//! match container.add(1234567) {
//!     Some(index) => stored_indices.push(index),
//!     None => println!("container is full"),
//! };
//!
//! let mut state = container.get_state();
//! state.for_each(|index: u32, value: &u32| println!("index: {}, value: {}", index, value));
//!
//! stored_indices.clear();
//!
//! unsafe { state.update() };
//! state.for_each(|index: u32, value: &u32| println!("index: {}, value: {}", index, value));
//! ```

use iceoryx2_bb_elementary::allocator::AllocationError;
use iceoryx2_bb_elementary::pointer_trait::PointerTrait;
use iceoryx2_bb_elementary::relocatable_container::RelocatableContainer;
use iceoryx2_bb_elementary::relocatable_ptr::RelocatablePointer;
use iceoryx2_bb_elementary::{allocator::BaseAllocator, math::align_to};
use iceoryx2_bb_log::{fail, fatal_panic};

use crate::mpmc::unique_index_set::*;
use std::alloc::Layout;
use std::fmt::Debug;
use std::{
    cell::UnsafeCell,
    mem::MaybeUninit,
    sync::atomic::{AtomicBool, Ordering},
};

/// Contains a state of the [`Container`]. Can be created with [`Container::get_state()`] and
/// updated when the [`Container`] has changed with [`ContainerState::update()`].
#[derive(Debug)]
pub struct ContainerState<'a, T: Copy + Debug> {
    container: &'a Container<T>,
    current_index_set_head: u64,
    data: Vec<MaybeUninit<(u32, T)>>,
    active_index: Vec<bool>,
}

impl<'a, T: Copy + Debug> ContainerState<'a, T> {
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
            if self.active_index[i] {
                let entry = unsafe { self.data[i].assume_init_ref() };
                callback(entry.0, &entry.1);
            }
        }
    }

    /// Syncs the state with the current state of the underlying [`Container`]. If the state has
    /// changed it returns true, otherwise false.
    ///
    /// ```
    /// use iceoryx2_bb_lock_free::mpmc::container::*;
    ///
    /// let container = FixedSizeContainer::<u128, 128>::new();
    ///
    /// let mut state = container.get_state();
    ///
    /// if unsafe { state.update() } {
    ///     println!("container has changed");
    /// } else {
    ///     println!("no container changes");
    /// }
    /// ```
    pub fn update(&mut self) -> bool {
        self.container.update_state(self)
    }
}

/// A **threadsafe** and **lock-free** runtime fixed size container. The compile time fixed size
/// container is called [`FixedSizeContainer`].
///
/// **Restriction:** T is not allowed to implement [`Drop`], it must be trivially dropable!
#[repr(C)]
#[derive(Debug)]
pub struct Container<T: Copy + Debug> {
    active_index_ptr: RelocatablePointer<AtomicBool>,
    data_ptr: RelocatablePointer<UnsafeCell<MaybeUninit<T>>>,
    capacity: usize,
    is_memory_initialized: AtomicBool,
    index_set: UniqueIndexSet,
}

unsafe impl<T: Copy + Debug> Send for Container<T> {}
unsafe impl<T: Copy + Debug> Sync for Container<T> {}

impl<T: Copy + Debug> RelocatableContainer for Container<T> {
    unsafe fn new_uninit(capacity: usize) -> Self {
        let distance_to_active_index =
            (std::mem::size_of::<Self>() + UniqueIndexSet::memory_size(capacity)) as isize;
        Self {
            active_index_ptr: RelocatablePointer::new(distance_to_active_index),
            data_ptr: RelocatablePointer::new(align_to::<MaybeUninit<T>>(
                distance_to_active_index as usize + capacity * std::mem::size_of::<AtomicBool>(),
            ) as isize),
            capacity,
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
                        std::mem::size_of::<AtomicBool>() * self.capacity,
                        std::mem::align_of::<AtomicBool>())), "{} since the allocation of the active index memory failed.",
                msg));
        self.data_ptr.init(
            fail!(from self, when allocator.allocate(Layout::from_size_align_unchecked(
                    std::mem::size_of::<T>() * self.capacity,
                    std::mem::align_of::<T>())),
                "{} since the allocation of the data memory failed.", msg
            ),
        );

        for i in 0..self.capacity {
            (self.active_index_ptr.as_ptr() as *mut AtomicBool)
                .add(i)
                .write(AtomicBool::new(false));
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

        let distance_to_active_index = align_to::<AtomicBool>(
            distance_to_data as usize + (std::mem::size_of::<u32>() * (capacity + 1)),
        ) as isize;
        let distance_to_container_data = align_to::<UnsafeCell<MaybeUninit<T>>>(
            distance_to_active_index as usize + (std::mem::size_of::<AtomicBool>() * capacity),
        ) as isize
            - std::mem::size_of::<RelocatablePointer<AtomicBool>>() as isize;

        Self {
            active_index_ptr: RelocatablePointer::new(distance_to_active_index),
            data_ptr: RelocatablePointer::new(distance_to_container_data),
            capacity,
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
        (std::mem::size_of::<u32>() * (capacity + 1))
            + ((std::mem::size_of::<AtomicBool>() + std::mem::size_of::<T>()) * capacity)
            + std::mem::align_of::<u32>()
            - 1
            + std::mem::align_of::<T>()
            - 1
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
    /// [`None`], otherwise [`Some`] containing the [`UniqueIndex`] to the underlying element.
    /// If the [`UniqueIndex`] goes out of scope the added element is removed.
    ///
    /// ```
    /// use iceoryx2_bb_lock_free::mpmc::container::*;
    ///
    /// const CAPACITY: usize = 139;
    /// let container = FixedSizeContainer::<u128, CAPACITY>::new();
    ///
    /// match container.add(1234567) {
    ///     Some(index) => println!("added at index {}", index.value()),
    ///     None => println!("container is full"),
    /// };
    /// ```
    ///
    /// # Safety
    ///
    ///  * Ensure that the either [`Container::new()`] was used or [`Container::init()`] was used
    ///     before calling this method
    ///
    pub unsafe fn add(&self, value: T) -> Option<UniqueIndex<'_>> {
        self.verify_memory_initialization("add");
        match self
            .index_set
            .acquire_with_additional_cleanup(|index: u32| {
                // set deactivate the active index to indicate that the value can be used again
                // requires that T does not implement drop
                unsafe { &*self.active_index_ptr.as_ptr().offset(index as isize) }
                    .store(false, Ordering::Relaxed);
            }) {
            Some(index) => {
                unsafe {
                    *(*self.data_ptr.as_ptr().offset(index.value() as isize)).get() =
                        MaybeUninit::new(value)
                };

                //////////////////////////////////////
                // SYNC POINT with reading data values
                //////////////////////////////////////
                unsafe {
                    &*self
                        .active_index_ptr
                        .as_ptr()
                        .offset(index.value() as isize)
                }
                .store(true, Ordering::Release);
                Some(index)
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
    ///
    /// **Important:** If the UniqueIndex still exists it causes double frees or freeing an index
    /// which was allocated afterwards
    ///
    pub unsafe fn remove_raw_index(&self, index: u32) {
        self.verify_memory_initialization("remove_raw_index");
        unsafe { &*self.active_index_ptr.as_ptr().offset(index as isize) }
            .store(false, Ordering::Relaxed);
        self.index_set.release_raw_index(index);
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

        let mut state = ContainerState {
            container: self,
            current_index_set_head: 0,
            data: vec![MaybeUninit::uninit(); self.capacity],
            active_index: vec![false; self.capacity],
        };
        self.update_state(&mut state);
        state
    }

    fn update_state(&self, previous_state: &mut ContainerState<T>) -> bool {
        let mut current_index_set_head = self.index_set.head.load(Ordering::Relaxed);

        if previous_state.current_index_set_head == current_index_set_head {
            return false;
        }

        // must be set once here, if the current_index_set_head changes in the loop below
        // the previous_state is updated again with the next clone_state iteration
        previous_state.current_index_set_head = current_index_set_head;

        for i in 0..self.capacity {
            // go through here element by element and do not start the operation from the
            // beginning when the content has changed.
            // only copy single entries otherwise we encounter starvation since this is a
            // heavy-weight operation
            loop {
                //////////////////////////////////////
                // SYNC POINT with reading data values
                //////////////////////////////////////
                unsafe {
                    // TODO: can be implemented more efficiently when only elements are copied that
                    // have been changed
                    *previous_state.active_index.as_mut_ptr().add(i) =
                        (*self.active_index_ptr.as_ptr().add(i)).load(Ordering::Acquire)
                };
                if unsafe { *previous_state.active_index.as_mut_ptr().add(i) } {
                    unsafe {
                        *previous_state.data.as_mut_ptr().add(i) = MaybeUninit::new((
                            i as u32,
                            *(*(*self.data_ptr.as_ptr().add(i)).get()).assume_init_ref(),
                        ))
                    };
                }

                let new_current_index_set_head = self.index_set.head.load(Ordering::Relaxed);
                if new_current_index_set_head == current_index_set_head {
                    break;
                }

                current_index_set_head = new_current_index_set_head;
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
    state: Container<T>,

    // DO NOT CHANGE MEMBER ORDER UniqueIndexSet variable data
    next_free_index: [UnsafeCell<u32>; CAPACITY],
    next_free_index_plus_one: UnsafeCell<u32>,

    // DO NOT CHANGE MEMBER ORDER actual Container variable data
    active_index: [AtomicBool; CAPACITY],
    data: [UnsafeCell<MaybeUninit<T>>; CAPACITY],
}

impl<T: Copy + Debug, const CAPACITY: usize> Default for FixedSizeContainer<T, CAPACITY> {
    fn default() -> Self {
        Self {
            state: unsafe {
                Container::new(
                    CAPACITY,
                    align_to::<u32>(std::mem::size_of::<Container<T>>()) as isize,
                )
            },
            next_free_index: core::array::from_fn(|i| UnsafeCell::new(i as u32 + 1)),
            next_free_index_plus_one: UnsafeCell::new(CAPACITY as u32 + 1),
            active_index: core::array::from_fn(|_| AtomicBool::new(false)),
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
        self.state.capacity()
    }

    /// Adds a new element to the [`FixedSizeContainer`]. If there is no more space available it returns
    /// [`None`], otherwise [`Some`] containing the [`UniqueIndex`] to the underlying element.
    /// If the [`UniqueIndex`] goes out of scope the added element is removed.
    ///
    /// ```
    /// use iceoryx2_bb_lock_free::mpmc::container::*;
    ///
    /// const CAPACITY: usize = 139;
    /// let container = FixedSizeContainer::<u128, CAPACITY>::new();
    ///
    /// match container.add(1234567) {
    ///     Some(index) => println!("added at index {}", index.value()),
    ///     None => println!("container is full"),
    /// };
    /// ```
    pub fn add(&self, value: T) -> Option<UniqueIndex<'_>> {
        unsafe { self.state.add(value) }
    }

    /// Useful in IPC context when an application holding the UniqueIndex has died.
    ///
    /// # Safety
    ///
    ///  * If the UniqueIndex still exists it causes double frees or freeing an index
    ///    which was allocated afterwards
    pub unsafe fn remove_raw_index(&self, index: u32) {
        self.state.remove_raw_index(index)
    }

    /// Returns [`ContainerState`] which contains all elements of this container. Be aware that
    /// this state can be out of date as soon as it is returned from this function.
    pub fn get_state(&self) -> ContainerState<T> {
        unsafe { self.state.get_state() }
    }
}
