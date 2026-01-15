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

use core::mem::MaybeUninit;
use core::{
    fmt::Debug,
    fmt::Display,
    hash::Hash,
    ops::{Deref, DerefMut},
};
use iceoryx2_log::{fail, fatal_panic};

/// Runtime fixed-capacity string where the user can provide a stateful allocator.
pub mod polymorphic_string;

/// Compile-time fixed-capacity string variant that is shared-memory compatible.
pub mod static_string;

/// Runtime fixed-capacity shared-memory compatible string
pub mod relocatable_string;

/// String helper functions
pub mod utils;

pub use polymorphic_string::*;
pub use relocatable_string::*;
pub use static_string::*;
pub use utils::*;

/// Error which can occur when a [`String`] is modified.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum StringModificationError {
    /// A string with unsupported unicode code points greater or equal 128 (U+0080) was provided
    InvalidCharacter,
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

#[doc(hidden)]
pub(crate) mod internal {
    use super::*;

    #[doc(hidden)]
    pub trait StringView {
        fn data(&self) -> &[MaybeUninit<u8>];

        /// # Safety
        ///
        /// * user must ensure that any modification keeps the initialized data contiguous
        /// * user must update len with [`StringView::set_len()`] when adding/removing elements
        unsafe fn data_mut(&mut self) -> &mut [MaybeUninit<u8>];

        /// # Safety
        ///
        /// * user must ensure that the len defines the number of initialized contiguous
        ///   elements in [`StringView::data_mut()`] and [`StringView::data()`]
        unsafe fn set_len(&mut self, len: u64);
    }
}

/// A UTF-8 string trait.
/// The string class uses Unicode (ISO/IEC 10646) terminology throughout its interface. In particular:
///   - A code point is the numerical index assigned to a character in the Unicode standard.
///   - A code unit is the basic component of a character encoding system. For UTF-8, the code unit has a size of 8-bits
///
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
pub trait String:
    internal::StringView
    + Debug
    + Display
    + PartialOrd
    + Ord
    + Hash
    + Deref<Target = [u8]>
    + DerefMut
    + PartialEq
    + Eq
{
    /// Returns a slice to the underlying bytes
    fn as_bytes(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.data().as_ptr() as *const u8, self.len()) }
    }

    /// Returns a null-terminated slice to the underlying bytes
    fn as_bytes_with_nul(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.data().as_ptr() as *const u8, self.len() + 1) }
    }

    /// Returns a zero terminated slice of the underlying bytes
    fn as_c_str(&self) -> *const core::ffi::c_char {
        self.data().as_ptr() as *const core::ffi::c_char
    }

    /// Returns a mutable slice to the underlying bytes
    fn as_mut_bytes(&mut self) -> &mut [u8] {
        unsafe {
            core::slice::from_raw_parts_mut(self.data_mut().as_mut_ptr() as *mut u8, self.len())
        }
    }

    /// Returns the content as a string slice if the bytes are valid UTF-8
    fn as_str(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(self.as_bytes()) }
    }

    /// Returns the capacity of the string
    fn capacity(&self) -> usize;

    /// Removes all bytes from the string and set the len to zero
    fn clear(&mut self) {
        unsafe { self.set_len(0) };
        unsafe { self.data_mut()[0].write(0) };
    }

    /// Finds the first occurrence of a  byte string in the given string. If the byte string was
    /// found the start position of the byte string is returned, otherwise [`None`].
    fn find(&self, bytes: &[u8]) -> Option<usize> {
        if self.len() < bytes.len() {
            return None;
        }

        for i in 0..self.len() - bytes.len() + 1 {
            let mut has_found = true;
            for (n, byte) in bytes.iter().enumerate() {
                if unsafe { *self.data()[i + n].as_ptr() } != *byte {
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

    /// True if the string is empty, otherwise false
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// True if the string is full, otherwise false.
    fn is_full(&self) -> bool {
        self.len() == self.capacity()
    }

    /// Inserts a byte at a provided index. If the index is out of bounds it panics.
    /// If the string has no more capacity left it fails otherwise it succeeds.
    ///
    /// ```
    /// # extern crate iceoryx2_bb_loggers;
    ///
    /// use iceoryx2_bb_container::string::*;
    ///
    /// const STRING_CAPACITY: usize = 123;
    ///
    /// let mut some_string = StaticString::<STRING_CAPACITY>::from_bytes(b"helo").unwrap();
    /// some_string.insert(3, 'l' as u8).unwrap();
    /// assert!(some_string == b"hello");
    /// ```
    fn insert(&mut self, idx: usize, byte: u8) -> Result<(), StringModificationError> {
        self.insert_bytes(idx, &[byte; 1])
    }

    /// Inserts a byte array at a provided index. If the index is out of bounds it panics.
    /// If the string has no more capacity left it fails otherwise it succeeds.
    ///
    /// ```
    /// # extern crate iceoryx2_bb_loggers;
    ///
    /// use iceoryx2_bb_container::string::*;
    ///
    /// const STRING_CAPACITY: usize = 123;
    ///
    /// let mut some_string = StaticString::<STRING_CAPACITY>::from_bytes(b"ho").unwrap();
    /// some_string.insert_bytes(1, b"ell").unwrap();
    /// assert!(some_string == b"hello");
    /// ```
    fn insert_bytes(&mut self, idx: usize, bytes: &[u8]) -> Result<(), StringModificationError> {
        let msg = "Unable to insert byte string";
        if self.len() < idx {
            fatal_panic!(from self, "{} \"{}\" since the index {} is out of bounds.",
                msg, as_escaped_string(bytes) , idx);
        }

        if self.capacity() < self.len() + bytes.len() {
            fail!(from self, with StringModificationError::InsertWouldExceedCapacity,
                "{} \"{}\" since it would exceed the maximum capacity of {}.",
                msg, as_escaped_string(bytes), self.capacity());
        }

        for byte in bytes {
            if 128 <= *byte || 0 == *byte {
                fail!(from self, with StringModificationError::InvalidCharacter,
                    "{} \"{}\" since it contains unsupported unicode points. Only unicode points less than 128 (U+0080) are supported",
                    msg, as_escaped_string(bytes));
            }
        }

        unsafe { self.insert_bytes_unchecked(idx, bytes) };

        Ok(())
    }

    /// Inserts a byte array at a provided index.
    ///
    /// # Safety
    ///
    ///  * The 'idx' must by less than [`String::len()`].
    ///  * The 'bytes.len()' must be less or equal than [`String::capacity()`] -
    ///    [`String::len()`]
    ///
    unsafe fn insert_bytes_unchecked(&mut self, idx: usize, bytes: &[u8]) {
        let data = unsafe { self.data_mut() };
        let ptr = data.as_mut_ptr();
        unsafe {
            core::ptr::copy(ptr.add(idx), ptr.add(idx + bytes.len()), self.len() - idx);
        }

        for (i, byte) in bytes.iter().enumerate() {
            self.data_mut()[idx + i].write(*byte);
        }

        let new_len = self.len() + bytes.len();
        self.set_len(new_len as u64);
        if new_len < self.capacity() {
            self.data_mut()[new_len].write(0);
        }
    }

    /// Returns the length of the string
    fn len(&self) -> usize;

    /// Removes the last character from the string and returns it. If the string is empty it
    /// returns none.
    /// ```
    /// # extern crate iceoryx2_bb_loggers;
    ///
    /// use iceoryx2_bb_container::string::*;
    ///
    /// const STRING_CAPACITY: usize = 123;
    ///
    /// let mut some_string = StaticString::<STRING_CAPACITY>::from_bytes(b"hello!").unwrap();
    /// let char = some_string.pop().unwrap();
    ///
    /// assert!(char == '!' as u8);
    /// assert!(some_string == b"hello");
    /// ```
    fn pop(&mut self) -> Option<u8> {
        if self.is_empty() {
            return None;
        }

        self.remove(self.len() - 1)
    }

    /// Adds a byte at the end of the string. If there is no more space left it fails, otherwise
    /// it succeeds.
    fn push(&mut self, byte: u8) -> Result<(), StringModificationError> {
        self.insert(self.len(), byte)
    }

    /// Adds a byte array at the end of the string. If there is no more space left it fails, otherwise
    /// it succeeds.
    fn push_bytes(&mut self, bytes: &[u8]) -> Result<(), StringModificationError> {
        self.insert_bytes(self.len(), bytes)
    }

    /// Removes a character at the provided index and returns it.
    fn remove(&mut self, idx: usize) -> Option<u8> {
        if self.len() < idx {
            return None;
        }

        let removed_byte = unsafe { *self.data()[idx].as_ptr() };

        self.remove_range(idx, 1);

        Some(removed_byte)
    }

    /// Removes a range beginning from idx.
    fn remove_range(&mut self, idx: usize, len: usize) -> bool {
        if self.len() < idx + len {
            return false;
        }

        if self.len() != idx + len {
            let data = unsafe { self.data_mut() };
            let ptr = data.as_mut_ptr();
            unsafe {
                core::ptr::copy(ptr.add(idx + len), ptr.add(idx), self.len() - (idx + len));
            }
        }

        let new_len = self.len() - len;
        unsafe { self.data_mut()[new_len].write(0) };
        unsafe { self.set_len(new_len as u64) };

        true
    }

    /// Removes all characters where f(c) returns false.
    fn retain<F: FnMut(u8) -> bool>(&mut self, mut f: F) {
        let len = self.len();
        for idx in (0..len).rev() {
            if f(unsafe { *self.data()[idx].as_ptr() }) {
                self.remove(idx);
            }
        }
    }

    /// Finds the last occurrence of a byte string in the given string. If the byte string was
    /// found the start position of the byte string is returned, otherwise [`None`].
    fn rfind(&self, bytes: &[u8]) -> Option<usize> {
        if self.len() < bytes.len() {
            return None;
        }

        for i in (0..self.len() - bytes.len() + 1).rev() {
            let mut has_found = true;
            for (n, byte) in bytes.iter().enumerate() {
                if unsafe { *self.data()[i + n].as_ptr() } != *byte {
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
    fn strip_prefix(&mut self, bytes: &[u8]) -> bool {
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
    fn strip_suffix(&mut self, bytes: &[u8]) -> bool {
        if self.len() < bytes.len() {
            return false;
        }

        let pos = self.len() - bytes.len();
        match self.rfind(bytes) {
            Some(v) => {
                if v != pos {
                    return false;
                }
                self.remove_range(pos, bytes.len())
            }
            None => false,
        }
    }

    /// Truncates the string to new_len.
    fn truncate(&mut self, new_len: usize) {
        if self.len() < new_len {
            return;
        }

        if new_len < self.capacity() {
            unsafe { self.data_mut()[new_len].write(0u8) };
        }
        unsafe { self.set_len(new_len as u64) };
    }
}
