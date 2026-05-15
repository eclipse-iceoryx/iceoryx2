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
//! The key properties of the robust unique index set:
//!
//!  * contains unique indices from 0 to CAPACITY
//!  * indices can be acquired or released
//!  * when releasing the last index the user has the option to lock the index set which means that
//!    new indices can be never acquired again
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
//! unsafe { index_set.release(new_index, owner_id, ReleaseMode::default()) };
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
//! index_set.release(new_index, owner_id, ReleaseMode::default());
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
//! index_set.recover(ReleaseMode::default(),
//!     |owner_id, _index| {
//!         owner_id == owner_id_of_dead_process
//!     },
//!     |owner_id, index| {
//!         println!("recover index {index} from {owner_id:?}");
//!     }
//! );
//! ```

use core::alloc::Layout;
use iceoryx2_bb_concurrency::atomic::Ordering;
use iceoryx2_bb_concurrency::atomic::{AtomicBool, AtomicU64};
use iceoryx2_bb_elementary::bump_allocator::BumpAllocator;
use iceoryx2_bb_elementary::enum_gen;
use iceoryx2_bb_elementary::relocatable_ptr::{PointerTrait, RelocatablePointer};
use iceoryx2_bb_elementary_traits::allocator::AllocationError;
use iceoryx2_bb_elementary_traits::relocatable_container::RelocatableContainer;
use iceoryx2_log::{fail, fatal_panic};

use crate::mpmc::unique_index_set_enums::{
    ReleaseMode, ReleaseState, UniqueIndexCreationError, UniqueIndexSetAcquireFailure,
};

enum_gen! { OwnerIdNewError
  entry:
    InvalidOwnerValue
}

enum_gen! { RobustUniqueIndexSetReleaseError
  entry:
    IndexIsNotOwnedByProvidedOwner
}

const GENERATION_COUNTER_LOCK_INDICATOR: u64 = u64::MAX;

/// Represents the who owns the index acquired by the [`RobustUniqueIndexSet`].
/// This is used to recover leaked indices from dead owners.
#[repr(C)]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct OwnerId(u64);

impl OwnerId {
    const EMPTY: OwnerId = OwnerId(u64::MAX);

    /// Constructs a new [`OwnerId`]. The value is not allowed to be zero.
    pub fn new(value: u64) -> Result<Self, OwnerIdNewError> {
        if value == Self::EMPTY.0 {
            fail!(from "OwnerId::new()", with OwnerIdNewError::InvalidOwnerValue,
                "Invalid value for OwnerId. The OwnerId cannot be 0.");
        }

        Ok(Self(value))
    }
}

struct SetState {
    generation_counter: u64,
    borrowed_indices: u64,
}

#[repr(C)]
#[derive(Debug)]
pub struct RobustUniqueIndexSet {
    cell_ptr: RelocatablePointer<AtomicU64>,
    capacity: usize,
    is_memory_initialized: AtomicBool,
    generation_counter: AtomicU64,
}

impl RelocatableContainer for RobustUniqueIndexSet {
    unsafe fn new_uninit(capacity: usize) -> Self {
        Self {
            cell_ptr: unsafe { RelocatablePointer::new_uninit() },
            capacity,
            is_memory_initialized: AtomicBool::new(false),
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
            self.cell_ptr.init(fail!(from self,
                when allocator.allocate(layout),
                "Failed to initialize since the allocation of the data memory failed."))
        };

        for i in 0..self.capacity {
            unsafe {
                (self.cell_ptr.as_ptr() as *mut AtomicU64)
                    .add(i)
                    .write(AtomicU64::new(OwnerId::EMPTY.0))
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
        self.generation_counter.load(Ordering::Relaxed) == GENERATION_COUNTER_LOCK_INDICATOR
    }

    fn lock(&self) -> ReleaseState {
        if self.is_locked() {
            return ReleaseState::Locked;
        }

        loop {
            let state = self.borrowed_indices_and_generation_counter();

            // When the number of borrowed indices is 0 and the generation counter is unchanged
            // this means that either the set is empty or that another thread is currently calling
            // `acquire()` may have already populated a cell but before incrementing the generation counter
            //
            // In the latter case, the `acquire()` operation needs to finish by calling
            // `increment_generation_counter` which would return `ReleaseState::Locked` and therefore
            // the index would not be provided to the user.
            if state.borrowed_indices == 0 {
                match self.generation_counter.compare_exchange(
                    state.generation_counter,
                    GENERATION_COUNTER_LOCK_INDICATOR,
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

    /// Returns the capacity of the [`RobustUniqueIndexSet`].
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// The compile time version of [`RobustUniqueIndexSet::memory_size()`]
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
    /// unsafe { index_set.release(new_index, owner_id, ReleaseMode::Default) };
    /// ```
    ///
    /// # Safety
    ///
    /// * Ensure that [`RobustUniqueIndexSet::init()`] was called once.
    ///
    pub unsafe fn acquire(&self, owner_id: OwnerId) -> Result<usize, UniqueIndexSetAcquireFailure> {
        let msg = "Unable to acquire another index";
        self.verify_init("acquire()");
        let cell_ptr = unsafe { self.cell_ptr.as_ptr() };

        loop {
            //////////////////////////////////////
            // SYNC POINT enforce order of:
            //   1. cell
            //   2. generation counter
            //////////////////////////////////////
            let current_generation_count = self.generation_counter.load(Ordering::Acquire);

            if current_generation_count == GENERATION_COUNTER_LOCK_INDICATOR {
                fail!(from self, with UniqueIndexSetAcquireFailure::IsLocked,
                    "{msg} since the RobustUniqueIndexSet is locked.");
            }

            for n in 0..self.capacity {
                let cell = unsafe { &*cell_ptr.add(n) };

                match cell.compare_exchange(
                    OwnerId::EMPTY.0,
                    owner_id.0,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => {
                        //////////////////////////////////////
                        // SYNC POINT enforce order of:
                        //   1. cell
                        //   2. generation counter
                        //////////////////////////////////////
                        if self.increment_generation_counter(Ordering::Release)
                            == GENERATION_COUNTER_LOCK_INDICATOR
                        {
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
    /// See [`RobustUniqueIndexSet::acquire()`].
    ///
    /// # Safety
    ///
    ///  * Ensure that [`RobustUniqueIndexSet::init()`] was called once.
    ///
    pub unsafe fn release(
        &self,
        index: usize,
        owner_id: OwnerId,
        mode: ReleaseMode,
    ) -> Result<ReleaseState, RobustUniqueIndexSetReleaseError> {
        self.verify_init("release()");

        let cell = unsafe { &*self.cell_ptr.as_ptr().add(index) };

        match cell.compare_exchange(
            owner_id.0,
            OwnerId::EMPTY.0,
            Ordering::Relaxed,
            Ordering::Relaxed,
        ) {
            Ok(_) => {
                //////////////////////////////////////
                // SYNC POINT enforce order of:
                //   1. cell
                //   2. generation counter
                //////////////////////////////////////
                self.increment_generation_counter(Ordering::Release);
            }
            Err(v) => {
                fail!(from self,
                    with RobustUniqueIndexSetReleaseError::IndexIsNotOwnedByProvidedOwner,
                    "The index {index} that shall be released is not owned by {} but by {}.", owner_id.0, v);
            }
        }

        if mode == ReleaseMode::LockIfLastIndex {
            Ok(self.lock())
        } else {
            Ok(ReleaseState::Unlocked)
        }
    }

    /// Recovers leaked indices for which the predicate with the [`OwnerId`] returns true.
    ///
    /// # Safety
    ///
    /// * Ensure that [`RobustUniqueIndexSet::init()`] was called once.
    ///
    pub unsafe fn recover<F: FnMut(OwnerId, usize) -> bool, S: FnMut(OwnerId, usize)>(
        &self,
        mode: ReleaseMode,
        mut predicate: F,
        mut recover_success: S,
    ) -> ReleaseState {
        self.verify_init("acquire()");

        if self.is_locked() {
            return ReleaseState::Locked;
        }

        let cell_ptr = unsafe { self.cell_ptr.as_ptr() };

        for n in 0..self.capacity {
            let cell = unsafe { &*cell_ptr.add(n) };
            let owner_id = OwnerId(cell.load(Ordering::Relaxed));

            if owner_id == OwnerId::EMPTY {
                continue;
            }

            if predicate(owner_id, n)
                && cell
                    .compare_exchange(
                        owner_id.0,
                        OwnerId::EMPTY.0,
                        Ordering::Relaxed,
                        Ordering::Relaxed,
                    )
                    .is_ok()
            {
                recover_success(owner_id, n);
                //////////////////////////////////////
                // SYNC POINT enforce order of:
                //   1. cell
                //   2. generation counter
                //////////////////////////////////////
                if self.increment_generation_counter(Ordering::Release)
                    == GENERATION_COUNTER_LOCK_INDICATOR
                {
                    return ReleaseState::Locked;
                }

                if mode == ReleaseMode::LockIfLastIndex {
                    self.lock();
                }
            }
        }

        match self.is_locked() {
            true => ReleaseState::Locked,
            false => ReleaseState::Unlocked,
        }
    }

    /// This function ensures that a generation counter with the value `GENERATION_COUNTER_LOCK_INDICIATOR` is
    /// never changed since it indicates a locked index set. If this would happen
    /// the index set would be opened for modification again and the lock would be
    /// reverted.
    fn increment_generation_counter(&self, ordering: Ordering) -> u64 {
        let mut current_generation_count = self.generation_counter.load(Ordering::Relaxed);
        loop {
            if current_generation_count == GENERATION_COUNTER_LOCK_INDICATOR {
                return GENERATION_COUNTER_LOCK_INDICATOR;
            }

            match self.generation_counter.compare_exchange(
                current_generation_count,
                current_generation_count + 1,
                ordering,
                Ordering::Relaxed,
            ) {
                Ok(_) => return current_generation_count + 1,
                Err(v) => current_generation_count = v,
            }
        }
    }

    /// Returns how many indices were already borrowed.
    pub fn borrowed_indices(&self) -> usize {
        self.borrowed_indices_and_generation_counter()
            .borrowed_indices as usize
    }

    /// This funtion returns the number of borrowed indices, meaning how many
    /// `cell`s != `OwnerId::EMPTY` combined with the generation counter at that
    /// time.
    fn borrowed_indices_and_generation_counter(&self) -> SetState {
        let cell_ptr = unsafe { self.cell_ptr.as_ptr() };
        loop {
            //////////////////////////////////////
            // SYNC POINT enforce order of:
            //   1. cell
            //   2. generation counter
            //////////////////////////////////////
            let initial_generation_count = self.generation_counter.load(Ordering::Acquire);

            if initial_generation_count == GENERATION_COUNTER_LOCK_INDICATOR {
                return SetState {
                    generation_counter: initial_generation_count,
                    borrowed_indices: 0,
                };
            }

            let mut count = 0;
            for n in 0..self.capacity {
                let cell = unsafe { &*cell_ptr.add(n) };
                if cell.load(Ordering::Relaxed) != OwnerId::EMPTY.0 {
                    count += 1;
                }
            }

            //////////////////////////////////////
            // SYNC POINT enforce order of:
            //   1. cell
            //   2. generation counter
            //////////////////////////////////////
            let new_generation_count = self.increment_generation_counter(Ordering::Release);

            if initial_generation_count + 1 == new_generation_count {
                return SetState {
                    generation_counter: new_generation_count,
                    borrowed_indices: count,
                };
            }
        }
    }
}

#[derive(Debug)]
pub struct StaticRobustUniqueIndexSetData<const CAPACITY: usize> {
    cells: [AtomicU64; CAPACITY],
}

impl<const CAPACITY: usize> Default for StaticRobustUniqueIndexSetData<CAPACITY> {
    fn default() -> Self {
        Self {
            cells: [const { AtomicU64::new(0) }; CAPACITY],
        }
    }
}

impl<const CAPACITY: usize> StaticRobustUniqueIndexSetData<CAPACITY> {
    pub fn allocator(&mut self) -> BumpAllocator {
        BumpAllocator::new(self.cells.as_mut_ptr().cast())
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct StaticRobustUniqueIndexSet<const CAPACITY: usize> {
    state: RobustUniqueIndexSet,
    cells: StaticRobustUniqueIndexSetData<CAPACITY>,
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
            cells: StaticRobustUniqueIndexSetData::default(),
        };

        unsafe {
            new_self
                .state
                .init(&new_self.cells.allocator())
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
    /// unsafe { index_set.release(new_index, owner_id, ReleaseMode::Default) };
    /// ```
    pub fn acquire(&self, owner_id: OwnerId) -> Result<usize, UniqueIndexSetAcquireFailure> {
        unsafe { self.state.acquire(owner_id) }
    }

    /// Releases a raw index.
    pub fn release(
        &self,
        index: usize,
        owner_id: OwnerId,
        mode: ReleaseMode,
    ) -> Result<ReleaseState, RobustUniqueIndexSetReleaseError> {
        unsafe { self.state.release(index, owner_id, mode) }
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
    pub fn recover<F: FnMut(OwnerId, usize) -> bool, S: FnMut(OwnerId, usize)>(
        &self,
        mode: ReleaseMode,
        predicate: F,
        recover_success: S,
    ) -> ReleaseState {
        unsafe { self.state.recover(mode, predicate, recover_success) }
    }
}
