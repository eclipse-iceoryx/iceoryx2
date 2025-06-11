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

// TODO: documentation
// TODO: rename FlatMap to FixedSizeFlatMap

use crate::slotmap::FixedSizeSlotMap;
// TODO: check if extensions work (old include)
use iceoryx2_bb_elementary_traits::{
    placement_default::PlacementDefault, zero_copy_send::ZeroCopySend,
};
use iceoryx2_bb_log::fail;

/// Failures caused by [`FlatMap::insert()`]
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum FlatMapError {
    /// The FlatMap already contains the key that shall be inserted
    KeyAlreadyExists,
    /// The FlatMap is full and cannot hold an additional key-value pair
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
pub struct FlatMap<K: Eq + ZeroCopySend, V: Clone + ZeroCopySend, const CAPACITY: usize> {
    map: FixedSizeSlotMap<Entry<K, V>, CAPACITY>,
}

unsafe impl<K: Eq + ZeroCopySend, V: Clone + ZeroCopySend, const CAPACITY: usize> ZeroCopySend
    for FlatMap<K, V, CAPACITY>
{
}

impl<K: Eq + ZeroCopySend, V: Clone + ZeroCopySend, const CAPACITY: usize> PlacementDefault
    for FlatMap<K, V, CAPACITY>
{
    unsafe fn placement_default(ptr: *mut Self) {
        let map_ptr = core::ptr::addr_of_mut!((*ptr).map);
        PlacementDefault::placement_default(map_ptr);
    }
}

impl<K: Eq + ZeroCopySend, V: Clone + ZeroCopySend, const CAPACITY: usize> Default
    for FlatMap<K, V, CAPACITY>
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Eq + ZeroCopySend, V: Clone + ZeroCopySend, const CAPACITY: usize> FlatMap<K, V, CAPACITY> {
    /// Creates a new FlatMap
    pub fn new() -> Self {
        Self {
            map: FixedSizeSlotMap::new(),
        }
    }

    /// Inserts a new key-value pair into the FlatMap. On success, the method returns [`Ok`],
    /// otherwise a [`FlatMapError`] describing the failure.
    pub fn insert(&mut self, id: K, value: V) -> Result<(), FlatMapError> {
        let msg = "Unable to insert key-value pair into FlatMap";
        let origin = "FlatMap::insert()";

        let mut iter = self.map.iter().skip_while(|kv| kv.1.id != id);
        if iter.next().is_some() {
            fail!(from origin, with FlatMapError::KeyAlreadyExists, "{msg} since the passed key already exists.");
        }
        if self.map.insert(Entry { id, value }).is_none() {
            fail!(from origin, with FlatMapError::IsFull, "{msg} since the FlatMap is full.");
        }
        Ok(())
    }

    /// Returns a copy of the value corresponding to the given key. If there is no such key, [`None`] is returned.
    pub fn get(&self, id: &K) -> Option<V> {
        let mut iter = self.map.iter().skip_while(|kv| kv.1.id != *id);
        iter.next().map(|kv| kv.1.value.clone())
    }

    /// Removes the given key and the corresponding value from the FlatMap.
    pub fn remove(&mut self, id: &K) {
        let mut iter = self.map.iter().skip_while(|kv| kv.1.id != *id);
        if let Some(kv) = iter.next() {
            let key = kv.0;
            self.map.remove(key);
        }
    }

    /// Returns true if the FlatMap is empty, otherwise false.
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Returns true if the FlatMap is full, otherwise false.
    pub fn is_full(&self) -> bool {
        self.map.is_full()
    }

    /// Returns true if the FlatMap contains the given key, otherwise false.
    pub fn contains(&self, id: &K) -> bool {
        let mut iter = self.map.iter().skip_while(|kv| kv.1.id != *id);
        iter.next().is_some()
    }

    /// Returns the number of stored key-value pairs.
    pub fn len(&self) -> usize {
        self.map.len()
    }
}
