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

//! A FlatMap is a data structure to store key-value pairs. The
//! [`FixedSizeFlatMap`](crate::flatmap::FixedSizeFlatMap) is a compile-time fixed-size flatmap
//! that is self-contained and shared-memory compatible.
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

use crate::slotmap::FixedSizeSlotMap;
use iceoryx2_bb_elementary_traits::{
    placement_default::PlacementDefault, zero_copy_send::ZeroCopySend,
};
use iceoryx2_bb_log::fail;

/// Failures caused by [`FixedSizeFlatMap::insert()`]
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

/// A data structure to store key-value pairs
#[repr(C)]
pub struct FixedSizeFlatMap<K: Eq + ZeroCopySend, V: Clone + ZeroCopySend, const CAPACITY: usize> {
    map: FixedSizeSlotMap<Entry<K, V>, CAPACITY>,
}

unsafe impl<K: Eq + ZeroCopySend, V: Clone + ZeroCopySend, const CAPACITY: usize> ZeroCopySend
    for FixedSizeFlatMap<K, V, CAPACITY>
{
}

impl<K: Eq + ZeroCopySend, V: Clone + ZeroCopySend, const CAPACITY: usize> PlacementDefault
    for FixedSizeFlatMap<K, V, CAPACITY>
{
    unsafe fn placement_default(ptr: *mut Self) {
        let map_ptr = core::ptr::addr_of_mut!((*ptr).map);
        PlacementDefault::placement_default(map_ptr);
    }
}

impl<K: Eq + ZeroCopySend, V: Clone + ZeroCopySend, const CAPACITY: usize> Default
    for FixedSizeFlatMap<K, V, CAPACITY>
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Eq + ZeroCopySend, V: Clone + ZeroCopySend, const CAPACITY: usize>
    FixedSizeFlatMap<K, V, CAPACITY>
{
    /// Creates a new FixedSizeFlatMap
    pub fn new() -> Self {
        Self {
            map: FixedSizeSlotMap::new(),
        }
    }

    /// Inserts a new key-value pair into the FixedSizeFlatMap. On success, the method returns [`Ok`],
    /// otherwise a [`FlatMapError`] describing the failure.
    pub fn insert(&mut self, id: K, value: V) -> Result<(), FlatMapError> {
        let msg = "Unable to insert key-value pair into FixedSizeFlatMap";
        let origin = "FixedSizeFlatMap::insert()";

        let mut iter = self.map.iter().skip_while(|kv| kv.1.id != id);
        if iter.next().is_some() {
            fail!(from origin, with FlatMapError::KeyAlreadyExists, "{msg} since the passed key already exists.");
        }
        if self.map.insert(Entry { id, value }).is_none() {
            fail!(from origin, with FlatMapError::IsFull, "{msg} since the FixedSizeFlatMap is full.");
        }
        Ok(())
    }

    /// Returns a copy of the value corresponding to the given key. If there is no such key, [`None`] is returned.
    pub fn get(&self, id: &K) -> Option<V> {
        let mut iter = self.map.iter().skip_while(|kv| kv.1.id != *id);
        iter.next().map(|kv| kv.1.value.clone())
    }

    /// Returns a reference to the value corresponding to the given key. If there is no such key, [`None`] is returned.
    pub fn get_ref(&self, id: &K) -> Option<&V> {
        let mut iter = self.map.iter().skip_while(|kv| kv.1.id != *id);
        iter.next().map(|kv| &kv.1.value)
    }

    /// Returns a mutable reference to the value corresponding to the given key. If there is no such key, [`None`] is returned.
    pub fn get_mut_ref(&mut self, id: &K) -> Option<&mut V> {
        let slot_map_entry = self.map.iter().find(|kv| kv.1.id == *id)?;
        self.map
            .get_mut(slot_map_entry.0)
            .map(|flat_map_entry| &mut flat_map_entry.value)
    }

    /// Removes the given key and the corresponding value from the FixedSizeFlatMap.
    pub fn remove(&mut self, id: &K) {
        let mut iter = self.map.iter().skip_while(|kv| kv.1.id != *id);
        if let Some(kv) = iter.next() {
            let key = kv.0;
            self.map.remove(key);
        }
    }

    /// Returns true if the FixedSizeFlatMap is empty, otherwise false.
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Returns true if the FixedSizeFlatMap is full, otherwise false.
    pub fn is_full(&self) -> bool {
        self.map.is_full()
    }

    /// Returns true if the FixedSizeFlatMap contains the given key, otherwise false.
    pub fn contains(&self, id: &K) -> bool {
        let mut iter = self.map.iter().skip_while(|kv| kv.1.id != *id);
        iter.next().is_some()
    }

    /// Returns the number of stored key-value pairs.
    pub fn len(&self) -> usize {
        self.map.len()
    }
}
