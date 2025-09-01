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

//! A SlotMap is a container that has a static unique key for every stored value. Adding or
//! removing values to the SlotMap do not change the unique key of the remaining values.
//! Multiple variationes of that container are available.
//!
//!  * [`SlotMap`](crate::slotmap::SlotMap), run-time fixed-size slotmap that is not shared-memory
//!    compatible since the memory resides in the heap.
//!  * [`FixedSizeSlotMap`](crate::slotmap::FixedSizeSlotMap), compile-time fixed-size slotmap that
//!    is self-contained and shared-memory compatible.
//!  * [`RelocatableSlotMap`](crate::slotmap::RelocatableSlotMap), run-time fixed-size slotmap that
//!    is shared-memory compatible.
//!
//! The SlotMap shall satisfy the following requirements:
//!
//!  * A new element can be inserted with a max runtime of `O(1)`
//!  * A new element can be inserted at a user-provided key with a max runtime of `O(1)`
//!  * An element can be removed by providing the corresponding key with a max runtime of `O(1)`
//!  * One can iterate over all elements of the SlotMap.
//!
//! The SlotMap is the perfect container when elements shall be added, removed and accesses quickly
//! but iteration is allowed to be slow.
//!
//! # User Examples
//!
//! ```
//! use iceoryx2_bb_container::slotmap::FixedSizeSlotMap;
//!
//! const CAPACITY: usize = 123;
//! let mut slotmap = FixedSizeSlotMap::<u64, CAPACITY>::new();
//!
//! let key = slotmap.insert(78181).unwrap();
//!
//! println!("value: {:?}", slotmap.get(key));
//! ```

use crate::queue::MetaQueue;
use crate::vec::MetaVec;
use crate::{queue::RelocatableQueue, vec::RelocatableVec};
use core::mem::MaybeUninit;
use iceoryx2_bb_derive_macros::ZeroCopySend;
use iceoryx2_bb_elementary::bump_allocator::BumpAllocator;
use iceoryx2_bb_elementary::relocatable_ptr::GenericRelocatablePointer;
use iceoryx2_bb_elementary_traits::generic_pointer::GenericPointer;
use iceoryx2_bb_elementary_traits::owning_pointer::GenericOwningPointer;
use iceoryx2_bb_elementary_traits::placement_default::PlacementDefault;
pub use iceoryx2_bb_elementary_traits::relocatable_container::RelocatableContainer;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use iceoryx2_bb_log::{fail, fatal_panic};
use iceoryx2_pal_concurrency_sync::iox_atomic::IoxAtomicBool;

/// A key of a [`SlotMap`], [`RelocatableSlotMap`] or [`FixedSizeSlotMap`] that identifies a
/// value.
#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct SlotMapKey(usize);

impl SlotMapKey {
    /// Creates a new [`SlotMapKey`] with the specified value.
    pub fn new(value: usize) -> Self {
        Self(value)
    }

    /// Returns the underlying value of the [`SlotMapKey`].
    pub fn value(&self) -> usize {
        self.0
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, ZeroCopySend)]
pub(crate) struct FreeListEntry {
    previous: usize,
    next: usize,
}

/// A runtime fixed-size, non-shared memory compatible [`SlotMap`]. The [`SlotMap`]s memory resides
/// in the heap.
pub type SlotMap<T> = MetaSlotMap<T, GenericOwningPointer>;

/// A runtime fixed-size, shared-memory compatible [`RelocatableSlotMap`].
pub type RelocatableSlotMap<T> = MetaSlotMap<T, GenericRelocatablePointer>;

const INVALID: usize = usize::MAX;

#[doc(hidden)]
/// The iterator of a [`SlotMap`], [`RelocatableSlotMap`] or [`FixedSizeSlotMap`].
pub struct Iter<'slotmap, T, Ptr: GenericPointer> {
    slotmap: &'slotmap MetaSlotMap<T, Ptr>,
    key: SlotMapKey,
}

#[doc(hidden)]
pub type OwningIter<'slotmap, T> = Iter<'slotmap, T, GenericOwningPointer>;
#[doc(hidden)]
pub type RelocatableIter<'slotmap, T> = Iter<'slotmap, T, GenericRelocatablePointer>;

impl<'slotmap, T, Ptr: GenericPointer> Iterator for Iter<'slotmap, T, Ptr> {
    type Item = (SlotMapKey, &'slotmap T);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((next_key, value)) = self.slotmap.next_available_key_after(self.key) {
            self.key.0 = next_key.0 + 1;
            Some((next_key, value))
        } else {
            None
        }
    }
}

#[doc(hidden)]
#[repr(C)]
#[derive(Debug)]
pub struct MetaSlotMap<T, Ptr: GenericPointer> {
    idx_to_data: MetaVec<usize, Ptr>,
    idx_to_data_free_list: MetaVec<FreeListEntry, Ptr>,
    data: MetaVec<Option<T>, Ptr>,
    data_next_free_index: MetaQueue<usize, Ptr>,
    idx_to_data_free_list_head: usize,
    is_initialized: IoxAtomicBool,
    len: usize,
}

impl<T, Ptr: GenericPointer> MetaSlotMap<T, Ptr> {
    #[inline(always)]
    fn verify_init(&self, source: &str) {
        debug_assert!(
                self.is_initialized
                    .load(core::sync::atomic::Ordering::Relaxed),
                "From: MetaSlotMap<{}>::{}, Undefined behavior - the object was not initialized with 'init' before.",
                core::any::type_name::<T>(), source
            );
    }

    fn next_available_key_after(&self, start: SlotMapKey) -> Option<(SlotMapKey, &T)> {
        let idx_to_data = &self.idx_to_data;

        for n in start.0..idx_to_data.len() {
            let data_idx = self.idx_to_data[n];
            if data_idx != INVALID {
                return Some((
                    SlotMapKey(n),
                    self.data[data_idx].as_ref().expect(
                        "By contract, data contains a value when idx_to_data contains a value",
                    ),
                ));
            }
        }

        None
    }

    pub(crate) unsafe fn initialize_data_structures(&mut self) {
        let capacity = self.capacity_impl();
        for n in 0..capacity {
            self.idx_to_data.push_impl(INVALID);
            self.data.push_impl(None);
            self.data_next_free_index.push_impl(n);

            let previous = if n == 0 { INVALID } else { n - 1 };
            let next = if n < capacity - 1 { n + 1 } else { INVALID };
            self.idx_to_data_free_list
                .push_impl(FreeListEntry { previous, next });
        }
    }

    pub(crate) unsafe fn iter_impl(&self) -> Iter<'_, T, Ptr> {
        self.verify_init("iter()");
        Iter {
            slotmap: self,
            key: SlotMapKey(0),
        }
    }

    pub(crate) unsafe fn contains_impl(&self, key: SlotMapKey) -> bool {
        self.verify_init("contains()");
        self.idx_to_data[key.0] != INVALID
    }

    pub(crate) unsafe fn get_impl(&self, key: SlotMapKey) -> Option<&T> {
        self.verify_init("get()");
        match self.idx_to_data[key.0] {
            INVALID => None,
            n => Some(self.data[n].as_ref().expect(
                "data and idx_to_data correspond and this value must be always available.",
            )),
        }
    }

    pub(crate) unsafe fn get_mut_impl(&mut self, key: SlotMapKey) -> Option<&mut T> {
        self.verify_init("get_mut()");
        match self.idx_to_data[key.0] {
            INVALID => None,
            n => Some(self.data[n].as_mut().expect(
                "data and idx_to_data correspond and this value must be always available.",
            )),
        }
    }

    unsafe fn acquire_next_free_index(&mut self) -> Option<usize> {
        if self.idx_to_data_free_list_head == INVALID {
            return None;
        }

        let free_idx = self.idx_to_data_free_list_head;
        let next = self.idx_to_data_free_list[free_idx].next;

        if next != INVALID {
            self.idx_to_data_free_list[next].previous = INVALID;
        }
        self.idx_to_data_free_list_head = next;
        Some(free_idx)
    }

    unsafe fn claim_index(&mut self, idx: usize) {
        if idx >= self.capacity_impl() {
            return;
        }

        let entry = self.idx_to_data_free_list[idx];
        if entry.previous != INVALID {
            self.idx_to_data_free_list[entry.previous].next = entry.next;
        }
        if entry.next != INVALID {
            self.idx_to_data_free_list[entry.next].previous = entry.previous;
        }
        self.idx_to_data_free_list[idx].next = INVALID;
        self.idx_to_data_free_list[idx].previous = INVALID;
    }

    unsafe fn release_free_index(&mut self, idx: usize) {
        if self.idx_to_data_free_list_head != INVALID {
            self.idx_to_data_free_list[self.idx_to_data_free_list_head].previous = idx;
        }

        self.idx_to_data_free_list[idx] = FreeListEntry {
            previous: INVALID,
            next: self.idx_to_data_free_list_head,
        };

        self.idx_to_data_free_list_head = idx;
    }

    pub(crate) unsafe fn insert_impl(&mut self, value: T) -> Option<SlotMapKey> {
        self.verify_init("insert()");
        self.acquire_next_free_index().map(|key| {
            let key = SlotMapKey(key);
            self.store_value(key, value);
            key
        })
    }

    pub(crate) unsafe fn insert_at_impl(&mut self, key: SlotMapKey, value: T) -> bool {
        self.verify_init("insert_at()");
        self.claim_index(key.value());
        self.store_value(key, value)
    }

    pub(crate) unsafe fn store_value(&mut self, key: SlotMapKey, value: T) -> bool {
        self.verify_init("store()");
        if key.0 > self.capacity_impl() {
            return false;
        }

        let data_idx = self.idx_to_data[key.0];
        if data_idx != INVALID {
            self.data[data_idx] = Some(value);
        } else {
            let n = self.data_next_free_index.pop_impl().expect(
                "data and idx_to_data correspond and there must be always a free index available.",
            );
            self.idx_to_data[key.0] = n;
            self.data[n] = Some(value);
            self.len += 1;
        }

        true
    }

    pub(crate) unsafe fn remove_impl(&mut self, key: SlotMapKey) -> Option<T> {
        self.verify_init("remove()");
        if key.0 > self.idx_to_data.len() {
            return None;
        }

        let data_idx = self.idx_to_data[key.0];
        if data_idx != INVALID {
            let ret = self.data[data_idx].take();
            let push_result = self.data_next_free_index.push_impl(data_idx);
            debug_assert!(push_result);
            self.release_free_index(key.0);
            self.idx_to_data[key.0] = INVALID;
            self.len -= 1;
            ret
        } else {
            None
        }
    }

    pub(crate) unsafe fn next_free_key_impl(&self) -> Option<SlotMapKey> {
        self.verify_init("next_free_key()");
        if self.idx_to_data_free_list_head == INVALID {
            return None;
        }

        Some(SlotMapKey::new(self.idx_to_data_free_list_head))
    }

    pub(crate) fn len_impl(&self) -> usize {
        self.len
    }

    pub(crate) fn capacity_impl(&self) -> usize {
        self.idx_to_data.capacity()
    }

    pub(crate) fn is_empty_impl(&self) -> bool {
        self.len_impl() == 0
    }

    pub(crate) fn is_full_impl(&self) -> bool {
        self.len_impl() == self.capacity_impl()
    }
}

impl<T> RelocatableContainer for RelocatableSlotMap<T> {
    unsafe fn new_uninit(capacity: usize) -> Self {
        Self {
            len: 0,
            idx_to_data_free_list_head: 0,
            idx_to_data: RelocatableVec::new_uninit(capacity),
            idx_to_data_free_list: RelocatableVec::new_uninit(capacity),
            data: RelocatableVec::new_uninit(capacity),
            data_next_free_index: RelocatableQueue::new_uninit(capacity),
            is_initialized: IoxAtomicBool::new(false),
        }
    }

    unsafe fn init<Allocator: iceoryx2_bb_elementary_traits::allocator::BaseAllocator>(
        &mut self,
        allocator: &Allocator,
    ) -> Result<(), iceoryx2_bb_elementary_traits::allocator::AllocationError> {
        if self
            .is_initialized
            .load(core::sync::atomic::Ordering::Relaxed)
        {
            fatal_panic!(from "RelocatableSlotMap::init()", "Memory already initialized. Initializing it twice may lead to undefined behavior.");
        }
        let msg = "Unable to initialize RelocatableSlotMap";
        fail!(from "RelocatableSlotMap::init()",
                  when self.idx_to_data.init(allocator),
                  "{msg} since the underlying idx_to_data vector could not be initialized.");
        fail!(from "RelocatableSlotMap::init()",
                  when self.idx_to_data_free_list.init(allocator),
                  "{msg} since the underlying idx_to_data_free_list vec could not be initialized.");
        fail!(from "RelocatableSlotMap::init()",
                  when self.data.init(allocator),
                  "{msg} since the underlying data vector could not be initialized.");
        fail!(from "RelocatableSlotMap::init()",
                  when self.data_next_free_index.init(allocator),
                  "{msg} since the underlying data_next_free_index queue could not be initialized.");

        self.initialize_data_structures();
        self.is_initialized
            .store(true, core::sync::atomic::Ordering::Relaxed);
        Ok(())
    }

    fn memory_size(capacity: usize) -> usize {
        Self::const_memory_size(capacity)
    }
}

impl<T> SlotMap<T> {
    /// Creates a new runtime-fixed size [`SlotMap`] on the heap with the given capacity.
    pub fn new(capacity: usize) -> Self {
        let mut new_self = Self {
            len: 0,
            idx_to_data_free_list_head: 0,
            idx_to_data: MetaVec::new(capacity),
            idx_to_data_free_list: MetaVec::new(capacity),
            data: MetaVec::new(capacity),
            data_next_free_index: MetaQueue::new(capacity),
            is_initialized: IoxAtomicBool::new(true),
        };
        unsafe { new_self.initialize_data_structures() };
        new_self
    }

    /// Returns the [`Iter`]ator to iterate over all entries.
    pub fn iter(&self) -> OwningIter<'_, T> {
        unsafe { self.iter_impl() }
    }

    /// Returns `true` if the provided `key` is contained, otherwise `false`.
    pub fn contains(&self, key: SlotMapKey) -> bool {
        unsafe { self.contains_impl(key) }
    }

    /// Returns a reference to the value stored under the given key. If there is no such key,
    /// [`None`] is returned.
    pub fn get(&self, key: SlotMapKey) -> Option<&T> {
        unsafe { self.get_impl(key) }
    }

    /// Returns a mutable reference to the value stored under the given key. If there is no
    /// such key, [`None`] is returned.
    pub fn get_mut(&mut self, key: SlotMapKey) -> Option<&mut T> {
        unsafe { self.get_mut_impl(key) }
    }

    /// Insert a value and returns the corresponding [`SlotMapKey`]. If the container is full
    /// [`None`] is returned.
    pub fn insert(&mut self, value: T) -> Option<SlotMapKey> {
        unsafe { self.insert_impl(value) }
    }

    /// Insert a value at the specified [`SlotMapKey`] and returns true.  If the provided key
    /// is out-of-bounds it returns `false` and adds nothing. If there is already a value
    /// stored at the `key`s index, the value is overridden with the provided value.
    pub fn insert_at(&mut self, key: SlotMapKey, value: T) -> bool {
        unsafe { self.insert_at_impl(key, value) }
    }

    /// Removes a value at the specified [`SlotMapKey`]. If there was no value corresponding
    /// to the [`SlotMapKey`] it returns None, otherwise Some(value).
    pub fn remove(&mut self, key: SlotMapKey) -> Option<T> {
        unsafe { self.remove_impl(key) }
    }

    /// Returns the [`SlotMapKey`] that will be used when the user calls
    /// [`SlotMap::insert()`]. If the [`SlotMap`] is full it returns [`None`].
    pub fn next_free_key(&self) -> Option<SlotMapKey> {
        unsafe { self.next_free_key_impl() }
    }

    /// Returns the number of stored values.
    pub fn len(&self) -> usize {
        self.len_impl()
    }

    /// Returns the capacity.
    pub fn capacity(&self) -> usize {
        self.capacity_impl()
    }

    /// Returns true if the container is empty, otherwise false.
    pub fn is_empty(&self) -> bool {
        self.is_empty_impl()
    }

    /// Returns true if the container is full, otherwise false.
    pub fn is_full(&self) -> bool {
        self.is_full_impl()
    }
}

unsafe impl<T: ZeroCopySend> ZeroCopySend for RelocatableSlotMap<T> {}

impl<T> RelocatableSlotMap<T> {
    /// Returns how many memory the [`RelocatableSlotMap`] will allocate from the allocator
    /// in [`RelocatableSlotMap::init()`].
    pub const fn const_memory_size(capacity: usize) -> usize {
        RelocatableVec::<usize>::const_memory_size(capacity)
            + RelocatableVec::<FreeListEntry>::const_memory_size(capacity)
            + RelocatableVec::<Option<T>>::const_memory_size(capacity)
            + RelocatableQueue::<usize>::const_memory_size(capacity)
    }

    /// Returns the [`Iter`]ator to iterate over all entries.
    ///
    /// # Safety
    ///
    ///  * [`RelocatableSlotMap::init()`] must be called once before
    ///
    pub unsafe fn iter(&self) -> RelocatableIter<'_, T> {
        self.iter_impl()
    }

    /// Returns `true` if the provided `key` is contained, otherwise `false`.
    ///
    /// # Safety
    ///
    ///  * [`RelocatableSlotMap::init()`] must be called once before
    ///
    pub unsafe fn contains(&self, key: SlotMapKey) -> bool {
        self.contains_impl(key)
    }

    /// Returns a reference to the value stored under the given key. If there is no such key,
    /// [`None`] is returned.
    ///
    /// # Safety
    ///
    ///  * [`RelocatableSlotMap::init()`] must be called once before
    ///
    pub unsafe fn get(&self, key: SlotMapKey) -> Option<&T> {
        self.get_impl(key)
    }

    /// Returns a mutable reference to the value stored under the given key. If there is no
    /// such key, [`None`] is returned.
    ///
    /// # Safety
    ///
    ///  * [`RelocatableSlotMap::init()`] must be called once before
    ///
    pub unsafe fn get_mut(&mut self, key: SlotMapKey) -> Option<&mut T> {
        self.get_mut_impl(key)
    }

    /// Insert a value and returns the corresponding [`SlotMapKey`]. If the container is full
    /// [`None`] is returned.
    ///
    /// # Safety
    ///
    ///  * [`RelocatableSlotMap::init()`] must be called once before
    ///
    pub unsafe fn insert(&mut self, value: T) -> Option<SlotMapKey> {
        self.insert_impl(value)
    }

    /// Insert a value at the specified [`SlotMapKey`] and returns true.  If the provided key
    /// is out-of-bounds it returns `false` and adds nothing. If there is already a value
    /// stored at the `key`s index, the value is overridden with the provided value.
    ///
    /// # Safety
    ///
    ///  * [`RelocatableSlotMap::init()`] must be called once before
    ///
    pub unsafe fn insert_at(&mut self, key: SlotMapKey, value: T) -> bool {
        self.insert_at_impl(key, value)
    }

    /// Removes a value at the specified [`SlotMapKey`]. If there was no value corresponding
    /// to the [`SlotMapKey`] it returns None, otherwise Some(value).
    ///
    /// # Safety
    ///
    ///  * [`RelocatableSlotMap::init()`] must be called once before
    ///
    pub unsafe fn remove(&mut self, key: SlotMapKey) -> Option<T> {
        self.remove_impl(key)
    }

    /// Returns the [`SlotMapKey`] that will be used when the user calls
    /// [`SlotMap::insert()`]. If the [`SlotMap`] is full it returns [`None`].
    ///
    /// # Safety
    ///
    ///  * [`RelocatableSlotMap::init()`] must be called once before
    ///
    pub unsafe fn next_free_key(&self) -> Option<SlotMapKey> {
        self.next_free_key_impl()
    }

    /// Returns the number of stored values.
    pub fn len(&self) -> usize {
        self.len_impl()
    }

    /// Returns the capacity.
    pub fn capacity(&self) -> usize {
        self.capacity_impl()
    }

    /// Returns true if the container is empty, otherwise false.
    pub fn is_empty(&self) -> bool {
        self.is_empty_impl()
    }

    /// Returns true if the container is full, otherwise false.
    pub fn is_full(&self) -> bool {
        self.is_full_impl()
    }
}

/// A compile-time fixed-size, shared memory compatible [`FixedSizeSlotMap`].
#[repr(C)]
#[derive(Debug)]
pub struct FixedSizeSlotMap<T, const CAPACITY: usize> {
    state: RelocatableSlotMap<T>,
    _idx_to_data: MaybeUninit<[usize; CAPACITY]>,
    _idx_to_data_free_list: MaybeUninit<[FreeListEntry; CAPACITY]>,
    _data: MaybeUninit<[Option<T>; CAPACITY]>,
    _data_next_free_index: MaybeUninit<[usize; CAPACITY]>,
}

unsafe impl<T: ZeroCopySend, const CAPACITY: usize> ZeroCopySend for FixedSizeSlotMap<T, CAPACITY> {}

impl<T, const CAPACITY: usize> PlacementDefault for FixedSizeSlotMap<T, CAPACITY> {
    unsafe fn placement_default(ptr: *mut Self) {
        let state_ptr = core::ptr::addr_of_mut!((*ptr).state);
        state_ptr.write(unsafe { RelocatableSlotMap::new_uninit(CAPACITY) });
        let allocator = BumpAllocator::new((*ptr)._idx_to_data.as_mut_ptr().cast());
        (*ptr)
            .state
            .init(&allocator)
            .expect("All required memory is preallocated.");
    }
}

impl<T, const CAPACITY: usize> Default for FixedSizeSlotMap<T, CAPACITY> {
    fn default() -> Self {
        let mut new_self = Self {
            _idx_to_data: MaybeUninit::uninit(),
            _idx_to_data_free_list: MaybeUninit::uninit(),
            _data: MaybeUninit::uninit(),
            _data_next_free_index: MaybeUninit::uninit(),
            state: unsafe { RelocatableSlotMap::new_uninit(CAPACITY) },
        };

        let allocator = BumpAllocator::new(new_self._idx_to_data.as_mut_ptr().cast());
        unsafe {
            new_self
                .state
                .init(&allocator)
                .expect("All required memory is preallocated.")
        };

        new_self
    }
}

impl<T, const CAPACITY: usize> FixedSizeSlotMap<T, CAPACITY> {
    /// Creates a new empty [`FixedSizeSlotMap`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the [`RelocatableIter`]ator to iterate over all entries.
    pub fn iter(&self) -> RelocatableIter<'_, T> {
        unsafe { self.state.iter_impl() }
    }

    /// Returns `true` if the provided `key` is contained, otherwise `false`.
    pub fn contains(&self, key: SlotMapKey) -> bool {
        unsafe { self.state.contains_impl(key) }
    }

    /// Returns a reference to the value stored under the given key. If there is no such key,
    /// [`None`] is returned.
    pub fn get(&self, key: SlotMapKey) -> Option<&T> {
        unsafe { self.state.get_impl(key) }
    }

    /// Returns a mutable reference to the value stored under the given key. If there is no
    /// such key, [`None`] is returned.
    pub fn get_mut(&mut self, key: SlotMapKey) -> Option<&mut T> {
        unsafe { self.state.get_mut_impl(key) }
    }

    /// Insert a value and returns the corresponding [`SlotMapKey`]. If the container is full
    /// [`None`] is returned.
    pub fn insert(&mut self, value: T) -> Option<SlotMapKey> {
        unsafe { self.state.insert_impl(value) }
    }

    /// Insert a value at the specified [`SlotMapKey`] and returns true.  If the provided key
    /// is out-of-bounds it returns `false` and adds nothing. If there is already a value
    /// stored at the `key`s index, the value is overridden with the provided value.
    pub fn insert_at(&mut self, key: SlotMapKey, value: T) -> bool {
        unsafe { self.state.insert_at_impl(key, value) }
    }

    /// Removes a value at the specified [`SlotMapKey`]. If there was no value corresponding
    /// to the [`SlotMapKey`] it returns None, otherwise Some(value).
    pub fn remove(&mut self, key: SlotMapKey) -> Option<T> {
        unsafe { self.state.remove_impl(key) }
    }

    /// Returns the [`SlotMapKey`] that will be used when the user calls
    /// [`SlotMap::insert()`]. If the [`SlotMap`] is full it returns [`None`].
    pub fn next_free_key(&self) -> Option<SlotMapKey> {
        unsafe { self.state.next_free_key_impl() }
    }

    /// Returns the number of stored values.
    pub fn len(&self) -> usize {
        self.state.len_impl()
    }

    /// Returns the capacity.
    pub fn capacity(&self) -> usize {
        self.state.capacity_impl()
    }

    /// Returns true if the container is empty, otherwise false.
    pub fn is_empty(&self) -> bool {
        self.state.is_empty_impl()
    }

    /// Returns true if the container is full, otherwise false.
    pub fn is_full(&self) -> bool {
        self.state.is_full_impl()
    }
}
