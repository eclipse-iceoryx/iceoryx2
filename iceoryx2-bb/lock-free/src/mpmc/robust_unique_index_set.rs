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

//! A **threadsafe**, **lock-free** and **robust** set of indices. In a multi-process context it keeps
//! a valid state, without index leaks, even when one process crashes. It provides to assign every
//! borrowed index an [`OwnerId`] which allows to recover other processes leaked indices from crashed
//! processes. Can be used as a building block for
//! allocators or lock-free containers.
//!
//! # Example
//!
//! ## Runtime fixed size RobustUniqueIndexSet
//!
//! ```
//! # extern crate iceoryx2_bb_loggers;
//!
//! use iceoryx2_bb_elementary::bump_allocator::*;
//! use iceoryx2_bb_lock_free::mpmc::robust_unique_index_set::*;
//! use iceoryx2_bb_lock_free::mpmc::unique_index_set_enums::*;
//! use iceoryx2_bb_elementary_traits::relocatable_container::*;
//!
//! const CAPACITY: usize = 128;
//! let mut memory = [0u8; RobustUniqueIndexSet::const_memory_size(CAPACITY)];
//! let allocator = BumpAllocator::new(memory.as_mut_ptr());
//!
//! let mut index_set = unsafe { RobustUniqueIndexSet::new_uninit(CAPACITY) };
//! unsafe { index_set.init(&allocator) }.expect("failed to allocate enough memory");
//!
//! let owner_id = OwnerId::new(313).unwrap(); // can be the PID + epoch for instance
//! let new_index = match unsafe { index_set.acquire(owner_id) } {
//!     Err(_) => panic!("Out of indices"),
//!     Ok(i) => i,
//! };
//!
//! println!("Acquired index {}", new_index);
//!
//! // return the index to the index set
//! unsafe { index_set.release(new_index, ReleaseMode::default()) };
//! ```
//!
//! ## Compile time FixedSizeUniqueIndexSet
//!
//! ```
//! # extern crate iceoryx2_bb_loggers;
//!
//! use iceoryx2_bb_lock_free::mpmc::robust_unique_index_set::*;
//! use iceoryx2_bb_lock_free::mpmc::unique_index_set_enums::*;
//!
//! const CAPACITY: usize = 128;
//!
//! let index_set = StaticRobustUniqueIndexSet::<CAPACITY>::new();
//!
//! let owner_id = OwnerId::new(313).unwrap(); // can be the PID + epoch for instance
//! let new_index = match index_set.acquire(owner_id) {
//!     Err(_) => panic!("Out of indices"),
//!     Ok(i) => i,
//! };
//!
//! println!("Acquired index {}", new_index);
//!
//! // return the index to the index set
//! index_set.release(new_index, ReleaseMode::default());
//! ```
//!
//! ## Recover indices from a dead process
//!
//! ```
//! # extern crate iceoryx2_bb_loggers;
//!
//! use iceoryx2_bb_lock_free::mpmc::robust_unique_index_set::*;
//! use iceoryx2_bb_lock_free::mpmc::unique_index_set_enums::*;
//!
//! const CAPACITY: usize = 128;
//!
//! let index_set = StaticRobustUniqueIndexSet::<CAPACITY>::new();
//! let owner_id_of_dead_process = OwnerId::new(313).unwrap();
//! index_set.recover(ReleaseMode::default(), |owner_id| {
//!     owner_id == owner_id_of_dead_process
//! });
//! ```

use core::alloc::Layout;
use iceoryx2_bb_concurrency::atomic::Ordering;
use iceoryx2_bb_concurrency::atomic::{AtomicBool, AtomicU64};
use iceoryx2_bb_elementary::bump_allocator::BumpAllocator;
use iceoryx2_bb_elementary::enum_gen;
use iceoryx2_bb_elementary::relocatable_ptr::{PointerTrait, RelocatablePointer};
use iceoryx2_bb_elementary_traits::allocator::AllocationError;
use iceoryx2_bb_elementary_traits::relocatable_container::RelocatableContainer;
use iceoryx2_log::{error, fail, fatal_panic};

use crate::mpmc::unique_index_set_enums::{
    ReleaseMode, ReleaseState, UniqueIndexCreationError, UniqueIndexSetAcquireFailure,
};

enum_gen! { OwnerIdNewError
  entry:
    InvalidOwnerValue
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct OwnerId(u64);

impl OwnerId {
    const EMPTY: OwnerId = OwnerId(0);

    pub fn new(value: u64) -> Result<Self, OwnerIdNewError> {
        if value == Self::EMPTY.0 {
            fail!(from "OwnerId::new()", with OwnerIdNewError::InvalidOwnerValue,
                "Invalid value for OwnerId. The OwnerId cannot be 0.");
        }

        Ok(Self(value))
    }
}

struct RepairState {
    generation_counter: u64,
    borrowed_indices: u64,
}

#[repr(C)]
#[derive(Debug)]
pub struct RobustUniqueIndexSet {
    data_ptr: RelocatablePointer<AtomicU64>,
    capacity: usize,
    is_memory_initialized: AtomicBool,
    borrowed_indices: AtomicU64,
    generation_counter: AtomicU64,
}

impl RelocatableContainer for RobustUniqueIndexSet {
    unsafe fn new_uninit(capacity: usize) -> Self {
        Self {
            data_ptr: unsafe { RelocatablePointer::new_uninit() },
            capacity,
            is_memory_initialized: AtomicBool::new(false),
            borrowed_indices: AtomicU64::new(0),
            generation_counter: AtomicU64::new(0),
        }
    }

    unsafe fn init<T: iceoryx2_bb_elementary_traits::allocator::BaseAllocator>(
        &mut self,
        allocator: &T,
    ) -> Result<(), AllocationError> {
        let msg = "Failed to initialize";
        if self.is_memory_initialized.load(Ordering::Relaxed) {
            fatal_panic!(from self,
                "Memory already initialized. Initializing it twice may lead to undefined behavior.")
        }

        let layout = match Layout::array::<AtomicU64>(self.capacity) {
            Ok(v) => v,
            Err(e) => {
                fail!(from self, with AllocationError::SizeTooLarge,
                    "{msg} since the provided capacity would exceed the maximum supported size. [{e:?}]");
            }
        };

        unsafe {
            self.data_ptr.init(fail!(from self,
                when allocator.allocate(layout),
                "Failed to initialize since the allocation of the data memory failed."))
        };

        for i in 0..self.capacity + 1 {
            unsafe {
                (self.data_ptr.as_ptr() as *mut AtomicU64)
                    .add(i)
                    .write(AtomicU64::new(0))
            };
        }

        self.is_memory_initialized.store(true, Ordering::Relaxed);

        Ok(())
    }

    fn memory_size(capacity: usize) -> usize {
        Self::const_memory_size(capacity)
    }
}

impl RobustUniqueIndexSet {
    #[inline(always)]
    fn verify_init(&self, source: &str) {
        debug_assert!(
            self.is_memory_initialized.load(Ordering::Relaxed),
            "Undefined behavior when calling RobustUniqueIndexSet::{source} and the object is not initialized."
        );
    }

    /// Returns if the [`RobustUniqueIndexSet`] is locked or not. If the set is locked no more indices
    /// can be borrowed.
    pub fn is_locked(&self) -> bool {
        self.generation_counter.load(Ordering::Relaxed) == u64::MAX
    }

    fn lock(&self) -> ReleaseState {
        if self.is_locked() {
            return ReleaseState::Locked;
        }

        loop {
            let state = self.repair_borrowed_indices();

            if state.borrowed_indices == 0 {
                match self.generation_counter.compare_exchange(
                    state.generation_counter,
                    u64::MAX,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => return ReleaseState::Locked,
                    Err(_) => continue,
                }
            } else {
                return ReleaseState::Unlocked;
            }
        }
    }

    /// Returns the current len.
    pub fn borrowed_indices(&self) -> usize {
        if self.is_locked() {
            0
        } else {
            self.borrowed_indices.load(Ordering::Relaxed) as _
        }
    }

    /// Returns the capacity of the [`RobustUniqueIndexSet`].
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// The compile time version of [`UniqueIndexSet::memory_size()`]
    pub const fn const_memory_size(capacity: usize) -> usize {
        core::mem::size_of::<AtomicU64>() * capacity + core::mem::align_of::<AtomicU64>() - 1
    }

    /// Acquires a new index. If the set does not contain any more indices it returns
    /// [`None`].
    ///
    /// # Example
    ///
    /// ```
    /// # extern crate iceoryx2_bb_loggers;
    ///
    /// use iceoryx2_bb_lock_free::mpmc::robust_unique_index_set::*;
    /// use iceoryx2_bb_lock_free::mpmc::unique_index_set_enums::*;
    ///
    /// const CAPACITY: usize = 128;
    ///
    /// let index_set = StaticRobustUniqueIndexSet::<CAPACITY>::new();
    ///
    /// let owner_id = OwnerId::new(313).unwrap(); // can be the PID + epoch for instance
    /// let new_index = match unsafe { index_set.acquire(owner_id) } {
    ///     Err(_) => panic!("Out of indices"),
    ///     Ok(i) => i,
    /// };
    ///
    /// println!("Acquired index {}", new_index);
    ///
    /// // return the index to the index set
    /// unsafe { index_set.release(new_index, ReleaseMode::Default) };
    /// ```
    ///
    /// # Safety
    ///
    /// * Ensure that [`RobustUniqueIndexSet::init()`] was called once.
    ///
    pub unsafe fn acquire(&self, owner_id: OwnerId) -> Result<usize, UniqueIndexSetAcquireFailure> {
        let msg = "Unable to acquire another index";
        self.verify_init("acquire()");

        loop {
            //////////////////////////////////////
            // SYNC POINT enforce order of:
            //   1. cell & borrowed_indices
            //   2. generation counter
            //////////////////////////////////////
            let current_generation_count = self.generation_counter.load(Ordering::Acquire);

            if current_generation_count == u64::MAX {
                fail!(from self, with UniqueIndexSetAcquireFailure::IsLocked,
                    "{msg} since the RobustUniqueIndexSet is locked.");
            }

            for n in 0..self.capacity {
                let cell = unsafe { &*self.data_ptr.as_ptr().add(n) };

                match cell.compare_exchange(
                    OwnerId::EMPTY.0,
                    owner_id.0,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => {
                        self.borrowed_indices.fetch_add(1, Ordering::Relaxed);

                        //////////////////////////////////////
                        // SYNC POINT enforce order of:
                        //   1. cell & borrowed_indices
                        //   2. generation counter
                        //////////////////////////////////////
                        if self.increment_generation_counter(Ordering::Release)
                            == ReleaseState::Locked
                        {
                            // revert the increment, not important, just for consistency
                            self.borrowed_indices.fetch_sub(1, Ordering::Relaxed);
                            cell.store(OwnerId::EMPTY.0, Ordering::Relaxed);
                            fail!(from self, with UniqueIndexSetAcquireFailure::IsLocked,
                                "{msg} since the RobustUniqueIndexSet is locked.");
                        }
                        return Ok(n);
                    }
                    Err(_) => continue,
                }
            }

            if current_generation_count == self.generation_counter.load(Ordering::Relaxed) {
                fail!(from self, with UniqueIndexSetAcquireFailure::OutOfIndices,
                    "{msg} since the RobustUniqueIndexSet is out of indices.");
            }
        }
    }

    /// Releases a raw index.
    ///
    /// # Example
    ///
    /// See [`RobustUniqueIndexSet::acquire_raw_index()`].
    ///
    /// # Safety
    ///
    ///  * Ensure that [`UniqueIndexSet::init()`] was called once.
    ///
    pub unsafe fn release(&self, index: usize, mode: ReleaseMode) -> ReleaseState {
        self.verify_init("release()");

        let cell = unsafe { &*self.data_ptr.as_ptr().add(index) };

        let cell_content = cell.load(Ordering::Relaxed);

        if cell_content == OwnerId::EMPTY.0 {
            error!(from self, "Release of the not acquired index {index} prevented." );
        } else {
            match cell.compare_exchange(
                cell_content,
                OwnerId::EMPTY.0,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => {
                    cell.store(OwnerId::EMPTY.0, Ordering::Relaxed);
                    self.borrowed_indices.fetch_sub(1, Ordering::Relaxed);

                    //////////////////////////////////////
                    // SYNC POINT enforce order of:
                    //   1. cell & borrowed_indices
                    //   2. generation counter
                    //////////////////////////////////////
                    self.increment_generation_counter(Ordering::Release);
                }
                Err(v) => {
                    if v == OwnerId::EMPTY.0 {
                        error!(from self,
                            "Prevented race in releasing the index {index} which would have resulted in a double free.");
                    } else {
                        error!(from self,
                            "It seems like the index {index} was modified while releasing it. This could indicate a corrupted system.");
                    }
                }
            }
        }

        if mode == ReleaseMode::LockIfLastIndex {
            self.lock()
        } else {
            ReleaseState::Unlocked
        }
    }

    /// Recovers leaked indices for which the predicate with the [`OwnerId`] returns true.
    ///
    /// # Safety
    ///
    /// * Ensure that [`RobustUniqueIndexSet::init()`] was called once.
    ///
    pub unsafe fn recover<F: FnMut(OwnerId) -> bool>(
        &self,
        mode: ReleaseMode,
        mut predicate: F,
    ) -> ReleaseState {
        self.verify_init("acquire()");

        for n in 0..self.capacity {
            let cell = unsafe { &*self.data_ptr.as_ptr().add(n) };
            let value = cell.load(Ordering::Relaxed);

            if value == OwnerId::EMPTY.0 {
                continue;
            }

            if predicate(OwnerId(value))
                && cell
                    .compare_exchange(
                        value,
                        OwnerId::EMPTY.0,
                        Ordering::Relaxed,
                        Ordering::Relaxed,
                    )
                    .is_ok()
            {
                //////////////////////////////////////
                // SYNC POINT enforce order of:
                //   1. cell
                //   2. generation counter
                //////////////////////////////////////
                if self.increment_generation_counter(Ordering::Release) == ReleaseState::Locked {
                    return ReleaseState::Locked;
                }

                if mode == ReleaseMode::LockIfLastIndex {
                    self.lock();
                } else {
                    // must be called here explicitly since self.lock() calls this
                    // already
                    self.repair_borrowed_indices();
                }
            }
        }

        match self.is_locked() {
            true => ReleaseState::Locked,
            false => ReleaseState::Unlocked,
        }
    }

    fn increment_generation_counter(&self, ordering: Ordering) -> ReleaseState {
        let mut current_generation_count = self.generation_counter.load(Ordering::Relaxed);
        loop {
            if current_generation_count == u64::MAX {
                return ReleaseState::Locked;
            }

            match self.generation_counter.compare_exchange(
                current_generation_count,
                current_generation_count + 1,
                ordering,
                Ordering::Relaxed,
            ) {
                Ok(_) => return ReleaseState::Unlocked,
                Err(v) => current_generation_count = v,
            }
        }
    }

    fn repair_borrowed_indices(&self) -> RepairState {
        loop {
            //////////////////////////////////////
            // SYNC POINT enforce order of:
            //   1. cell
            //   2. generation counter
            //////////////////////////////////////
            let current_generation_count = self.generation_counter.load(Ordering::Acquire);

            if current_generation_count == u64::MAX {
                return RepairState {
                    generation_counter: current_generation_count,
                    borrowed_indices: 0,
                };
            }

            let mut count = 0;
            for n in 0..self.capacity {
                let cell = unsafe { &*self.data_ptr.as_ptr().add(n) };
                if cell.load(Ordering::Relaxed) != OwnerId::EMPTY.0 {
                    count += 1;
                }
            }

            self.borrowed_indices.store(count, Ordering::Relaxed);

            //////////////////////////////////////
            // SYNC POINT enforce order of:
            //   1. borrowed_indices
            //   2. generation counter
            //////////////////////////////////////
            match self.generation_counter.compare_exchange(
                current_generation_count,
                current_generation_count + 1,
                Ordering::Release,
                Ordering::Relaxed,
            ) {
                Ok(_) => {
                    return RepairState {
                        generation_counter: current_generation_count + 1,
                        borrowed_indices: count,
                    };
                }
                Err(_) => continue,
            }
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct StaticRobustUniqueIndexSet<const CAPACITY: usize> {
    state: RobustUniqueIndexSet,
    cells: [AtomicU64; CAPACITY],
}

impl<const CAPACITY: usize> Default for StaticRobustUniqueIndexSet<CAPACITY> {
    fn default() -> Self {
        Self::new_with_reduced_capacity(CAPACITY).expect("Does not exceed supported capacity.")
    }
}

unsafe impl<const CAPACITY: usize> Sync for StaticRobustUniqueIndexSet<CAPACITY> {}
unsafe impl<const CAPACITY: usize> Send for StaticRobustUniqueIndexSet<CAPACITY> {}

impl<const CAPACITY: usize> StaticRobustUniqueIndexSet<CAPACITY> {
    /// Creates a new [`StaticRobustUniqueIndexSet`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new [`StaticRobustUniqueIndexSet`] where the capacity is reduced. If the capacity
    /// is greater than CAPACITY or zero it fails.
    pub fn new_with_reduced_capacity(capacity: usize) -> Result<Self, UniqueIndexCreationError> {
        let origin = "StaticRobustUniqueIndexSet::new_with_reduced_capacity()";
        let msg = "Unable to create new StaticRobustUniqueIndexSet";
        if capacity > CAPACITY {
            fail!(from origin,
                with UniqueIndexCreationError::ProvidedCapacityGreaterThanMaxCapacity,
                "{msg} since the provided capacity value {capacity} exceeds maximum supported capacity of {CAPACITY}");
        }

        if capacity == 0 {
            fail!(from origin, with UniqueIndexCreationError::ProvidedCapacityIsZero,
                "{msg} since the provided capacity value is zero.");
        }

        let mut new_self = Self {
            state: unsafe { RobustUniqueIndexSet::new_uninit(capacity) },
            cells: core::array::from_fn(|_| AtomicU64::new(0)),
        };

        let allocator = BumpAllocator::new(new_self.cells.as_mut_ptr().cast());
        unsafe {
            new_self
                .state
                .init(&allocator)
                .expect("All required memory is preallocated")
        };

        Ok(new_self)
    }

    /// Acquires a new index. If the set does not contain any more indices it returns
    /// [`None`].
    ///
    /// ```
    /// # extern crate iceoryx2_bb_loggers;
    ///
    /// use iceoryx2_bb_lock_free::mpmc::robust_unique_index_set::*;
    /// use iceoryx2_bb_lock_free::mpmc::unique_index_set_enums::*;
    ///
    /// const CAPACITY: usize = 128;
    ///
    /// let index_set = StaticRobustUniqueIndexSet::<CAPACITY>::new();
    ///
    /// let owner_id = OwnerId::new(313).unwrap(); // can be PID + epoch
    /// let new_index = match unsafe { index_set.acquire(owner_id) } {
    ///     Err(_) => panic!("Out of indices"),
    ///     Ok(i) => i,
    /// };
    ///
    /// println!("Acquired index {}", new_index);
    ///
    /// // return the index to the index set
    /// unsafe { index_set.release(new_index, ReleaseMode::Default) };
    /// ```
    pub fn acquire(&self, owner_id: OwnerId) -> Result<usize, UniqueIndexSetAcquireFailure> {
        unsafe { self.state.acquire(owner_id) }
    }

    /// Releases a raw index.
    pub fn release(&self, index: usize, mode: ReleaseMode) -> ReleaseState {
        unsafe { self.state.release(index, mode) }
    }

    /// See [`StaticRobustUniqueIndexSet::capacity()`]
    pub fn capacity(&self) -> usize {
        self.state.capacity()
    }

    /// See [`StaticRobustUniqueIndexSet::is_locked()`]
    pub fn is_locked(&self) -> bool {
        self.state.is_locked()
    }

    /// Returns the current len.
    pub fn borrowed_indices(&self) -> usize {
        self.state.borrowed_indices()
    }

    /// Recovers leaked indices for which the predicate with the [`OwnerId`] returns true.
    pub fn recover<F: FnMut(OwnerId) -> bool>(
        &self,
        mode: ReleaseMode,
        predicate: F,
    ) -> ReleaseState {
        unsafe { self.state.recover(mode, predicate) }
    }
}
