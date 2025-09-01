// Copyright (c) 2025 Contributors to the Eclipse Foundation
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

//! A FlatMap is a data structure to store key-value pairs. Multiple variations of that container
//! are available.
//!
//!  * [`FixedSizeFlatMap`](crate::flatmap::FixedSizeFlatMap), compile-time fixed-size flatmap
//!    that is self-contained and shared-memory compatible.
//!  * [`RelocatableFlatMap`](crate::flatmap::RelocatableFlatMap), run-time fixed-size flatmap that
//!    is shared-memory compatible.
//!  * [`FlatMap`](crate::flatmap::FlatMap), run-time fixed-size flatmap that is not shared-memory
//!    compatible since the memory resides in the heap.
//!
//! # User Examples
//!
//! ```
//! use iceoryx2_bb_container::flatmap::FixedSizeFlatMap;
//!
//! const CAPACITY: usize = 100;
//! let mut map = FixedSizeFlatMap::<u8, u8, CAPACITY>::new();
//! assert_eq!(map.insert(23, 4).is_ok(), true);
//! assert_eq!(map.get(&23).unwrap(), 4);
//! ```

use crate::slotmap::FreeListEntry;
use crate::slotmap::{MetaSlotMap, RelocatableSlotMap};
use core::fmt::Debug;
use core::mem::MaybeUninit;
use iceoryx2_bb_elementary::bump_allocator::BumpAllocator;
use iceoryx2_bb_elementary::relocatable_ptr::GenericRelocatablePointer;
use iceoryx2_bb_elementary_traits::generic_pointer::GenericPointer;
use iceoryx2_bb_elementary_traits::owning_pointer::GenericOwningPointer;
pub use iceoryx2_bb_elementary_traits::relocatable_container::RelocatableContainer;
use iceoryx2_bb_elementary_traits::{
    placement_default::PlacementDefault, zero_copy_send::ZeroCopySend,
};
use iceoryx2_bb_log::{fail, fatal_panic};
use iceoryx2_pal_concurrency_sync::iox_atomic::IoxAtomicBool;

/// Failures caused by insert()
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum FlatMapError {
    /// The FlatMap already contains the key that shall be inserted.
    KeyAlreadyExists,
    /// The FlatMap is full and cannot hold an additional key-value pair.
    IsFull,
}

#[repr(C)]
struct Entry<K: Eq, V: Clone> {
    id: K,
    value: V,
}

unsafe impl<K: Eq + ZeroCopySend, V: Clone + ZeroCopySend> ZeroCopySend for Entry<K, V> {}

/// A runtime fixed-size, non-shared memory compatible [`FlatMap`]. The [`FlatMap`]s memory resides
/// in the heap.
pub type FlatMap<K, V> = MetaFlatMap<K, V, GenericOwningPointer>;

/// A runtime fixed-size, shared-memory compatible [`RelocatableFlatMap`].
pub type RelocatableFlatMap<K, V> = MetaFlatMap<K, V, GenericRelocatablePointer>;

#[doc(hidden)]
#[repr(C)]
pub struct MetaFlatMap<K: Eq, V: Clone, Ptr: GenericPointer> {
    map: MetaSlotMap<Entry<K, V>, Ptr>,
    is_initialized: IoxAtomicBool,
}

impl<K: Eq + Debug, V: Clone + Debug, Ptr: GenericPointer> Debug for MetaFlatMap<K, V, Ptr> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "MetaFlatMap<{}, {}, {}> {{ len: {}, is_initialized: {} }}",
            core::any::type_name::<K>(),
            core::any::type_name::<V>(),
            core::any::type_name::<Ptr>(),
            self.len_impl(),
            self.is_initialized
                .load(core::sync::atomic::Ordering::Relaxed),
        )
    }
}

impl<K: Eq, V: Clone, Ptr: GenericPointer> MetaFlatMap<K, V, Ptr> {
    #[inline(always)]
    fn verify_init(&self, source: &str) {
        debug_assert!(
                self.is_initialized
                    .load(core::sync::atomic::Ordering::Relaxed),
                "From: MetaFlatMap<{}, {}>::{}, Undefined behavior - the object was not initialized with 'init' before.",
                core::any::type_name::<K>(), core::any::type_name::<V>(), source
            );
    }

    pub(crate) unsafe fn insert_impl(&mut self, id: K, value: V) -> Result<(), FlatMapError> {
        self.verify_init("insert()");

        let msg = "Unable to insert key-value pair into FlatMap";
        let origin = "MetaFlatMap::insert_impl()";

        let mut iter = self.map.iter_impl().skip_while(|kv| kv.1.id != id);
        if iter.next().is_some() {
            fail!(from origin, with FlatMapError::KeyAlreadyExists, "{msg} since the passed key already exists.");
        }
        if self.map.insert_impl(Entry { id, value }).is_none() {
            fail!(from origin, with FlatMapError::IsFull, "{msg} since the FlatMap is full.");
        }
        Ok(())
    }

    pub(crate) unsafe fn get_impl(&self, id: &K) -> Option<V> {
        self.verify_init("get()");

        self.get_ref_impl(id).cloned()
    }

    pub(crate) unsafe fn get_ref_impl(&self, id: &K) -> Option<&V> {
        self.verify_init("get_ref()");

        let mut iter = self.map.iter_impl().skip_while(|kv| kv.1.id != *id);
        iter.next().map(|kv| &kv.1.value)
    }

    pub(crate) unsafe fn get_mut_ref_impl(&mut self, id: &K) -> Option<&mut V> {
        self.verify_init("get_mut_ref()");

        let slot_map_entry = self.map.iter_impl().find(|kv| kv.1.id == *id)?;
        self.map
            .get_mut_impl(slot_map_entry.0)
            .map(|flat_map_entry| &mut flat_map_entry.value)
    }

    pub(crate) unsafe fn remove_impl(&mut self, id: &K) -> Option<V> {
        self.verify_init("remove()");

        let mut iter = self.map.iter_impl().skip_while(|kv| kv.1.id != *id);
        if let Some(kv) = iter.next() {
            let key = kv.0;
            self.map.remove_impl(key).map(|e| e.value)
        } else {
            None
        }
    }

    pub(crate) fn is_empty_impl(&self) -> bool {
        self.map.is_empty_impl()
    }

    pub(crate) fn is_full_impl(&self) -> bool {
        self.map.is_full_impl()
    }

    pub(crate) unsafe fn contains_impl(&self, id: &K) -> bool {
        self.verify_init("contains()");

        self.get_ref_impl(id).is_some()
    }

    pub(crate) fn len_impl(&self) -> usize {
        self.map.len_impl()
    }
}

impl<K: Eq, V: Clone> FlatMap<K, V> {
    /// Creates a new runtime-fixed size [`FlatMap`] on the heap with the given capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            map: MetaSlotMap::new(capacity),
            is_initialized: IoxAtomicBool::new(true),
        }
    }

    /// Inserts a new key-value pair into the [`FlatMap`]. On success, the method returns [`Ok`],
    /// otherwise a [`FlatMapError`] describing the failure.
    pub fn insert(&mut self, id: K, value: V) -> Result<(), FlatMapError> {
        unsafe { self.insert_impl(id, value) }
    }

    /// Returns a copy of the value corresponding to the given key. If there is no such key,
    /// [`None`] is returned.
    pub fn get(&self, id: &K) -> Option<V> {
        unsafe { self.get_impl(id) }
    }

    /// Returns a reference to the value corresponding to the given key. If there is no such
    /// key, [`None`] is returned.
    pub fn get_ref(&self, id: &K) -> Option<&V> {
        unsafe { self.get_ref_impl(id) }
    }

    /// Returns a mutable reference to the value corresponding to the given key. If there is
    /// no such key, [`None`] is returned.
    pub fn get_mut_ref(&mut self, id: &K) -> Option<&mut V> {
        unsafe { self.get_mut_ref_impl(id) }
    }

    /// Removes a key (`id`) from the [`FlatMap`], returning the Some(value) at the key if the key
    /// was previously in the map or [`None`] otherwise.
    pub fn remove(&mut self, id: &K) -> Option<V> {
        unsafe { self.remove_impl(id) }
    }

    /// Returns true if the [`FlatMap`] is empty, otherwise false.
    pub fn is_empty(&self) -> bool {
        self.is_empty_impl()
    }

    /// Returns true if the [`FlatMap`] is full, otherwise false.
    pub fn is_full(&self) -> bool {
        self.is_full_impl()
    }

    /// Returns true if the [`FlatMap`] contains the given key, otherwise false.
    pub fn contains(&self, id: &K) -> bool {
        unsafe { self.contains_impl(id) }
    }

    /// Returns the number of stored key-value pairs.
    pub fn len(&self) -> usize {
        self.len_impl()
    }
}

impl<K: Eq, V: Clone> RelocatableContainer for RelocatableFlatMap<K, V> {
    unsafe fn new_uninit(capacity: usize) -> Self {
        Self {
            map: RelocatableSlotMap::new_uninit(capacity),
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
            fatal_panic!(from "RelocatableFlatMap::init()", "Memory already initialized. Initializing it twice may lead to undefined behavior.");
        }
        let msg = "Unable to initialize RelocatableFlatMap";
        fail!(from "RelocatableFlatMap::init()", when self.map.init(allocator), "{msg} since the underlying RelocatableSlotMap could not be initialized.");
        self.is_initialized
            .store(true, core::sync::atomic::Ordering::Relaxed);
        Ok(())
    }

    fn memory_size(capacity: usize) -> usize {
        Self::const_memory_size(capacity)
    }
}

unsafe impl<K: Eq + ZeroCopySend, V: Clone + ZeroCopySend> ZeroCopySend
    for RelocatableFlatMap<K, V>
{
}

impl<K: Eq, V: Clone> RelocatableFlatMap<K, V> {
    /// Returns how much memory the [`RelocatableFlatMap`] will allocate from the allocator
    /// in [`RelocatableFlatMap::init()`].
    pub const fn const_memory_size(capacity: usize) -> usize {
        RelocatableSlotMap::<Entry<K, V>>::const_memory_size(capacity)
    }

    /// Inserts a new key-value pair into the map. On success, the method returns [`Ok`],
    /// otherwise a [`FlatMapError`] describing the failure.
    ///
    /// # Safety
    ///
    ///  * [`RelocatableFlatMap::init()`] must be called once before
    ///
    pub unsafe fn insert(&mut self, id: K, value: V) -> Result<(), FlatMapError> {
        self.insert_impl(id, value)
    }

    /// Returns a copy of the value corresponding to the given key. If there is no such key,
    /// [`None`] is returned.
    ///
    /// # Safety
    ///
    ///  * [`RelocatableFlatMap::init()`] must be called once before
    ///
    pub unsafe fn get(&self, id: &K) -> Option<V> {
        self.get_impl(id)
    }

    /// Returns a reference to the value corresponding to the given key. If there is no such
    /// key, [`None`] is returned.
    ///
    /// # Safety
    ///
    ///  * [`RelocatableFlatMap::init()`] must be called once before
    ///
    pub unsafe fn get_ref(&self, id: &K) -> Option<&V> {
        self.get_ref_impl(id)
    }

    /// Returns a mutable reference to the value corresponding to the given key. If there is
    /// no such key, [`None`] is returned.
    ///
    /// # Safety
    ///
    ///  * [`RelocatableFlatMap::init()`] must be called once before
    ///
    pub unsafe fn get_mut_ref(&mut self, id: &K) -> Option<&mut V> {
        self.get_mut_ref_impl(id)
    }

    /// Removes a key (`id`) from the map, returning the Some(value) at the key if the key
    /// was previously in the map or [`None`] otherwise.
    /// # Safety
    ///
    ///  * [`RelocatableFlatMap::init()`] must be called once before
    ///
    pub unsafe fn remove(&mut self, id: &K) -> Option<V> {
        self.remove_impl(id)
    }

    /// Returns true if the map is empty, otherwise false.
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Returns true if the map is full, otherwise false.
    pub fn is_full(&self) -> bool {
        self.map.is_full()
    }

    /// Returns true if the map contains the given key, otherwise false.
    ///
    /// # Safety
    ///
    ///  * [`RelocatableFlatMap::init()`] must be called once before
    ///
    pub unsafe fn contains(&self, id: &K) -> bool {
        self.contains_impl(id)
    }

    /// Returns the number of stored key-value pairs.
    pub fn len(&self) -> usize {
        self.map.len()
    }
}

/// A compile-time fixed-size, shared-memory compatible [`FixedSizeFlatMap`].
#[repr(C)]
pub struct FixedSizeFlatMap<K: Eq, V: Clone, const CAPACITY: usize> {
    map: RelocatableFlatMap<K, V>,
    _idx_to_data: MaybeUninit<[usize; CAPACITY]>,
    _idx_to_data_free_list: MaybeUninit<[FreeListEntry; CAPACITY]>,
    _data: MaybeUninit<[Option<Entry<K, V>>; CAPACITY]>,
    _data_next_free_index: MaybeUninit<[usize; CAPACITY]>,
}

unsafe impl<K: Eq + ZeroCopySend, V: Clone + ZeroCopySend, const CAPACITY: usize> ZeroCopySend
    for FixedSizeFlatMap<K, V, CAPACITY>
{
}

impl<K: Eq, V: Clone, const CAPACITY: usize> PlacementDefault for FixedSizeFlatMap<K, V, CAPACITY> {
    unsafe fn placement_default(ptr: *mut Self) {
        let map_ptr = core::ptr::addr_of_mut!((*ptr).map);
        map_ptr.write(unsafe { RelocatableFlatMap::new_uninit(CAPACITY) });
        let allocator = BumpAllocator::new((*ptr)._idx_to_data.as_mut_ptr().cast());
        (*ptr)
            .map
            .init(&allocator)
            .expect("All required memory is preallocated.");
    }
}

impl<K: Eq, V: Clone, const CAPACITY: usize> Default for FixedSizeFlatMap<K, V, CAPACITY> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Eq + Debug, V: Clone + Debug, const CAPACITY: usize> Debug
    for FixedSizeFlatMap<K, V, CAPACITY>
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "MetaFlatMap<{}, {}, {}> {{ {:?} }}",
            core::any::type_name::<K>(),
            core::any::type_name::<V>(),
            CAPACITY,
            self.map
        )
    }
}

impl<K: Eq, V: Clone, const CAPACITY: usize> FixedSizeFlatMap<K, V, CAPACITY> {
    /// Creates a new [`FixedSizeFlatMap`]
    pub fn new() -> Self {
        let mut new_self = Self {
            map: unsafe { RelocatableFlatMap::new_uninit(CAPACITY) },
            _idx_to_data: MaybeUninit::uninit(),
            _idx_to_data_free_list: MaybeUninit::uninit(),
            _data: MaybeUninit::uninit(),
            _data_next_free_index: MaybeUninit::uninit(),
        };
        let allocator = BumpAllocator::new(new_self._idx_to_data.as_mut_ptr().cast());
        unsafe {
            new_self
                .map
                .init(&allocator)
                .expect("All required memory is preallocated.")
        };
        new_self
    }

    /// Inserts a new key-value pair into the [`FixedSizeFlatMap`]. On success, the method returns [`Ok`],
    /// otherwise a [`FlatMapError`] describing the failure.
    pub fn insert(&mut self, id: K, value: V) -> Result<(), FlatMapError> {
        unsafe { self.map.insert(id, value) }
    }

    /// Returns a copy of the value corresponding to the given key. If there is no such key,
    /// [`None`] is returned.
    pub fn get(&self, id: &K) -> Option<V> {
        unsafe { self.map.get(id) }
    }

    /// Returns a reference to the value corresponding to the given key. If there is no such
    /// key, [`None`] is returned.
    pub fn get_ref(&self, id: &K) -> Option<&V> {
        unsafe { self.map.get_ref(id) }
    }

    /// Returns a mutable reference to the value corresponding to the given key. If there is
    /// no such key, [`None`] is returned.
    pub fn get_mut_ref(&mut self, id: &K) -> Option<&mut V> {
        unsafe { self.map.get_mut_ref(id) }
    }

    /// Removes a key (`id`) from the [`FixedSizeFlatMap`], returning the Some(value) at the key
    /// if the key was previously in the map or [`None`] otherwise.
    pub fn remove(&mut self, id: &K) -> Option<V> {
        unsafe { self.map.remove(id) }
    }

    /// Returns true if the [`FixedSizeFlatMap`] is empty, otherwise false.
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Returns true if the [`FixedSizeFlatMap`] is full, otherwise false.
    pub fn is_full(&self) -> bool {
        self.map.is_full()
    }

    /// Returns true if the [`FixedSizeFlatMap`] contains the given key, otherwise false.
    pub fn contains(&self, id: &K) -> bool {
        unsafe { self.map.contains(id) }
    }

    /// Returns the number of stored key-value pairs.
    pub fn len(&self) -> usize {
        self.map.len()
    }
}
