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

//! Relocatable (inter-process shared memory compatible) string implementations.
//!
//! The [`StaticString`](crate::string::StaticString) has a fixed capacity defined at compile time.
//! It is memory-layout compatible to the C++ counterpart in the iceoryx2-bb-container C++ library
//! and can be used for zero-copy cross-language communication.
//!
//! # Example
//!
//! ```
//! # extern crate iceoryx2_bb_loggers;
//!
//! use iceoryx2_bb_container::string::*;
//!
//! const STRING_CAPACITY: usize = 123;
//!
//! let mut some_string = StaticString::<STRING_CAPACITY>::new();
//! some_string.push_bytes(b"hello").unwrap();
//! some_string.push('!' as u8).unwrap();
//! some_string.push('!' as u8).unwrap();
//!
//! println!("removed byte {:?}", some_string.remove(0));
//! ```

use alloc::format;
use core::str::FromStr;
use core::{
    cmp::Ordering,
    fmt::{Debug, Display},
    hash::Hash,
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
};

use iceoryx2_bb_derive_macros::{PlacementDefault, ZeroCopySend};
use iceoryx2_bb_elementary_traits::placement_default::PlacementDefault;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use iceoryx2_log::fail;
use serde::{de::Visitor, Deserialize, Deserializer, Serialize, Serializer};

use crate::string::{
    as_escaped_string, internal::StringView, strnlen, String, StringModificationError,
};

/// Variant of the [`String`] that has a compile-time fixed capacity and is
/// shared-memory compatible.
#[derive(PlacementDefault, ZeroCopySend, Clone, Copy)]
#[repr(C)]
pub struct StaticString<const CAPACITY: usize> {
    data: [MaybeUninit<u8>; CAPACITY],
    terminator: u8,
    len: u64,
}

impl<const CAPACITY: usize> Serialize for StaticString<CAPACITY> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(core::str::from_utf8(self.as_bytes()).unwrap())
    }
}

struct StaticStringVisitor<const CAPACITY: usize>;

impl<const CAPACITY: usize> Visitor<'_> for StaticStringVisitor<CAPACITY> {
    type Value = StaticString<CAPACITY>;

    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        formatter.write_str(&format!("a string with a length of at most {CAPACITY}"))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match StaticString::from_bytes(v.as_bytes()) {
            Ok(v) => Ok(v),
            Err(_) => Err(E::custom(format!(
                "the string exceeds the maximum length of {CAPACITY}"
            ))),
        }
    }
}

impl<'de, const CAPACITY: usize> Deserialize<'de> for StaticString<CAPACITY> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(StaticStringVisitor)
    }
}

unsafe impl<const CAPACITY: usize> Send for StaticString<CAPACITY> {}

impl<const CAPACITY: usize, const CAPACITY_OTHER: usize> PartialOrd<StaticString<CAPACITY_OTHER>>
    for StaticString<CAPACITY>
{
    fn partial_cmp(&self, other: &StaticString<CAPACITY_OTHER>) -> Option<Ordering> {
        self.as_bytes().partial_cmp(other.as_bytes())
    }
}

impl<const CAPACITY: usize> Ord for StaticString<CAPACITY> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_bytes().cmp(other.as_bytes())
    }
}

impl<const CAPACITY: usize> Hash for StaticString<CAPACITY> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        state.write(self.as_bytes())
    }
}

impl<const CAPACITY: usize> Deref for StaticString<CAPACITY> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.as_bytes()
    }
}

impl<const CAPACITY: usize> DerefMut for StaticString<CAPACITY> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_bytes()
    }
}

impl<const CAPACITY: usize, const OTHER_CAPACITY: usize> PartialEq<StaticString<OTHER_CAPACITY>>
    for StaticString<CAPACITY>
{
    fn eq(&self, other: &StaticString<OTHER_CAPACITY>) -> bool {
        *self.as_bytes() == *other.as_bytes()
    }
}

impl<const CAPACITY: usize> Eq for StaticString<CAPACITY> {}

impl<const CAPACITY: usize> PartialEq<&[u8]> for StaticString<CAPACITY> {
    fn eq(&self, other: &&[u8]) -> bool {
        *self.as_bytes() == **other
    }
}

impl<const CAPACITY: usize> PartialEq<&str> for StaticString<CAPACITY> {
    fn eq(&self, other: &&str) -> bool {
        *self.as_bytes() == *other.as_bytes()
    }
}

impl<const CAPACITY: usize> PartialEq<StaticString<CAPACITY>> for &str {
    fn eq(&self, other: &StaticString<CAPACITY>) -> bool {
        *self.as_bytes() == *other.as_bytes()
    }
}

impl<const CAPACITY: usize, const OTHER_CAPACITY: usize> PartialEq<[u8; OTHER_CAPACITY]>
    for StaticString<CAPACITY>
{
    fn eq(&self, other: &[u8; OTHER_CAPACITY]) -> bool {
        *self.as_bytes() == *other
    }
}

impl<const CAPACITY: usize, const OTHER_CAPACITY: usize> PartialEq<&[u8; OTHER_CAPACITY]>
    for StaticString<CAPACITY>
{
    fn eq(&self, other: &&[u8; OTHER_CAPACITY]) -> bool {
        *self.as_bytes() == **other
    }
}

impl<const CAPACITY: usize> Debug for StaticString<CAPACITY> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "StaticString<{}> {{ len: {}, data: \"{}\" }}",
            CAPACITY,
            self.len,
            as_escaped_string(self.as_bytes())
        )
    }
}

impl<const CAPACITY: usize> Display for StaticString<CAPACITY> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", as_escaped_string(self.as_bytes()))
    }
}

impl<const CAPACITY: usize> TryFrom<&str> for StaticString<CAPACITY> {
    type Error = StringModificationError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from(value.as_bytes())
    }
}

impl<const CAPACITY: usize, const N: usize> TryFrom<&[u8; N]> for StaticString<CAPACITY> {
    type Error = StringModificationError;

    fn try_from(value: &[u8; N]) -> Result<Self, Self::Error> {
        Self::try_from(value.as_slice())
    }
}

impl<const CAPACITY: usize> TryFrom<&[u8]> for StaticString<CAPACITY> {
    type Error = StringModificationError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if CAPACITY < value.len() {
            fail!(from "StaticString::from<&[u8]>()",
                with StringModificationError::InsertWouldExceedCapacity,
                "The provided string \"{}\" does not fit into the StaticString with capacity {}",
                as_escaped_string(value), CAPACITY);
        }

        let mut new_self = Self::new();
        new_self.push_bytes(value)?;
        Ok(new_self)
    }
}

impl<const CAPACITY: usize> Default for StaticString<CAPACITY> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const CAPACITY: usize> StringView for StaticString<CAPACITY> {
    fn data(&self) -> &[MaybeUninit<u8>] {
        &self.data
    }

    unsafe fn data_mut(&mut self) -> &mut [MaybeUninit<u8>] {
        &mut self.data
    }

    unsafe fn set_len(&mut self, len: u64) {
        self.len = len
    }
}

impl<const CAPACITY: usize> FromStr for StaticString<CAPACITY> {
    type Err = StringModificationError;

    fn from_str(s: &str) -> Result<Self, StringModificationError> {
        Self::from_bytes(s.as_bytes())
    }
}

impl<const CAPACITY: usize> StaticString<CAPACITY> {
    /// Creates a new and empty [`StaticString`]
    pub const fn new() -> Self {
        let mut new_self = Self {
            len: 0,
            data: unsafe { MaybeUninit::uninit().assume_init() },
            terminator: 0,
        };
        new_self.data[0] = MaybeUninit::new(0);
        new_self
    }

    /// Creates a new [`StaticString`]. The user has to ensure that the string can hold the
    /// bytes.
    ///
    /// # Safety
    ///
    ///  * `bytes` len must be smaller or equal than [`StaticString::capacity()`]
    ///  * all unicode code points must be smaller 128 and not 0.
    ///
    pub const unsafe fn from_bytes_unchecked_restricted(bytes: &[u8], len: usize) -> Self {
        debug_assert!(bytes.len() <= CAPACITY);
        debug_assert!(len <= bytes.len());

        let mut new_self = Self::new();
        core::ptr::copy_nonoverlapping(bytes.as_ptr(), new_self.data.as_mut_ptr().cast(), len);
        core::ptr::write::<u8>(new_self.data.as_mut_ptr().add(len).cast(), 0);
        new_self.len = len as u64;
        new_self
    }

    /// Creates a new [`StaticString`]. The user has to ensure that the string can hold the
    /// bytes.
    ///
    /// # Safety
    ///
    ///  * `bytes` len must be smaller or equal than [`StaticString::capacity()`]
    ///  * all unicode code points must be smaller 128 and not 0.
    ///
    pub const unsafe fn from_bytes_unchecked(bytes: &[u8]) -> Self {
        debug_assert!(bytes.len() <= CAPACITY);

        Self::from_bytes_unchecked_restricted(bytes, bytes.len())
    }

    /// Creates a new [`StaticString`] from a byte slice
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, StringModificationError> {
        let mut new_self = Self::new();
        new_self.insert_bytes(0, bytes)?;

        Ok(new_self)
    }

    /// Creates a new [`StaticString`] from a byte slice. If the byte slice does not fit
    /// into the [`StaticString`] it will be truncated.
    pub fn from_bytes_truncated(bytes: &[u8]) -> Result<Self, StringModificationError> {
        let mut new_self = Self::new();
        new_self.insert_bytes(0, &bytes[0..core::cmp::min(bytes.len(), CAPACITY)])?;
        Ok(new_self)
    }

    /// Creates a new [`StaticString`] from a string slice. If the string slice does not fit
    /// into the [`StaticString`] it will be truncated.
    pub fn from_str_truncated(s: &str) -> Result<Self, StringModificationError> {
        Self::from_bytes_truncated(s.as_bytes())
    }

    /// Creates a new byte string from a given null-terminated string
    ///
    /// # Safety
    ///
    ///  * `ptr` must point to a valid memory position
    ///  * `ptr` must be '\0' (null) terminated
    ///
    pub unsafe fn from_c_str(
        ptr: *const core::ffi::c_char,
    ) -> Result<Self, StringModificationError> {
        let string_length = strnlen(ptr, CAPACITY + 1);
        if CAPACITY < string_length {
            return Err(StringModificationError::InsertWouldExceedCapacity);
        }

        Self::from_bytes(core::slice::from_raw_parts(ptr.cast(), string_length))
    }

    /// Returns the capacity of the [`StaticString`]
    pub const fn capacity() -> usize {
        CAPACITY
    }

    /// Returns a slice to the underlying bytes
    pub const fn as_bytes_const(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.data.as_ptr().cast(), self.len as usize) }
    }
}

impl<const CAPACITY: usize> String for StaticString<CAPACITY> {
    fn capacity(&self) -> usize {
        CAPACITY
    }

    fn len(&self) -> usize {
        self.len as usize
    }
}

#[allow(missing_docs)]
pub struct StringMemoryLayoutMetrics {
    pub string_size: usize,
    pub string_alignment: usize,
    pub size_data: usize,
    pub offset_data: usize,
    pub size_len: usize,
    pub offset_len: usize,
    pub len_is_unsigned: bool,
}

trait _StringMemoryLayoutFieldLenInspection {
    fn is_unsigned(&self) -> bool;
}

impl _StringMemoryLayoutFieldLenInspection for u64 {
    fn is_unsigned(&self) -> bool {
        true
    }
}

impl StringMemoryLayoutMetrics {
    #[allow(missing_docs)]
    pub fn from_string<const CAPACITY: usize>(v: &StaticString<CAPACITY>) -> Self {
        StringMemoryLayoutMetrics {
            string_size: core::mem::size_of_val(v),
            string_alignment: core::mem::align_of_val(v),
            size_data: core::mem::size_of_val(&v.data) + core::mem::size_of_val(&v.terminator),
            offset_data: core::mem::offset_of!(StaticString<CAPACITY>, data),
            size_len: core::mem::size_of_val(&v.len),
            offset_len: core::mem::offset_of!(StaticString<CAPACITY>, len),
            len_is_unsigned: v.len.is_unsigned(),
        }
    }
}
