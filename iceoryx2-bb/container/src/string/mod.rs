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

/// Error which can occur when a [`String`] is modified.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum StringModificationError {
    /// The content that shall be added would exceed the maximum capacity of the
    /// [`String`].
    InsertWouldExceedCapacity,
}

impl core::fmt::Display for StringModificationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "StringModificationError::{self:?}")
    }
}

impl core::error::Error for StringModificationError {}

pub trait String {
    /// Returns a slice to the underlying bytes
    fn as_bytes(&self) -> &[u8];

    /// Returns a null-terminated slice to the underlying bytes
    fn as_bytes_with_nul(&self) -> &[u8];

    /// Returns a zero terminated slice of the underlying bytes
    fn as_c_str(&self) -> *const core::ffi::c_char;

    /// Returns a mutable slice to the underlying bytes
    fn as_mut_bytes(&mut self) -> &mut [u8];

    /// Returns the content as a string slice if the bytes are valid UTF-8
    fn as_str(&self) -> Result<&str, core::str::Utf8Error>;

    /// Returns the content as a string slice without checking for valid UTF-8
    ///
    /// # Safety
    ///
    ///  * must be valid utf-8
    ///
    unsafe fn as_str_unchecked(&self) -> &str;

    /// Returns the capacity of the string
    fn capacity(&self) -> usize;

    /// Removes all bytes from the string and set the len to zero
    fn clear(&mut self);

    /// Finds the first occurrence of a  byte string in the given string. If the byte string was
    /// found the start position of the byte string is returned, otherwise [`None`].
    fn find(&self, bytes: &[u8]) -> Option<usize>;

    /// True if the string is empty, otherwise false
    fn is_empty(&self) -> bool;

    /// True if the string is full, otherwise false.
    fn is_full(&self) -> bool;

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
    fn insert(&mut self, idx: usize, byte: u8) -> Result<(), StringModificationError>;

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
    fn insert_bytes(&mut self, idx: usize, bytes: &[u8]) -> Result<(), StringModificationError>;

    /// Inserts a byte array at a provided index.
    ///
    /// # Safety
    ///
    ///  * The 'idx' must by less than [`FixedSizeByteString::len()`].
    ///  * The 'bytes.len()' must be less or equal than [`FixedSizeByteString::capacity()`] -
    ///    [`FixedSizeByteString::len()`]
    ///
    unsafe fn insert_bytes_unchecked(&mut self, idx: usize, bytes: &[u8]);

    /// Returns the length of the string
    fn len(&self) -> usize;

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
    fn pop(&mut self) -> Option<u8>;

    /// Adds a byte at the end of the string. If there is no more space left it fails, otherwise
    /// it succeeds.
    fn push(&mut self, byte: u8) -> Result<(), StringModificationError>;

    /// Adds a byte array at the end of the string. If there is no more space left it fails, otherwise
    /// it succeeds.
    fn push_bytes(&mut self, bytes: &[u8]) -> Result<(), StringModificationError>;

    /// Removes a character at the provided index and returns it.
    fn remove(&mut self, idx: usize) -> u8;

    /// Removes a range beginning from idx.
    fn remove_range(&mut self, idx: usize, len: usize);

    /// Removes all characters where f(c) returns false.
    fn retain<F: FnMut(u8) -> bool>(&mut self, f: F);

    /// Finds the last occurrence of a byte string in the given string. If the byte string was
    /// found the start position of the byte string is returned, otherwise [`None`].
    fn rfind(&self, bytes: &[u8]) -> Option<usize>;

    /// Removes a given prefix from the string. If the prefix was not found it returns false,
    /// otherwise the prefix is removed and the function returns true.
    fn strip_prefix(&mut self, bytes: &[u8]) -> bool;

    /// Removes a given suffix from the string. If the suffix was not found it returns false,
    /// otherwise the suffix is removed and the function returns true.
    fn strip_suffix(&mut self, bytes: &[u8]) -> bool;

    /// Truncates the string to new_len.
    fn truncate(&mut self, new_len: usize);
}
