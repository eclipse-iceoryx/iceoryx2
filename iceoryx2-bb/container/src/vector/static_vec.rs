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

//! Relocatable shared-memory compatible vector with compile time fixed size
//! capacity. It is memory-layout compatible to the C++ container in the
//! iceoryx2-bb-container C++ library and can be used for zero-copy
//! cross-language communication.
//!
//! # Example
//!
//! ```
//! # extern crate iceoryx2_bb_loggers;
//!
//! use iceoryx2_bb_container::vector::*;
//!
//! const CAPACITY: usize = 123;
//! let mut my_vec = StaticVec::<usize, CAPACITY>::new();
//!
//! my_vec.push(123); // returns false, when capacity is exceeded
//! ```

use alloc::format;
use core::{fmt::Debug, mem::MaybeUninit};
use core::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use iceoryx2_bb_elementary_traits::{
    placement_default::PlacementDefault, zero_copy_send::ZeroCopySend,
};
use iceoryx2_log::fail;
use serde::{de::Visitor, Deserialize, Serialize};

pub use crate::vector::Vector;
use crate::vector::{internal, VectorModificationError};

/// Relocatable shared-memory compatible vector with compile time fixed size
/// capacity. It is memory-layout compatible to the C++ container in the
/// iceoryx2-bb-container C++ library and can be used for zero-copy
/// cross-language communication.
///
/// In contrast to the Rust [`alloc::vec::Vec`] it has a defined reverse drop order.
#[repr(C)]
pub struct StaticVec<T, const CAPACITY: usize> {
    data: [MaybeUninit<T>; CAPACITY],
    len: u64,
}

impl<T: Debug, const CAPACITY: usize> Debug for StaticVec<T, CAPACITY> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "StaticVec<{}, {}> {{ len: {}, content: [ ",
            core::any::type_name::<T>(),
            CAPACITY,
            self.len,
        )?;

        if !self.is_empty() {
            write!(f, "{:?}", self[0])?;
        }

        for idx in 1..self.len() {
            write!(f, ", {:?}", self[idx])?;
        }

        write!(f, " ] }}")
    }
}

unsafe impl<T: ZeroCopySend, const CAPACITY: usize> ZeroCopySend for StaticVec<T, CAPACITY> {}

impl<T, const CAPACITY: usize> Drop for StaticVec<T, CAPACITY> {
    fn drop(&mut self) {
        self.clear();
    }
}

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
            if new_vec.push(element).is_err() {
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
        // We do not have to initialize the `MaybeUninit` array at all, see:
        // https://google.github.io/learn_unsafe_rust/advanced_unsafety/uninitialized.html
        // core::ptr::addr_of_mut!((*ptr).data).write([const { MaybeUninit::uninit() }; CAPACITY]);
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

impl<T: Clone, const CAPACITY: usize> TryFrom<&[T]> for StaticVec<T, CAPACITY> {
    type Error = VectorModificationError;
    fn try_from(value: &[T]) -> Result<Self, Self::Error> {
        if CAPACITY < value.len() {
            let origin = format!(
                "StaticVec::<{}, {}>::try_from()",
                core::any::type_name::<T>(),
                CAPACITY
            );
            fail!(from origin, with VectorModificationError::InsertWouldExceedCapacity,
                "Failed to create the vector since the slice len {} is greater than the vectors capacity.",
                value.len());
        }

        let mut new_self = Self::new();
        unsafe { new_self.extend_from_slice_unchecked(value) };
        Ok(new_self)
    }
}

impl<T: Eq, const CAPACITY: usize> Eq for StaticVec<T, CAPACITY> {}

impl<T: Clone, const CAPACITY: usize> Clone for StaticVec<T, CAPACITY> {
    fn clone(&self) -> Self {
        Self {
            len: self.len,
            data: {
                let mut data = [const { MaybeUninit::uninit() }; CAPACITY];
                for (idx, item) in data.iter_mut().enumerate().take(self.len()) {
                    item.write(unsafe { self.data[idx].assume_init_ref() }.clone());
                }
                data
            },
        }
    }
}

unsafe impl<T: Send, const CAPACITY: usize> Send for StaticVec<T, CAPACITY> {}

impl<T, const CAPACITY: usize> StaticVec<T, CAPACITY> {
    /// Creates a new vector.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the capacity of the vector
    pub const fn capacity() -> usize {
        CAPACITY
    }
}

impl<T, const CAPACITY: usize> internal::VectorView<T> for StaticVec<T, CAPACITY> {
    fn data(&self) -> &[MaybeUninit<T>] {
        &self.data
    }

    unsafe fn data_mut(&mut self) -> &mut [MaybeUninit<T>] {
        &mut self.data
    }

    unsafe fn set_len(&mut self, len: u64) {
        self.len = len
    }
}

impl<T, const CAPACITY: usize> Vector<T> for StaticVec<T, CAPACITY> {
    fn capacity(&self) -> usize {
        CAPACITY
    }

    fn len(&self) -> usize {
        self.len as usize
    }
}

#[allow(missing_docs)]
pub struct VectorMemoryLayoutMetrics {
    pub vector_size: usize,
    pub vector_alignment: usize,
    pub size_data: usize,
    pub offset_data: usize,
    pub size_len: usize,
    pub offset_len: usize,
    pub len_is_unsigned: bool,
}

trait _VectorMemoryLayoutFieldLenInspection {
    fn is_unsigned(&self) -> bool;
}

impl _VectorMemoryLayoutFieldLenInspection for u64 {
    fn is_unsigned(&self) -> bool {
        true
    }
}

impl VectorMemoryLayoutMetrics {
    #[allow(missing_docs)]
    pub fn from_vector<T, const CAPACITY: usize>(v: &StaticVec<T, CAPACITY>) -> Self {
        VectorMemoryLayoutMetrics {
            vector_size: core::mem::size_of_val(v),
            vector_alignment: core::mem::align_of_val(v),
            size_data: core::mem::size_of_val(&v.data),
            offset_data: core::mem::offset_of!(StaticVec<T,CAPACITY>, data),
            size_len: core::mem::size_of_val(&v.len),
            offset_len: core::mem::offset_of!(StaticVec<T, CAPACITY>, len),
            len_is_unsigned: v.len.is_unsigned(),
        }
    }
}
