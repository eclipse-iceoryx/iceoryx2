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
//!     Ok(index) => stored_indices.push(index),
//!     Err(_) => println!("container is full"),
//! };
//!
//! let mut state = container.get_state();
//! state.for_each(|handle: ContainerHandle, value: &u32| {
//!     println!("handle: {:?}, value: {}", handle, value);
//!     CallbackProgression::Continue
//! });
//!
//! stored_indices.clear();
//!
//! if unsafe { container.update_state(&mut state) } {
//!     println!("container state has changed");
//!     state.for_each(|handle: ContainerHandle, value: &u32| {
//!         println!("handle: {:?}, value: {}", handle, value);
//!         CallbackProgression::Continue
//!     });
//! }
//! ```

pub use crate::mpmc::unique_index_set::ReleaseMode;
use iceoryx2_bb_elementary::bump_allocator::BumpAllocator;
pub use iceoryx2_bb_elementary::CallbackProgression;

use iceoryx2_bb_elementary::math::align_to;
use iceoryx2_bb_elementary::math::unaligned_mem_size;
use iceoryx2_bb_elementary::relocatable_ptr::RelocatablePointer;
use iceoryx2_bb_elementary::unique_id::UniqueId;
use iceoryx2_bb_elementary_traits::allocator::AllocationError;
use iceoryx2_bb_elementary_traits::allocator::BaseAllocator;
use iceoryx2_bb_elementary_traits::pointer_trait::PointerTrait;
use iceoryx2_bb_elementary_traits::relocatable_container::RelocatableContainer;
use iceoryx2_bb_log::{fail, fatal_panic};
use iceoryx2_pal_concurrency_sync::iox_atomic::{IoxAtomicBool, IoxAtomicU64};

use crate::mpmc::unique_index_set::*;
use core::alloc::Layout;
use core::fmt::Debug;
use core::{cell::UnsafeCell, mem::MaybeUninit, sync::atomic::Ordering};

/// States the reason why an element could not be added to the [`Container`]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerAddFailure {
    /// The new element would exceed the maximum [`Container::capacity()`].
    OutOfSpace,
    /// The last element was removed from the [`Container`] with the option
    /// [`ReleaseMode::LockIfLastIndex`] which prevents adding new elements when the [`Container`]
    /// reached the [`Container::is_empty()`] state.
    IsLocked,
}

impl From<UniqueIndexSetAcquireFailure> for ContainerAddFailure {
    fn from(value: UniqueIndexSetAcquireFailure) -> Self {
        match value {
            UniqueIndexSetAcquireFailure::IsLocked => ContainerAddFailure::IsLocked,
            UniqueIndexSetAcquireFailure::OutOfIndices => ContainerAddFailure::OutOfSpace,
        }
    }
}

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
    /// state.for_each(|handle: ContainerHandle, value: &u128| {
    ///     println!("handle: {:?}, value: {}", handle, value);
    ///     CallbackProgression::Continue
    /// });
    /// ```
    pub fn for_each<F: FnMut(ContainerHandle, &T) -> CallbackProgression>(&self, mut callback: F) {
        for i in 0..self.data.len() {
            if self.active_index[i] % 2 == 1
                && callback(
                    ContainerHandle {
                        index: i as _,
                        container_id: self.container_id,
                    },
                    unsafe { self.data[i].assume_init_ref() },
                ) == CallbackProgression::Stop
            {
                return;
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
    active_index_ptr: RelocatablePointer<IoxAtomicU64>,
    data_ptr: RelocatablePointer<UnsafeCell<MaybeUninit<T>>>,
    capacity: usize,
    change_counter: IoxAtomicU64,
    is_initialized: IoxAtomicBool,
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
            (core::mem::size_of::<Self>() + UniqueIndexSet::memory_size(capacity)) as isize;
        Self {
            container_id: UniqueId::new(),
            active_index_ptr: RelocatablePointer::new(distance_to_active_index),
            data_ptr: RelocatablePointer::new(align_to::<MaybeUninit<T>>(
                distance_to_active_index as usize
                    + capacity * core::mem::size_of::<IoxAtomicBool>(),
            ) as isize),
            capacity,
            change_counter: IoxAtomicU64::new(0),
            index_set: UniqueIndexSet::new_uninit(capacity),
            is_initialized: IoxAtomicBool::new(false),
        }
    }

    unsafe fn init<Allocator: BaseAllocator>(
        &mut self,
        allocator: &Allocator,
    ) -> Result<(), AllocationError> {
        if self.is_initialized.load(Ordering::Relaxed) {
            fatal_panic!(from self, "Memory already initialized. Initializing it twice may lead to undefined behavior.");
        }
        let msg = "Unable to initialize";

        fail!(from self, when self.index_set.init(allocator),
            "{} since the underlying UniqueIndexSet could not be initialized", msg);

        self.active_index_ptr.init(fail!(from self, when allocator.allocate(Layout::from_size_align_unchecked(
                        core::mem::size_of::<IoxAtomicU64>() * self.capacity,
                        core::mem::align_of::<IoxAtomicU64>())), "{} since the allocation of the active index memory failed.",
                msg));
        self.data_ptr.init(
            fail!(from self, when allocator.allocate(Layout::from_size_align_unchecked(
                    core::mem::size_of::<T>() * self.capacity,
                    core::mem::align_of::<T>())),
                "{} since the allocation of the data memory failed.", msg
            ),
        );

        for i in 0..self.capacity {
            (self.active_index_ptr.as_ptr() as *mut IoxAtomicU64)
                .add(i)
                .write(IoxAtomicU64::new(0));
            (self.data_ptr.as_ptr() as *mut UnsafeCell<MaybeUninit<T>>)
                .add(i)
                .write(UnsafeCell::new(MaybeUninit::uninit()));
        }
        self.is_initialized.store(true, Ordering::Relaxed);

        Ok(())
    }

    fn memory_size(capacity: usize) -> usize {
        Self::const_memory_size(capacity)
    }
}

impl<T: Copy + Debug> Container<T> {
    #[inline(always)]
    fn verify_init(&self, source: &str) {
        debug_assert!(self.is_initialized.load(Ordering::Relaxed),
            "Undefined behavior when calling Container<{}>::{} and the object is not initialized with 'init'.",
            core::any::type_name::<T>(), source);
    }

    /// Returns the required memory size of the data segment of the [`Container`].
    pub const fn const_memory_size(capacity: usize) -> usize {
        UniqueIndexSet::const_memory_size(capacity)
        //  ActiveIndexPtr
        + unaligned_mem_size::<IoxAtomicU64>(capacity)
        // data ptr
        + unaligned_mem_size::<T>(capacity)
    }

    /// Returns the capacity of the container.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Returns true if the container is locked, otherwise false.
    /// If the [`Container`] is locked no more elements can be added to it.
    pub fn is_locked(&self) -> bool {
        self.index_set.is_locked()
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
    ///  * Ensure that [`Container::init()`] was called before calling this method
    ///  * Use [`Container::remove()`] to release the acquired index again. Otherwise, the
    ///    element will leak.
    ///
    pub unsafe fn add(&self, value: T) -> Result<ContainerHandle, ContainerAddFailure> {
        self.verify_init("add()");

        let index = self.index_set.acquire_raw_index()?;
        core::ptr::copy_nonoverlapping(
            &value,
            (*self.data_ptr.as_ptr().add(index as _)).get().cast(),
            1,
        );

        //////////////////////////////////////
        // SYNC POINT with reading data values
        //////////////////////////////////////
        unsafe { &*self.active_index_ptr.as_ptr().add(index as _) }.fetch_add(1, Ordering::Release);

        // MUST HAPPEN AFTER all other operations
        self.change_counter.fetch_add(1, Ordering::Release);
        Ok(ContainerHandle {
            index,
            container_id: self.container_id.value(),
        })
    }

    /// Useful in IPC context when an application holding the UniqueIndex has died.
    ///
    /// # Safety
    ///
    ///  * Ensure that [`Container::init()`] was called before calling this method
    ///  * Ensure that no one else possesses the [`UniqueIndex`] and the index was unrecoverable
    ///    lost
    ///  * Ensure that the `handle` was acquired by the same [`Container`]
    ///    with [`Container::add()`], otherwise the method will panic.
    ///
    /// **Important:** If the UniqueIndex still exists it causes double frees or freeing an index
    /// which was allocated afterwards
    ///
    pub unsafe fn remove(&self, handle: ContainerHandle, mode: ReleaseMode) -> ReleaseState {
        self.verify_init("remove()");
        debug_assert!(
            handle.container_id == self.container_id.value(),
            "The ContainerHandle used as handle was not created by this Container instance."
        );

        unsafe { &*self.active_index_ptr.as_ptr().add(handle.index as _) }
            .fetch_add(1, Ordering::Relaxed);
        let release_state = self.index_set.release_raw_index(handle.index, mode);

        // MUST HAPPEN AFTER all other operations
        self.change_counter.fetch_add(1, Ordering::Release);
        release_state
    }

    /// Returns [`ContainerState`] which contains all elements of this container. Be aware that
    /// this state can be out of date as soon as it is returned from this function.
    ///
    /// # Safety
    ///
    ///  * Ensure that [`Container::init()`] was called before calling this method
    ///
    pub unsafe fn get_state(&self) -> ContainerState<T> {
        self.verify_init("get_state()");

        let mut state = ContainerState::new(self.container_id.value(), self.capacity);
        self.update_state(&mut state);
        state
    }

    /// Syncs the [`ContainerState`] with the current state of the [`Container`]. If the state has
    /// changed it returns true, otherwise false.
    ///
    /// # Safety
    ///
    ///  * Ensure that [`Container::init()`] was called before calling this method
    ///  * Ensure that the input argument `previous_state` was acquired by the same [`Container`]
    ///    with [`Container::get_state()`], otherwise the method will panic.
    ///
    pub unsafe fn update_state(&self, previous_state: &mut ContainerState<T>) -> bool {
        debug_assert!(
            previous_state.container_id == self.container_id.value(),
            "The ContainerState used as previous_state was not created by this Container instance."
        );

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
    active_index: [IoxAtomicU64; CAPACITY],
    data: [UnsafeCell<MaybeUninit<T>>; CAPACITY],
}

impl<T: Copy + Debug, const CAPACITY: usize> Default for FixedSizeContainer<T, CAPACITY> {
    fn default() -> Self {
        let mut new_self = Self {
            container: unsafe { Container::new_uninit(CAPACITY) },
            next_free_index: core::array::from_fn(|i| UnsafeCell::new(i as u32 + 1)),
            next_free_index_plus_one: UnsafeCell::new(CAPACITY as u32 + 1),
            active_index: core::array::from_fn(|_| IoxAtomicU64::new(0)),
            data: core::array::from_fn(|_| UnsafeCell::new(MaybeUninit::uninit())),
        };

        let allocator = BumpAllocator::new(new_self.next_free_index.as_mut_ptr().cast());
        unsafe {
            new_self
                .container
                .init(&allocator)
                .expect("All required memory is preallocated.")
        };

        new_self
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

    /// Returns true if the container is empty, otherwise false
    pub fn is_empty(&self) -> bool {
        self.container.is_empty()
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
    ///     Ok(index) => {
    ///         println!("added at index {:?}", index);
    ///         unsafe { container.remove(index, ReleaseMode::Default) };
    ///     },
    ///     Err(_) => println!("container is full"),
    /// };
    ///
    /// ```
    ///
    /// # Safety
    ///
    ///  * Use [`FixedSizeContainer::remove()`] to release the acquired index again. Otherwise,
    ///    the element will leak.
    ///
    pub unsafe fn add(&self, value: T) -> Result<ContainerHandle, ContainerAddFailure> {
        self.container.add(value)
    }

    /// Useful in IPC context when an application holding the UniqueIndex has died.
    ///
    /// # Safety
    ///
    ///  * If the UniqueIndex still exists it causes double frees or freeing an index
    ///    which was allocated afterwards
    pub unsafe fn remove(&self, handle: ContainerHandle, mode: ReleaseMode) -> ReleaseState {
        self.container.remove(handle, mode)
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
    ///    with [`Container::get_state()`].
    ///
    pub unsafe fn update_state(&self, previous_state: &mut ContainerState<T>) -> bool {
        unsafe { self.container.update_state(previous_state) }
    }
}
