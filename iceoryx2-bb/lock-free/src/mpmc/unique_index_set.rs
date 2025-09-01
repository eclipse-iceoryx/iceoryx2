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

//! A **threadsafe** and **lock-free** set of indices. Can be used as a building block for
//! allocators or lock-free containers. When it is created it is filled with indices which can
//! be acquired and returned.
//!
//! # Example
//!
//! ## Runtime fixed size UniqueIndexSet
//!
//! ```
//! use iceoryx2_bb_elementary::bump_allocator::*;
//! use iceoryx2_bb_lock_free::mpmc::unique_index_set::*;
//! use iceoryx2_bb_elementary_traits::relocatable_container::*;
//!
//! const CAPACITY: usize = 128;
//! let mut memory = [0u8; UniqueIndexSet::const_memory_size(CAPACITY)];
//! let allocator = BumpAllocator::new(memory.as_mut_ptr());
//!
//! let mut index_set = unsafe { UniqueIndexSet::new_uninit(CAPACITY) };
//! unsafe { index_set.init(&allocator) }.expect("failed to allocate enough memory");
//!
//! let new_index = match unsafe { index_set.acquire() } {
//!     Err(_) => panic!("Out of indices"),
//!     Ok(i) => i,
//! };
//!
//! println!("Acquired index {}", new_index.value());
//!
//! // return the index to the index set
//! drop(new_index);
//! ```
//!
//! ## Compile time FixedSizeUniqueIndexSet
//!
//! ```
//! use iceoryx2_bb_lock_free::mpmc::unique_index_set::*;
//!
//! const CAPACITY: usize = 128;
//!
//! let index_set = FixedSizeUniqueIndexSet::<CAPACITY>::new();
//!
//! let new_index = match index_set.acquire() {
//!     Err(_) => panic!("Out of indices"),
//!     Ok(i) => i,
//! };
//!
//! println!("Acquired index {}", new_index.value());
//!
//! // return the index to the index set
//! drop(new_index);
//! ```
//!
//! ## Manual index return
//!
//! ```
//! use iceoryx2_bb_lock_free::mpmc::unique_index_set::*;
//!
//! const CAPACITY: usize = 128;
//!
//! let index_set = FixedSizeUniqueIndexSet::<CAPACITY>::new();
//!
//! let new_index = match unsafe { index_set.acquire_raw_index() } {
//!     Err(_) => panic!("Out of indices"),
//!     Ok(i) => i,
//! };
//!
//! println!("Acquired index {}", new_index);
//!
//! // return the index to the index set
//! unsafe { index_set.release_raw_index(new_index, ReleaseMode::Default) };
//! ```

use core::alloc::Layout;
use core::cell::UnsafeCell;
use core::fmt::Debug;
use core::sync::atomic::{fence, Ordering};
use iceoryx2_bb_elementary::bump_allocator::BumpAllocator;
use iceoryx2_bb_elementary::enum_gen;
use iceoryx2_bb_elementary::relocatable_ptr::RelocatablePointer;
use iceoryx2_bb_elementary_traits::allocator::{AllocationError, BaseAllocator};
use iceoryx2_bb_elementary_traits::pointer_trait::PointerTrait;
use iceoryx2_bb_elementary_traits::relocatable_container::RelocatableContainer;
use iceoryx2_bb_log::{fail, fatal_panic};
use iceoryx2_pal_concurrency_sync::iox_atomic::{IoxAtomicBool, IoxAtomicU64};

enum_gen! { UniqueIndexCreationError
  entry:
    ProvidedCapacityGreaterThanMaxCapacity,
    ProvidedCapacityIsZero
}

/// Describes if indices can still be acquired after the call to
/// [`UniqueIndexSet::release_raw_index()`].
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ReleaseMode {
    /// No more indices can be acquired with [`UniqueIndexSet::acquire_raw_index()`] if the
    /// released index was the last one.
    LockIfLastIndex,
    /// Indices can still be acquired with [`UniqueIndexSet::acquire_raw_index()`] after the
    /// operation
    Default,
}

/// Defines the state of the [`UniqueIndexSet`] after the release operation
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ReleaseState {
    /// The [`UniqueIndexSet`] is in locked mode since the last index was released. New indices
    /// can no longer acquired from the [`UniqueIndexSet`].
    Locked,
    /// New indices can still be acquired from the [`UniqueIndexSet`]
    Unlocked,
}

/// It states the reason if an index could not be acquired.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum UniqueIndexSetAcquireFailure {
    /// The [`UniqueIndexSet`] does not contain any more indices
    OutOfIndices,
    /// The [`UniqueIndexSet`] is in a locked state and indices can no longer be acquired.
    IsLocked,
}

/// Represents a [`UniqueIndex`]. When it goes out of scope it releases the index in the
/// corresponding [`UniqueIndexSet`] or [`FixedSizeUniqueIndexSet`].
///
/// The underlying value can be acquired with [`UniqueIndex::value()`].
pub struct UniqueIndex<'a> {
    value: u32,
    index_set: &'a UniqueIndexSet,
}

impl Debug for UniqueIndex<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "UniqueIndex {{ value: {}, index_set addr: {:#x} }}",
            self.value,
            core::ptr::addr_of!(self.index_set) as u64
        )
    }
}

impl UniqueIndex<'_> {
    /// Returns the value of the index.
    pub fn value(&self) -> u32 {
        self.value
    }
}

impl Drop for UniqueIndex<'_> {
    fn drop(&mut self) {
        unsafe {
            self.index_set
                .release_raw_index(self.value, ReleaseMode::Default)
        };
    }
}

/// A **non-movable** UniqueIndexSet with a runtime fixed capacity. The compile time version is called
/// [`FixedSizeUniqueIndexSet`].
///
/// # Examples
///
/// ## With a custom allocator
/// ```
/// use iceoryx2_bb_elementary::bump_allocator::*;
/// use iceoryx2_bb_lock_free::mpmc::unique_index_set::*;
/// use iceoryx2_bb_elementary_traits::relocatable_container::*;
///
/// const CAPACITY: usize = 128;
/// let mut memory = [0u8; UniqueIndexSet::const_memory_size(CAPACITY)];
/// let allocator = BumpAllocator::new(memory.as_mut_ptr());
///
/// let mut index_set = unsafe { UniqueIndexSet::new_uninit(CAPACITY) };
/// unsafe { index_set.init(&allocator) }.expect("failed to allocate enough memory");
///
/// let new_index = match unsafe { index_set.acquire() } {
///     Err(_) => panic!("Out of indices"),
///     Ok(i) => i,
/// };
/// ```
///
/// ## Provide memory in a separate struct
/// ```
/// use iceoryx2_bb_lock_free::mpmc::unique_index_set::*;
/// use iceoryx2_bb_elementary_traits::relocatable_container::*;
/// use iceoryx2_bb_elementary::bump_allocator::BumpAllocator;
/// use core::mem::MaybeUninit;
///
/// const CAPACITY: usize = 128;
///
/// // We require repr(C) to ensure that the variables have the exact ordering like the members are
/// // declared.
/// #[repr(C)]
/// struct FixedSizeSet {
///     set: UniqueIndexSet,
///     data: [MaybeUninit<u8>; UniqueIndexSet::const_memory_size(CAPACITY)]
/// }
///
/// impl FixedSizeSet {
///     pub fn new() -> Self {
///         let mut new_self = FixedSizeSet {
///             set: unsafe { UniqueIndexSet::new_uninit(CAPACITY) },
///             data: [MaybeUninit::uninit(); UniqueIndexSet::const_memory_size(CAPACITY)]
///         };
///
///         let allocator = BumpAllocator::new(new_self.data.as_mut_ptr().cast());
///         unsafe {
///             new_self.set.init(&allocator).expect("Enough memory provided.")
///         };
///         new_self
///     }
/// }
/// ```
#[repr(C)]
#[derive(Debug)]
pub struct UniqueIndexSet {
    data_ptr: RelocatablePointer<UnsafeCell<u32>>,
    capacity: u32,
    pub(crate) head: IoxAtomicU64,
    is_memory_initialized: IoxAtomicBool,
}

unsafe impl Sync for UniqueIndexSet {}
unsafe impl Send for UniqueIndexSet {}

const LOCK_ACQUIRE: u32 = 0x00ffffff;

struct HeadDetails {
    head: u32,
    aba: u16,
    borrowed_indices: u32,
}

impl HeadDetails {
    fn from(value: u64) -> Self {
        Self {
            head: ((value >> 40) as u32) & 0xffffff,
            aba: (value >> 24) as u16,
            borrowed_indices: (value as u32) & 0xffffff,
        }
    }

    fn value(&self) -> u64 {
        (((self.head & 0x00ffffff) as u64) << 40)
            | ((self.aba as u64) << 24)
            | ((self.borrowed_indices & 0x00ffffff) as u64)
    }
}

impl RelocatableContainer for UniqueIndexSet {
    unsafe fn new_uninit(capacity: usize) -> Self {
        debug_assert!(
            capacity < 2usize.pow(24) - 1,
            "The provided capacity exceeds the maximum supported capacity of the UniqueIndexSet"
        );

        Self {
            data_ptr: RelocatablePointer::new_uninit(),
            capacity: capacity as u32,
            head: IoxAtomicU64::new(0),
            is_memory_initialized: IoxAtomicBool::new(false),
        }
    }

    unsafe fn init<T: BaseAllocator>(&mut self, allocator: &T) -> Result<(), AllocationError> {
        if self.is_memory_initialized.load(Ordering::Relaxed) {
            fatal_panic!(from self, "Memory already initialized. Initializing it twice may lead to undefined behavior.");
        }

        self.data_ptr.init(fail!(from self, when allocator
            .allocate(Layout::from_size_align_unchecked(
                core::mem::size_of::<u32>() * (self.capacity + 1) as usize,
                core::mem::align_of::<u32>())),
            "Failed to initialize since the allocation of the data memory failed."
        ));

        for i in 0..self.capacity + 1 {
            (self.data_ptr.as_ptr() as *mut UnsafeCell<u32>)
                .offset(i as isize)
                .write(UnsafeCell::new(i + 1));
        }

        self.is_memory_initialized.store(true, Ordering::Relaxed);
        Ok(())
    }

    fn memory_size(capacity: usize) -> usize {
        Self::const_memory_size(capacity)
    }
}

impl UniqueIndexSet {
    #[inline(always)]
    fn verify_init(&self, source: &str) {
        debug_assert!(
            self.is_memory_initialized.load(Ordering::Relaxed),
            "Undefined behavior when calling UniqueIndexSet::{source} and the object is not initialized."
        );
    }

    /// The compile time version of [`UniqueIndexSet::memory_size()`]
    pub const fn const_memory_size(capacity: usize) -> usize {
        core::mem::size_of::<UnsafeCell<u32>>() * (capacity + 1) + core::mem::align_of::<u32>() - 1
    }

    /// Acquires a new [`UniqueIndex`]. If the set does not contain any more indices it returns
    /// [`None`].
    ///
    /// # Safety
    ///
    /// * Ensure that [`UniqueIndexSet::init()`] was called once.
    ///
    pub unsafe fn acquire(&self) -> Result<UniqueIndex<'_>, UniqueIndexSetAcquireFailure> {
        self.verify_init("acquire()");
        unsafe { self.acquire_raw_index() }.map(|v| UniqueIndex {
            value: v,
            index_set: self,
        })
    }

    /// Returns the capacity of the [`UniqueIndexSet`].
    pub fn capacity(&self) -> u32 {
        self.capacity
    }

    /// Returns the current len.
    pub fn borrowed_indices(&self) -> usize {
        let s = HeadDetails::from(self.head.load(Ordering::Relaxed)).borrowed_indices;
        if s == LOCK_ACQUIRE {
            0
        } else {
            s as usize
        }
    }

    /// Returns if the [`UniqueIndexSet`] is locked or not. If the set is locked no more indices
    /// can be borrowed.
    pub fn is_locked(&self) -> bool {
        let s = HeadDetails::from(self.head.load(Ordering::Relaxed)).borrowed_indices;
        s == LOCK_ACQUIRE
    }

    /// Acquires a raw ([`u32`]) index from the [`UniqueIndexSet`]. Returns [`None`] when no more
    /// indices are available. The index **must** be returned manually with
    /// [`UniqueIndexSet::release_raw_index()`].
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2_bb_lock_free::mpmc::unique_index_set::*;
    ///
    /// const CAPACITY: usize = 128;
    ///
    /// let index_set = FixedSizeUniqueIndexSet::<CAPACITY>::new();
    ///
    /// let new_index = match unsafe { index_set.acquire_raw_index() } {
    ///     Err(_) => panic!("Out of indices"),
    ///     Ok(i) => i,
    /// };
    ///
    /// println!("Acquired index {}", new_index);
    ///
    /// // return the index to the index set
    /// unsafe { index_set.release_raw_index(new_index, ReleaseMode::Default) };
    /// ```
    ///
    /// # Safety
    ///
    ///  * Ensure that [`UniqueIndexSet::init()`] was called once.
    ///  * The index must be manually released with [`UniqueIndexSet::release_raw_index()`]
    ///    otherwise the index is leaked.
    pub unsafe fn acquire_raw_index(&self) -> Result<u32, UniqueIndexSetAcquireFailure> {
        self.verify_init("acquire_raw_index()");
        let mut old_value = self.head.load(Ordering::Acquire);
        let mut old = HeadDetails::from(old_value);

        loop {
            if old.head >= self.capacity {
                return Err(UniqueIndexSetAcquireFailure::OutOfIndices);
            }

            if old.borrowed_indices == LOCK_ACQUIRE {
                return Err(UniqueIndexSetAcquireFailure::IsLocked);
            }

            let new_value = HeadDetails {
                head: *self.get_next_free_index(old.head),
                aba: old.aba.wrapping_add(1),
                borrowed_indices: old.borrowed_indices + 1,
            }
            .value();

            old = match self.head.compare_exchange(
                old_value,
                new_value,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => break,
                Err(v) => {
                    old_value = v;
                    HeadDetails::from(v)
                }
            }
        }

        let index = old.head;
        *self.get_next_free_index(index) = self.capacity + 1;

        fence(Ordering::Acquire);
        Ok(index)
    }

    /// Releases a raw index.
    ///
    /// # Example
    ///
    /// See [`UniqueIndexSet::acquire_raw_index()`].
    ///
    /// # Safety
    ///
    ///  * Ensure that [`UniqueIndexSet::init()`] was called once.
    ///  * It must be ensured that the index was acquired before and is not released twice.
    ///  * Shall be only used when the index was acquired with
    ///    [`UniqueIndexSet::acquire_raw_index()`]
    pub unsafe fn release_raw_index(&self, index: u32, mode: ReleaseMode) -> ReleaseState {
        self.verify_init("release_raw_index()");
        fence(Ordering::Release);

        let mut release_state;
        let mut old_value = self.head.load(Ordering::Acquire);
        let mut old = HeadDetails::from(old_value);

        loop {
            *self.get_next_free_index(index) = old.head;

            let borrowed_indices =
                if mode == ReleaseMode::LockIfLastIndex && old.borrowed_indices == 1 {
                    release_state = ReleaseState::Locked;
                    LOCK_ACQUIRE
                } else {
                    release_state = ReleaseState::Unlocked;
                    old.borrowed_indices - 1
                };

            let new_value = HeadDetails {
                head: index,
                aba: old.aba.wrapping_add(1),
                borrowed_indices,
            }
            .value();

            old = match self.head.compare_exchange(
                old_value,
                new_value,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => {
                    return release_state;
                }
                Err(v) => {
                    old_value = v;
                    HeadDetails::from(v)
                }
            };
        }
    }

    #[allow(clippy::mut_from_ref)]
    // convenience function to access internally mutable object
    fn get_next_free_index(&self, index: u32) -> &mut u32 {
        #[deny(clippy::mut_from_ref)]
        unsafe {
            &mut *(*self.data_ptr.as_ptr().offset(index as isize)).get()
        }
    }
}

/// The compile time fixed size version of the [`UniqueIndexSet`].
///
/// # Example
///
/// ```
/// use iceoryx2_bb_lock_free::mpmc::unique_index_set::*;
///
/// const CAPACITY: usize = 128;
///
/// let index_set = FixedSizeUniqueIndexSet::<CAPACITY>::new();
///
/// let new_index = match index_set.acquire() {
///     Err(_) => panic!("Out of indices"),
///     Ok(i) => i,
/// };
/// ```
#[derive(Debug)]
#[repr(C)]
pub struct FixedSizeUniqueIndexSet<const CAPACITY: usize> {
    pub(crate) state: UniqueIndexSet,
    next_free_index: [UnsafeCell<u32>; CAPACITY],
    next_free_index_plus_one: UnsafeCell<u32>,
}

impl<const CAPACITY: usize> Default for FixedSizeUniqueIndexSet<CAPACITY> {
    fn default() -> Self {
        Self::new_with_reduced_capacity(CAPACITY).expect("Does not exceed supported capacity.")
    }
}

unsafe impl<const CAPACITY: usize> Sync for FixedSizeUniqueIndexSet<CAPACITY> {}
unsafe impl<const CAPACITY: usize> Send for FixedSizeUniqueIndexSet<CAPACITY> {}

impl<const CAPACITY: usize> FixedSizeUniqueIndexSet<CAPACITY> {
    /// Creates a new [`FixedSizeUniqueIndexSet`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new [`FixedSizeUniqueIndexSet`] where the capacity is reduced. If the capacity
    /// is greater than CAPACITY or zero it fails.
    pub fn new_with_reduced_capacity(capacity: usize) -> Result<Self, UniqueIndexCreationError> {
        if capacity > CAPACITY {
            fail!(from "FixedSizeUniqueIndexSet::new_with_reduced_capacity", with UniqueIndexCreationError::ProvidedCapacityGreaterThanMaxCapacity,
                "Provided value of capacity {} exceeds maximum supported capacity of {}.",
                capacity, CAPACITY);
        }

        if capacity == 0 {
            fail!(from "FixedSizeUniqueIndexSet::new_with_reduced_capacity", with UniqueIndexCreationError::ProvidedCapacityIsZero,
                "Provided value of capacity is zero.");
        }

        let mut new_self = Self {
            state: unsafe { UniqueIndexSet::new_uninit(capacity) },
            next_free_index: core::array::from_fn(|i| UnsafeCell::new(i as u32 + 1)),
            next_free_index_plus_one: UnsafeCell::new(capacity as u32 + 1),
        };

        let allocator = BumpAllocator::new(new_self.next_free_index.as_mut_ptr().cast());
        unsafe {
            new_self
                .state
                .init(&allocator)
                .expect("All required memory is preallocated.")
        };

        Ok(new_self)
    }

    /// See [`UniqueIndexSet::acquire()`]
    pub fn acquire(&self) -> Result<UniqueIndex<'_>, UniqueIndexSetAcquireFailure> {
        unsafe { self.state.acquire() }
    }

    /// See [`UniqueIndexSet::capacity()`]
    pub fn capacity(&self) -> u32 {
        self.state.capacity()
    }

    /// See [`UniqueIndexSet::is_locked()`]
    pub fn is_locked(&self) -> bool {
        self.state.is_locked()
    }

    /// See [`UniqueIndexSet::acquire_raw_index()`]
    ///
    /// # Safety
    ///
    ///  * The acquired index must be returned manually with
    ///    [`FixedSizeUniqueIndexSet::release_raw_index()`]
    ///
    pub unsafe fn acquire_raw_index(&self) -> Result<u32, UniqueIndexSetAcquireFailure> {
        self.state.acquire_raw_index()
    }

    /// See [`UniqueIndexSet::release_raw_index()`]
    ///
    /// # Safety
    ///
    ///  * The release index must have been acquired with
    ///    [`FixedSizeUniqueIndexSet::acquire_raw_index()`]
    ///  * The index should not be released twice
    ///
    pub unsafe fn release_raw_index(&self, index: u32, mode: ReleaseMode) -> ReleaseState {
        self.state.release_raw_index(index, mode)
    }

    /// Returns the current len.
    pub fn borrowed_indices(&self) -> usize {
        self.state.borrowed_indices()
    }
}

#[cfg(test)]
mod test {
    use iceoryx2_bb_testing::assert_that;

    use super::HeadDetails;

    #[test]
    fn head_details() {
        let sut_value = HeadDetails {
            head: 12345,
            aba: 6789,
            borrowed_indices: 54321,
        }
        .value();

        let sut = HeadDetails::from(sut_value);

        assert_that!(sut.head, eq 12345);
        assert_that!(sut.aba, eq 6789);
        assert_that!(sut.borrowed_indices, eq 54321);
    }

    #[test]
    fn head_details_head_field_is_non_overlapping() {
        let sut = HeadDetails::from(
            HeadDetails {
                head: 2u32.pow(24) - 1,
                aba: 0,
                borrowed_indices: 0,
            }
            .value(),
        );

        assert_that!(sut.head, eq 2u32.pow(24) - 1);
        assert_that!(sut.aba, eq 0);
        assert_that!(sut.borrowed_indices, eq 0);
    }

    #[test]
    fn head_details_aba_field_is_non_overlapping() {
        let sut = HeadDetails::from(
            HeadDetails {
                head: 0,
                aba: u16::MAX,
                borrowed_indices: 0,
            }
            .value(),
        );

        assert_that!(sut.head, eq 0);
        assert_that!(sut.aba, eq u16::MAX);
        assert_that!(sut.borrowed_indices, eq 0);
    }

    #[test]
    fn head_details_borrowed_indices_field_is_non_overlapping() {
        let sut = HeadDetails::from(
            HeadDetails {
                head: 0,
                aba: 0,
                borrowed_indices: 2u32.pow(24) - 1,
            }
            .value(),
        );

        assert_that!(sut.head, eq 0);
        assert_that!(sut.aba, eq 0);
        assert_that!(sut.borrowed_indices, eq 2u32.pow(24) - 1);
    }
}
