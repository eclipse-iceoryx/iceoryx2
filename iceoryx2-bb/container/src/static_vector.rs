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

// TODO: document, drop order is from last element to first (reverse order)

use core::{fmt::Debug, mem::MaybeUninit};
use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use iceoryx2_bb_elementary_traits::{
    placement_default::PlacementDefault, zero_copy_send::ZeroCopySend,
};
use iceoryx2_bb_log::fatal_panic;
use serde::{de::Visitor, Deserialize, Serialize};

/// Relocatable vector with compile time fixed size capacity.
#[repr(C)]
pub struct StaticVec<T, const CAPACITY: usize> {
    data: [MaybeUninit<T>; CAPACITY],
    len: u64,
}

impl<T: Debug, const CAPACITY: usize> Debug for StaticVec<T, CAPACITY> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "StaticVec<{}, {}> {{ {:?} }}",
            core::any::type_name::<T>(),
            CAPACITY,
            //todo
            self.data
        )
    }
}

unsafe impl<T: ZeroCopySend, const CAPACITY: usize> ZeroCopySend for StaticVec<T, CAPACITY> {}

impl<'de, T: Serialize + Deserialize<'de>, const CAPACITY: usize> Serialize
    for StaticVec<T, CAPACITY>
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.as_slice().serialize(serializer)
    }
}

struct StaticVecVisitor<T, const CAPACITY: usize> {
    _value: PhantomData<T>,
}

impl<'de, T: Deserialize<'de>, const CAPACITY: usize> Visitor<'de>
    for StaticVecVisitor<T, CAPACITY>
{
    type Value = StaticVec<T, CAPACITY>;

    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        let str = format!(
            "an array of at most {} elements of type {}",
            CAPACITY,
            core::any::type_name::<T>()
        );
        formatter.write_str(&str)
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut new_vec = Self::Value::new();

        while let Some(element) = seq.next_element()? {
            if !new_vec.push(element) {
                return Err(<A::Error as serde::de::Error>::custom(format!(
                    "the array can hold at most {CAPACITY} elements"
                )));
            }
        }

        Ok(new_vec)
    }
}

impl<'de, T: Deserialize<'de>, const CAPACITY: usize> Deserialize<'de> for StaticVec<T, CAPACITY> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(StaticVecVisitor::<T, CAPACITY> {
            _value: PhantomData,
        })
    }
}

impl<T, const CAPACITY: usize> PlacementDefault for StaticVec<T, CAPACITY> {
    unsafe fn placement_default(ptr: *mut Self) {
        core::ptr::addr_of_mut!((*ptr).len).write(0);
        core::ptr::addr_of_mut!((*ptr).data).write([const { MaybeUninit::uninit() }; CAPACITY]);
    }
}

impl<T, const CAPACITY: usize> Default for StaticVec<T, CAPACITY> {
    fn default() -> Self {
        Self {
            len: 0,
            data: [const { MaybeUninit::uninit() }; CAPACITY],
        }
    }
}

impl<T, const CAPACITY: usize> Deref for StaticVec<T, CAPACITY> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T, const CAPACITY: usize> DerefMut for StaticVec<T, CAPACITY> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}

impl<T: PartialEq, const CAPACITY: usize> PartialEq for StaticVec<T, CAPACITY> {
    fn eq(&self, other: &Self) -> bool {
        if self.len != other.len {
            return false;
        }

        for i in 0..self.len() {
            if other[i] != self[i] {
                return false;
            }
        }

        true
    }
}

impl<T: Eq, const CAPACITY: usize> Eq for StaticVec<T, CAPACITY> {}

impl<T: Clone, const CAPACITY: usize> Clone for StaticVec<T, CAPACITY> {
    fn clone(&self) -> Self {
        Self {
            len: self.len.clone(),
            data: {
                let mut data = [const { MaybeUninit::uninit() }; CAPACITY];
                for idx in 0..self.len() {
                    data[idx].write(unsafe { self.data[idx].assume_init_ref() }.clone());
                }
                data
            },
        }
    }
}

unsafe impl<T: Send, const CAPACITY: usize> Send for StaticVec<T, CAPACITY> {}

impl<T, const CAPACITY: usize> StaticVec<T, CAPACITY> {
    /// Returns a mutable slice to the contents of the vector
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        let len = self.len();
        unsafe {
            core::mem::transmute::<&mut [MaybeUninit<T>], &mut [T]>(
                &mut self.data.as_mut_slice()[0..len],
            )
        }
    }

    /// Returns a slice to the contents of the vector
    pub fn as_slice(&self) -> &[T] {
        let len = self.len();
        unsafe { core::mem::transmute::<&[MaybeUninit<T>], &[T]>(&self.data.as_slice()[0..len]) }
    }

    /// Returns the capacity of the vector
    pub const fn capacity() -> usize {
        CAPACITY
    }

    /// Removes all elements from the vector
    pub fn clear(&mut self) {
        for idx in (0..self.len()).rev() {
            unsafe { self.data[idx].assume_init_drop() };
        }

        self.len = 0;
    }

    /// Append all elements from other via [`Clone`].
    pub fn extend_from_slice(&mut self, other: &[T]) -> bool
    where
        T: Clone,
    {
        if Self::capacity() < self.len() + other.len() {
            return false;
        }

        for (i, element) in other.iter().enumerate() {
            self.data[i + self.len()].write(element.clone());
        }

        self.len += other.len() as u64;

        true
    }

    /// Fill the remaining space of the vector with value.
    pub fn fill(&mut self, value: T)
    where
        T: Clone,
    {
        for idx in self.len()..Self::capacity() {
            self.data[idx].write(value.clone());
        }
    }

    /// Fill the remaining space of the vector with value.
    pub fn fill_with<F: FnMut() -> T>(&mut self, mut f: F) {
        for idx in self.len()..Self::capacity() {
            self.data[idx].write(f());
        }
    }

    /// Returns true if the vector is empty, otherwise false
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns true if the vector is full, otherwise false
    pub fn is_full(&self) -> bool {
        self.len == CAPACITY as u64
    }

    /// Returns the number of elements stored inside the vector
    pub fn len(&self) -> usize {
        self.len as usize
    }

    /// Creates a new vector.
    pub fn new() -> Self {
        Self::default()
    }

    /// Removes the last element of the vector and returns it to the user. If the vector is empty
    /// it returns [`None`].
    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }

        let value = core::mem::replace(&mut self.data[self.len() - 1], MaybeUninit::uninit());
        self.len -= 1;
        Some(unsafe { value.assume_init() })
    }

    /// Adds an element at the end of the vector. If the vector is full and the element cannot be
    /// added it returns false, otherwise true.
    pub fn push(&mut self, value: T) -> bool {
        if self.is_full() {
            return false;
        }

        self.data[self.len()].write(value);
        self.len += 1;
        true
    }

    /// Removes the element at the provided index and returns it.
    pub fn remove(&mut self, index: usize) -> T {
        if self.len() <= index {
            let origin = format!(
                "StaticVec::<{}, {}>::remove()",
                core::any::type_name::<T>(),
                CAPACITY
            );
            fatal_panic!(from origin,
                "Out-of-bounds access of index {index} while the size of the vector is {}",
                self.len());
        }

        let value = unsafe { core::ptr::read(self.data[index].as_ptr()) };

        unsafe {
            core::ptr::copy(
                self.data[index + 1].as_ptr(),
                self.data[index].as_mut_ptr(),
                self.len() - index - 1,
            )
        };

        value
    }
}
