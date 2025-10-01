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

//! Relocatable (inter-process shared memory compatible) string implementations.
//!
//! The [`StaticString`](crate::byte_string::StaticString) has a fixed capacity defined at compile time.
//!
//! # Example
//!
//! ```
//! use iceoryx2_bb_container::byte_string::*;
//!
//! const STRING_CAPACITY: usize = 123;
//!
//! let mut some_string = StaticString::<STRING_CAPACITY>::new();
//! some_string.push_bytes(b"hello").unwrap();
//! some_string.push('!' as u8).unwrap();
//! some_string.push('!' as u8).unwrap();
//!
//! println!("removed byte {}", some_string.remove(0));
//! ```

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
use iceoryx2_bb_log::{fail, fatal_panic};
use serde::{de::Visitor, Deserialize, Deserializer, Serialize, Serializer};

use crate::string::{as_escaped_string, internal::StringView, strnlen, String};

/// Error which can occur when a [`StaticString`] is modified.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum StaticStringModificationError {
    /// A string containing Unicode code points greater or equal 128 (U+0080) was provided
    /// for insertion or creation.
    InvalidCharacter,
    /// The content that shall be added would exceed the maximum capacity of the
    /// [`StaticString`].
    InsertWouldExceedCapacity,
}

impl core::fmt::Display for StaticStringModificationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "StaticStringModificationError::{self:?}")
    }
}

impl core::error::Error for StaticStringModificationError {}

/// A UTF-8 string with fixed static capacity and contiguous inplace storage.
/// The string class uses Unicode (ISO/IEC 10646) terminology throughout its interface. In particular:
/// - A code point is the numerical index assigned to a character in the Unicode standard.
/// - A code unit is the basic component of a character encoding system. For UTF-8, the code unit has a size of 8-bits
/// For example, the code point U+0041 represents the letter 'A' and can be encoded in a single 8-bit code unit in
/// UTF-8. The code point U+1F4A9 requires four 8-bit code units in the UTF-8 encoding.
///
/// The NUL code point (U+0000) is not allowed anywhere in the string.
///
/// ## Note
///
/// Currently only Unicode code points less than 128 (U+0080) are supported.
/// This restricts the valid contents of a string to those UTF8 strings
/// that are also valid 7-bit ASCII strings. Full Unicode support will get added later.
///
/// `Capacity` - Maximum number of UTF-8 code units that the string can store, excluding the terminating NUL character.
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
        self.data[..self.len as usize]
            .iter()
            .zip(other.data[..other.len as usize].iter())
            .map(|(lhs, rhs)| unsafe { lhs.assume_init_read().cmp(rhs.assume_init_ref()) })
            .find(|&ord| ord != Ordering::Equal)
            .or(Some(self.len.cmp(&other.len)))
    }
}

impl<const CAPACITY: usize> Ord for StaticString<CAPACITY> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
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

impl<const CAPACITY: usize, const BYTE_CAPACITY: usize> From<&[u8; BYTE_CAPACITY]>
    for StaticString<CAPACITY>
{
    fn from(value: &[u8; BYTE_CAPACITY]) -> Self {
        if CAPACITY < BYTE_CAPACITY {
            fatal_panic!(from "StaticString::from<[u8; ..]>()", "The byte array does not fit into the StaticString");
        }

        let mut new_self = Self::new();
        new_self.push_bytes(value).unwrap();
        new_self
    }
}

impl<const CAPACITY: usize> TryFrom<&str> for StaticString<CAPACITY> {
    type Error = StaticStringModificationError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if CAPACITY < value.len() {
            fail!(from "StaticString::from<&str>()",
                with StaticStringModificationError::InsertWouldExceedCapacity,
                "The provided string \"{}\" does not fit into the StaticString with capacity {}",
                value, CAPACITY);
        }

        let mut new_self = Self::new();
        new_self.push_bytes(value.as_bytes()).unwrap();
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
    ///
    pub unsafe fn from_bytes_unchecked(bytes: &[u8]) -> Self {
        debug_assert!(CAPACITY < bytes.len());

        Self::from_bytes_truncated(bytes)
    }

    /// Creates a new [`StaticString`] from a byte slice
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, StaticStringModificationError> {
        let mut new_self = Self::new();
        fail!(from "StaticString", when new_self.push_bytes(bytes),
                with StaticStringModificationError::InsertWouldExceedCapacity,
                "Unbale to create from \"{}\" since it would exceed the capacity of {}.",
                as_escaped_string(bytes), CAPACITY);

        Ok(new_self)
    }

    /// Creates a new [`StaticString`] from a byte slice. If the byte slice does not fit
    /// into the [`StaticString`] it will be truncated.
    pub fn from_bytes_truncated(bytes: &[u8]) -> Self {
        let mut new_self = Self::new();
        new_self.len = core::cmp::min(bytes.len(), CAPACITY) as u64;
        for (i, byte) in bytes.iter().enumerate().take(new_self.len()) {
            new_self.data[i].write(*byte);
        }

        if new_self.len() < CAPACITY {
            new_self.data[new_self.len()].write(0);
        }

        new_self
    }

    /// Creates a new [`StaticString`] from a string slice. If the string slice does not fit
    /// into the [`StaticString`] it will be truncated.
    pub fn from_str_truncated(s: &str) -> Self {
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
    ) -> Result<Self, StaticStringModificationError> {
        let string_length = strnlen(ptr, CAPACITY + 1);
        if CAPACITY < string_length {
            return Err(StaticStringModificationError::InsertWouldExceedCapacity);
        }

        let mut new_self = Self::new();
        core::ptr::copy_nonoverlapping(
            ptr,
            new_self.as_mut_bytes().as_mut_ptr() as *mut core::ffi::c_char,
            string_length,
        );
        new_self.len = string_length as u64;

        Ok(new_self)
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
