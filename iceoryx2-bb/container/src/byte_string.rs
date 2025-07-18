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
//! The [`FixedSizeByteString`](crate::byte_string::FixedSizeByteString) has a fixed capacity defined at compile time.
//!
//! # Example
//!
//! ```
//! use iceoryx2_bb_container::byte_string::*;
//!
//! const STRING_CAPACITY: usize = 123;
//!
//! let mut some_string = FixedSizeByteString::<STRING_CAPACITY>::new();
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

/// Returns the length of a string
///
/// # Safety
///
///  * The string must be '\0' (null) terminated.
///
pub unsafe fn strnlen(ptr: *const core::ffi::c_char, len: usize) -> usize {
    const NULL_TERMINATION: core::ffi::c_char = 0;
    for i in 0..len {
        if *ptr.add(i) == NULL_TERMINATION {
            return i;
        }
    }

    len
}

/// Error which can occur when a [`FixedSizeByteString`] is modified.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FixedSizeByteStringModificationError {
    /// The content that shall be added would exceed the maximum capacity of the
    /// [`FixedSizeByteString`].
    InsertWouldExceedCapacity,
}

impl core::fmt::Display for FixedSizeByteStringModificationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "FixedSizeByteStringModificationError::{self:?}")
    }
}

impl core::error::Error for FixedSizeByteStringModificationError {}

/// Relocatable string with compile time fixed size capacity.
#[derive(PlacementDefault, ZeroCopySend, Clone, Copy)]
#[repr(C)]
pub struct FixedSizeByteString<const CAPACITY: usize> {
    len: usize,
    data: [MaybeUninit<u8>; CAPACITY],
    terminator: u8,
}

impl<const CAPACITY: usize> Serialize for FixedSizeByteString<CAPACITY> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(core::str::from_utf8(self.as_bytes()).unwrap())
    }
}

struct FixedSizeByteStringVisitor<const CAPACITY: usize>;

impl<const CAPACITY: usize> Visitor<'_> for FixedSizeByteStringVisitor<CAPACITY> {
    type Value = FixedSizeByteString<CAPACITY>;

    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        formatter.write_str(&format!("a string with a length of at most {CAPACITY}"))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match FixedSizeByteString::from_bytes(v.as_bytes()) {
            Ok(v) => Ok(v),
            Err(_) => Err(E::custom(format!(
                "the string exceeds the maximum length of {CAPACITY}"
            ))),
        }
    }
}

impl<'de, const CAPACITY: usize> Deserialize<'de> for FixedSizeByteString<CAPACITY> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(FixedSizeByteStringVisitor)
    }
}

unsafe impl<const CAPACITY: usize> Send for FixedSizeByteString<CAPACITY> {}

impl<const CAPACITY: usize, const CAPACITY_OTHER: usize>
    PartialOrd<FixedSizeByteString<CAPACITY_OTHER>> for FixedSizeByteString<CAPACITY>
{
    fn partial_cmp(&self, other: &FixedSizeByteString<CAPACITY_OTHER>) -> Option<Ordering> {
        self.data[..self.len]
            .iter()
            .zip(other.data[..other.len].iter())
            .map(|(lhs, rhs)| unsafe { lhs.assume_init_read().cmp(rhs.assume_init_ref()) })
            .find(|&ord| ord != Ordering::Equal)
            .or(Some(self.len.cmp(&other.len)))
    }
}

impl<const CAPACITY: usize> Ord for FixedSizeByteString<CAPACITY> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl<const CAPACITY: usize> Hash for FixedSizeByteString<CAPACITY> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        state.write(self.as_bytes())
    }
}

impl<const CAPACITY: usize> Deref for FixedSizeByteString<CAPACITY> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.as_bytes()
    }
}

impl<const CAPACITY: usize> DerefMut for FixedSizeByteString<CAPACITY> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_bytes()
    }
}

impl<const CAPACITY: usize, const OTHER_CAPACITY: usize>
    PartialEq<FixedSizeByteString<OTHER_CAPACITY>> for FixedSizeByteString<CAPACITY>
{
    fn eq(&self, other: &FixedSizeByteString<OTHER_CAPACITY>) -> bool {
        *self.as_bytes() == *other.as_bytes()
    }
}

impl<const CAPACITY: usize> Eq for FixedSizeByteString<CAPACITY> {}

impl<const CAPACITY: usize> PartialEq<&[u8]> for FixedSizeByteString<CAPACITY> {
    fn eq(&self, other: &&[u8]) -> bool {
        *self.as_bytes() == **other
    }
}

impl<const CAPACITY: usize> PartialEq<&str> for FixedSizeByteString<CAPACITY> {
    fn eq(&self, other: &&str) -> bool {
        *self.as_bytes() == *other.as_bytes()
    }
}

impl<const CAPACITY: usize> PartialEq<FixedSizeByteString<CAPACITY>> for &str {
    fn eq(&self, other: &FixedSizeByteString<CAPACITY>) -> bool {
        *self.as_bytes() == *other.as_bytes()
    }
}

impl<const CAPACITY: usize, const OTHER_CAPACITY: usize> PartialEq<[u8; OTHER_CAPACITY]>
    for FixedSizeByteString<CAPACITY>
{
    fn eq(&self, other: &[u8; OTHER_CAPACITY]) -> bool {
        *self.as_bytes() == *other
    }
}

impl<const CAPACITY: usize, const OTHER_CAPACITY: usize> PartialEq<&[u8; OTHER_CAPACITY]>
    for FixedSizeByteString<CAPACITY>
{
    fn eq(&self, other: &&[u8; OTHER_CAPACITY]) -> bool {
        *self.as_bytes() == **other
    }
}

impl<const CAPACITY: usize> Debug for FixedSizeByteString<CAPACITY> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "FixedSizeByteString<{}> {{ len: {}, data: \"{}\" }}",
            CAPACITY,
            self.len,
            as_escaped_string(self.as_bytes())
        )
    }
}

impl<const CAPACITY: usize> Display for FixedSizeByteString<CAPACITY> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", as_escaped_string(self.as_bytes()))
    }
}

impl<const CAPACITY: usize, const BYTE_CAPACITY: usize> From<&[u8; BYTE_CAPACITY]>
    for FixedSizeByteString<CAPACITY>
{
    fn from(value: &[u8; BYTE_CAPACITY]) -> Self {
        if CAPACITY < BYTE_CAPACITY {
            fatal_panic!(from "FixedSizeByteString::from<[u8; ..]>()", "The byte array does not fit into the FixedSizeByteString");
        }

        let mut new_self = Self::new();
        new_self.push_bytes(value).unwrap();
        new_self
    }
}

impl<const CAPACITY: usize> TryFrom<&str> for FixedSizeByteString<CAPACITY> {
    type Error = FixedSizeByteStringModificationError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if CAPACITY < value.len() {
            fail!(from "FixedSizeByteString::from<&str>()",
                with FixedSizeByteStringModificationError::InsertWouldExceedCapacity,
                "The provided string \"{}\" does not fit into the FixedSizeByteString with capacity {}",
                value, CAPACITY);
        }

        let mut new_self = Self::new();
        new_self.push_bytes(value.as_bytes()).unwrap();
        Ok(new_self)
    }
}

impl<const CAPACITY: usize> Default for FixedSizeByteString<CAPACITY> {
    fn default() -> Self {
        Self::new()
    }
}

/// Adds escape characters to the string so that it can be used for console output.
pub fn as_escaped_string(bytes: &[u8]) -> String {
    String::from_utf8(
        bytes
            .iter()
            .flat_map(|c| match *c {
                b'\t' => vec![b'\\', b't'].into_iter(),
                b'\r' => vec![b'\\', b'r'].into_iter(),
                b'\n' => vec![b'\\', b'n'].into_iter(),
                b'\x20'..=b'\x7e' => vec![*c].into_iter(),
                _ => {
                    let hex_digits: &[u8; 16] = b"0123456789abcdef";
                    vec![
                        b'\\',
                        b'x',
                        hex_digits[(c >> 4) as usize],
                        hex_digits[(c & 0xf) as usize],
                    ]
                    .into_iter()
                }
            })
            .collect::<Vec<u8>>(),
    )
    .unwrap()
}

impl<const CAPACITY: usize> FixedSizeByteString<CAPACITY> {
    /// Creates a new and empty [`FixedSizeByteString`]
    pub const fn new() -> Self {
        let mut new_self = Self {
            len: 0,
            data: unsafe { MaybeUninit::uninit().assume_init() },
            terminator: 0,
        };
        new_self.data[0] = MaybeUninit::new(0);
        new_self
    }

    /// Creates a new [`FixedSizeByteString`]. The user has to ensure that the string can hold the
    /// bytes.
    ///
    /// # Safety
    ///
    ///  * `bytes` len must be smaller or equal than [`FixedSizeByteString::capacity()`]
    ///
    pub unsafe fn new_unchecked(bytes: &[u8]) -> Self {
        if CAPACITY < bytes.len() {
            panic!("Insufficient capacity to store bytes.");
        }

        Self::from_bytes_truncated(bytes)
    }

    /// Creates a new [`FixedSizeByteString`] from a byte slice
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, FixedSizeByteStringModificationError> {
        let mut new_self = Self::new();
        fail!(from "FixedSizeByteString", when new_self.push_bytes(bytes),
                with FixedSizeByteStringModificationError::InsertWouldExceedCapacity,
                "Unbale to create from \"{}\" since it would exceed the capacity of {}.",
                as_escaped_string(bytes), CAPACITY);

        Ok(new_self)
    }

    /// Creates a new [`FixedSizeByteString`] from a byte slice. If the byte slice does not fit
    /// into the [`FixedSizeByteString`] it will be truncated.
    pub fn from_bytes_truncated(bytes: &[u8]) -> Self {
        let mut new_self = Self::new();
        new_self.len = core::cmp::min(bytes.len(), CAPACITY);
        for (i, byte) in bytes.iter().enumerate().take(new_self.len) {
            new_self.data[i].write(*byte);
        }

        if new_self.len < CAPACITY {
            new_self.data[new_self.len].write(0);
        }

        new_self
    }

    /// Creates a new [`FixedSizeByteString`] from a string slice. If the string slice does not fit
    /// into the [`FixedSizeByteString`] it will be truncated.
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
    ) -> Result<Self, FixedSizeByteStringModificationError> {
        let string_length = strnlen(ptr, CAPACITY + 1);
        if CAPACITY < string_length {
            return Err(FixedSizeByteStringModificationError::InsertWouldExceedCapacity);
        }

        let mut new_self = Self::new();
        core::ptr::copy_nonoverlapping(
            ptr,
            new_self.as_mut_bytes().as_mut_ptr() as *mut core::ffi::c_char,
            string_length,
        );
        new_self.len = string_length;

        Ok(new_self)
    }

    /// Returns a slice to the underlying bytes
    pub const fn as_bytes(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.data.as_ptr() as *const u8, self.len) }
    }

    /// Returns a null-terminated slice to the underlying bytes
    pub const fn as_bytes_with_nul(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.data.as_ptr() as *const u8, self.len + 1) }
    }

    /// Returns a zero terminated slice of the underlying bytes
    pub const fn as_c_str(&self) -> *const core::ffi::c_char {
        self.data.as_ptr() as *const core::ffi::c_char
    }

    /// Returns a mutable slice to the underlying bytes
    pub fn as_mut_bytes(&mut self) -> &mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(self.data.as_mut_ptr() as *mut u8, self.len) }
    }

    /// Returns the content as a string slice if the bytes are valid UTF-8
    pub fn as_str(&self) -> Result<&str, core::str::Utf8Error> {
        core::str::from_utf8(self.as_bytes())
    }

    /// Returns the content as a string slice without checking for valid UTF-8
    ///
    /// # Safety
    ///
    ///  * must be valid utf-8
    ///
    pub unsafe fn as_str_unchecked(&self) -> &str {
        core::str::from_utf8_unchecked(self.as_bytes())
    }
    /// Returns the capacity of the string
    pub const fn capacity() -> usize {
        CAPACITY
    }

    /// Returns the length of the string
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Removes all bytes from the string and set the len to zero
    pub fn clear(&mut self) {
        self.len = 0;
        self.data[0].write(0);
    }

    /// True if the string is empty, otherwise false
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// True if the string is full, otherwise false.
    pub const fn is_full(&self) -> bool {
        self.len == CAPACITY
    }

    /// Inserts a byte at a provided index. If the index is out of bounds it panics.
    /// If the string has no more capacity left it fails otherwise it succeeds.
    ///
    /// ```
    /// use iceoryx2_bb_container::byte_string::*;
    ///
    /// const STRING_CAPACITY: usize = 123;
    ///
    /// let mut some_string = FixedSizeByteString::<STRING_CAPACITY>::from(b"helo");
    /// some_string.insert(3, 'l' as u8).unwrap();
    /// assert!(some_string == b"hello");
    /// ```
    pub fn insert(
        &mut self,
        idx: usize,
        byte: u8,
    ) -> Result<(), FixedSizeByteStringModificationError> {
        self.insert_bytes(idx, &[byte; 1])
    }

    /// Inserts a byte array at a provided index. If the index is out of bounds it panics.
    /// If the string has no more capacity left it fails otherwise it succeeds.
    ///
    /// ```
    /// use iceoryx2_bb_container::byte_string::*;
    ///
    /// const STRING_CAPACITY: usize = 123;
    ///
    /// let mut some_string = FixedSizeByteString::<STRING_CAPACITY>::from(b"ho");
    /// some_string.insert_bytes(1, b"ell").unwrap();
    /// assert!(some_string == b"hello");
    /// ```
    pub fn insert_bytes(
        &mut self,
        idx: usize,
        bytes: &[u8],
    ) -> Result<(), FixedSizeByteStringModificationError> {
        let msg = "Unable to insert byte string";
        if self.len < idx {
            fatal_panic!(from self, "{} \"{}\" since the index {} is out of bounds.",
                msg, as_escaped_string(bytes) , idx);
        }

        if CAPACITY < self.len + bytes.len() {
            fail!(from self, with FixedSizeByteStringModificationError::InsertWouldExceedCapacity,
                "{} \"{}\" since it would exceed the maximum capacity of {}.",
                msg, as_escaped_string(bytes), CAPACITY);
        }

        unsafe { self.insert_bytes_unchecked(idx, bytes) };

        Ok(())
    }

    /// Inserts a byte array at a provided index.
    ///
    /// # Safety
    ///
    ///  * The 'idx' must by less than [`FixedSizeByteString::len()`].
    ///  * The 'bytes.len()' must be less or equal than [`FixedSizeByteString::capacity()`] -
    ///    [`FixedSizeByteString::len()`]
    ///
    pub unsafe fn insert_bytes_unchecked(&mut self, idx: usize, bytes: &[u8]) {
        unsafe {
            core::ptr::copy(
                self.data[idx].as_ptr(),
                self.data[idx].as_mut_ptr().add(bytes.len()),
                self.len - idx,
            );
        }

        for (i, byte) in bytes.iter().enumerate() {
            self.data[idx + i].write(*byte);
        }

        self.len += bytes.len();
        if self.len < CAPACITY {
            self.data[self.len].write(0);
        }
    }

    /// Removes the last character from the string and returns it. If the string is empty it
    /// returns none.
    /// ```
    /// use iceoryx2_bb_container::byte_string::*;
    ///
    /// const STRING_CAPACITY: usize = 123;
    ///
    /// let mut some_string = FixedSizeByteString::<STRING_CAPACITY>::from(b"hello!");
    /// let char = some_string.pop().unwrap();
    ///
    /// assert!(char == '!' as u8);
    /// assert!(some_string == b"hello");
    /// ```
    pub fn pop(&mut self) -> Option<u8> {
        if self.is_empty() {
            return None;
        }

        Some(self.remove(self.len - 1))
    }

    /// Adds a byte at the end of the string. If there is no more space left it fails, otherwise
    /// it succeeds.
    pub fn push(&mut self, byte: u8) -> Result<(), FixedSizeByteStringModificationError> {
        self.insert(self.len, byte)
    }

    /// Adds a byte array at the end of the string. If there is no more space left it fails, otherwise
    /// it succeeds.
    pub fn push_bytes(&mut self, bytes: &[u8]) -> Result<(), FixedSizeByteStringModificationError> {
        self.insert_bytes(self.len, bytes)
    }

    /// Removes a character at the provided index and returns it.
    pub fn remove(&mut self, idx: usize) -> u8 {
        if self.len < idx {
            fatal_panic!(from self, "Unable to remove byte at position {} since it is out of bounds.",
                idx);
        }

        let removed_byte = unsafe { *self.data[idx].as_ptr() };

        self.remove_range_impl(idx, 1);

        removed_byte
    }

    /// Removes a range beginning from idx.
    pub fn remove_range(&mut self, idx: usize, len: usize) {
        if self.len < idx + len {
            fatal_panic!(from self, "Unable to remove range from position {} with length {} since it is out of bounds.",
                idx, len);
        }

        self.remove_range_impl(idx, len)
    }

    fn remove_range_impl(&mut self, idx: usize, len: usize) {
        unsafe {
            core::ptr::copy(
                self.data[idx + len].as_ptr(),
                self.data[idx].as_mut_ptr(),
                self.len - (idx + len),
            );
        }

        self.len -= len;
        self.data[self.len].write(0);
    }

    /// Removes all characters where f(c) returns false.
    pub fn retain<F: FnMut(u8) -> bool>(&mut self, f: F) {
        self.retain_impl(f);
    }

    pub(crate) fn retain_impl<F: FnMut(u8) -> bool>(&mut self, mut f: F) -> F {
        let len = self.len;
        for i in 0..len {
            let idx = len - i - 1;
            if f(unsafe { *self.data[idx].as_ptr() }) {
                self.remove(idx);
            }
        }
        f
    }

    /// Finds the first occurrence of a  byte string in the given string. If the byte string was
    /// found the start position of the byte string is returned, otherwise [`None`].
    pub fn find(&self, bytes: &[u8]) -> Option<usize> {
        if self.len() < bytes.len() {
            return None;
        }

        for i in 0..self.len() - bytes.len() + 1 {
            let mut has_found = true;
            for (n, byte) in bytes.iter().enumerate() {
                if unsafe { *self.data[i + n].as_ptr() } != *byte {
                    has_found = false;
                    break;
                }
            }

            if has_found {
                return Some(i);
            }
        }

        None
    }

    /// Finds the last occurrence of a byte string in the given string. If the byte string was
    /// found the start position of the byte string is returned, otherwise [`None`].
    pub fn rfind(&self, bytes: &[u8]) -> Option<usize> {
        if self.len() < bytes.len() {
            return None;
        }

        for i in (0..self.len() - bytes.len() + 1).rev() {
            let mut has_found = true;
            for (n, byte) in bytes.iter().enumerate() {
                if unsafe { *self.data[i + n].as_ptr() } != *byte {
                    has_found = false;
                    break;
                }
            }

            if has_found {
                return Some(i);
            }
        }

        None
    }

    /// Removes a given prefix from the string. If the prefix was not found it returns false,
    /// otherwise the prefix is removed and the function returns true.
    pub fn strip_prefix(&mut self, bytes: &[u8]) -> bool {
        match self.find(bytes) {
            Some(0) => {
                self.remove_range(0, bytes.len());
                true
            }
            _ => false,
        }
    }

    /// Removes a given suffix from the string. If the suffix was not found it returns false,
    /// otherwise the suffix is removed and the function returns true.
    pub fn strip_suffix(&mut self, bytes: &[u8]) -> bool {
        if self.len() < bytes.len() {
            return false;
        }

        let pos = self.len() - bytes.len();
        match self.rfind(bytes) {
            Some(v) => {
                if v != pos {
                    return false;
                }
                self.remove_range(pos, bytes.len());
                true
            }
            None => false,
        }
    }

    /// Truncates the string to new_len.
    pub fn truncate(&mut self, new_len: usize) {
        if self.len() < new_len {
            return;
        }

        self.len = new_len;
        if self.len < CAPACITY {
            self.data[self.len].write(0u8);
        }
    }
}
