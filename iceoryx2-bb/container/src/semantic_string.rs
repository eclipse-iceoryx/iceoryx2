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

use crate::byte_string::FixedSizeByteStringModificationError;
use crate::byte_string::{as_escaped_string, strlen, FixedSizeByteString};
use iceoryx2_bb_elementary::enum_gen;
use iceoryx2_bb_log::fail;
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::ops::Deref;

enum_gen! {SemanticStringError
  entry:
    InvalidCharacter,
    InvalidName

  generalization:
    ExceedsMaximumLength <= FixedSizeByteStringModificationError
}

impl std::fmt::Display for SemanticStringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::write!(f, "{}::{:?}", std::stringify!(Self), self)
    }
}

impl std::error::Error for SemanticStringError {}

pub trait SemanticStringAccessor<const CAPACITY: usize> {
    /// Creates a new empty SemanticStringAccessor which may violates the content contract.
    ///
    /// # Safety
    ///
    ///  * The user must set the value to a valid value before starting to work with the
    ///    [`SemanticString`].
    ///
    unsafe fn new_empty() -> Self;

    ///
    /// # Safety
    ///
    ///  * Ensure that the modification result is not violating the content contract
    ///
    unsafe fn get_mut_string(&mut self) -> &mut FixedSizeByteString<CAPACITY>;

    fn as_string(&self) -> &FixedSizeByteString<CAPACITY>;

    fn is_invalid_content(string: &[u8]) -> bool;
    fn does_contain_invalid_characters(string: &[u8]) -> bool;
}

/// Lists the operations all path representing strings share
pub trait SemanticString<const CAPACITY: usize>:
    SemanticStringAccessor<CAPACITY>
    + Debug
    + Display
    + Sized
    + Deref<Target = [u8]>
    + PartialEq
    + Eq
    + Hash
{
    /// Creates a new name. If it contains invalid characters or exceeds the maximum supported
    /// length of the system or contains illegal strings it fails.
    fn new(value: &[u8]) -> Result<Self, SemanticStringError> {
        let msg = "Unable to create SemanticString";
        let origin = "SemanticString::new()";

        let mut new_self = unsafe { <Self as SemanticStringAccessor<CAPACITY>>::new_empty() };
        fail!(from origin, when new_self.push_bytes(value),
            "{} due to an invalid value \"{}\".", msg, as_escaped_string(value));

        Ok(new_self)
    }

    /// Creates a new name from a given ptr. The user has to ensure that it is null-terminated.
    ///
    /// # Safety
    ///
    ///  * The pointer must be '\0' (null) terminated
    ///  * The pointer must be valid and non-null
    ///
    unsafe fn from_c_str(ptr: *mut std::ffi::c_char) -> Result<Self, SemanticStringError> {
        Self::new(std::slice::from_raw_parts(ptr as *const u8, strlen(ptr)))
    }

    /// Returns the contents as a slice
    fn as_bytes(&self) -> &[u8] {
        self.as_string().as_bytes()
    }

    /// Returns a zero terminated slice of the underlying bytes
    fn as_c_str(&self) -> *const std::ffi::c_char {
        self.as_string().as_c_str()
    }

    /// Returns the capacity of the file system type
    fn capacity(&self) -> usize {
        self.as_string().capacity()
    }

    /// Returns true when the string is full, otherwise false
    fn is_full(&self) -> bool {
        self.as_string().is_full()
    }

    /// Returns true when the string is empty, otherwise false
    fn is_empty(&self) -> bool {
        self.as_string().is_empty()
    }

    /// Returns the length of the string
    fn len(&self) -> usize {
        self.as_string().len()
    }

    /// Inserts a single byte at a specific position. When the capacity is exceeded, the byte is an
    /// illegal character or the content would result in an illegal name it fails.
    fn insert(&mut self, idx: usize, byte: u8) -> Result<(), SemanticStringError> {
        self.insert_bytes(idx, &[byte; 1])
    }

    /// Inserts a byte slice at a specific position. When the capacity is exceeded, the byte slice contains
    /// illegal characters or the content would result in an illegal name it fails.
    fn insert_bytes(&mut self, idx: usize, bytes: &[u8]) -> Result<(), SemanticStringError> {
        let msg = "Unable to insert byte string";
        if Self::does_contain_invalid_characters(bytes) {
            fail!(from self, with SemanticStringError::InvalidCharacter,
                "{} \"{}\" since it contains illegal characters.",
                msg, as_escaped_string(bytes) );
        }

        fail!(from self, when unsafe { self.get_mut_string().insert_bytes(idx, bytes) },
                with SemanticStringError::ExceedsMaximumLength,
                    "{} \"{}\" since it would exceed the maximum allowed length of {}.",
                        msg, as_escaped_string(bytes), CAPACITY);

        if Self::is_invalid_content(self.as_bytes()) {
            unsafe { self.get_mut_string().remove_range(idx, bytes.len()) };
            fail!(from self, with SemanticStringError::InvalidName,
                "{} \"{}\" since it would result in an illegal name.",
                msg, as_escaped_string(bytes));
        }

        Ok(())
    }

    /// Removes the last character. If the string is empty it returns [`None`].
    /// If the removal would create an illegal name it fails.
    fn pop(&mut self) -> Result<Option<u8>, SemanticStringError> {
        if self.len() == 0 {
            return Ok(None);
        }

        Ok(Some(self.remove(self.len() - 1)?))
    }

    /// Adds a single byte at the end. When the capacity is exceeded, the byte is an
    /// illegal character or the content would result in an illegal name it fails.
    fn push(&mut self, byte: u8) -> Result<(), SemanticStringError> {
        self.insert(self.len(), byte)
    }

    /// Adds a byte slice at the end. When the capacity is exceeded, the byte slice contains
    /// illegal characters or the content would result in an illegal name it fails.
    fn push_bytes(&mut self, bytes: &[u8]) -> Result<(), SemanticStringError> {
        self.insert_bytes(self.len(), bytes)
    }

    /// Removes a byte at a specific position and returns it.
    /// If the removal would create an illegal name it fails.
    fn remove(&mut self, idx: usize) -> Result<u8, SemanticStringError> {
        let value = unsafe { self.get_mut_string().remove(idx) };

        if Self::is_invalid_content(self.as_bytes()) {
            unsafe { self.get_mut_string().insert(idx, value).unwrap() };
            fail!(from self, with SemanticStringError::InvalidName,
                "Unable to remove character at position {} since it would result in an illegal name.",
                idx);
        }

        Ok(value)
    }

    /// Removes a range.
    /// If the removal would create an illegal name it fails.
    fn remove_range(&mut self, idx: usize, len: usize) -> Result<(), SemanticStringError> {
        let mut temp = *self.as_string();
        temp.remove_range(idx, len);
        if Self::is_invalid_content(temp.as_bytes()) {
            fail!(from self, with SemanticStringError::InvalidName,
                "Unable to remove range from {} with lenght {} since it would result in the illegal name \"{}\".",
                    idx, len, temp);
        }

        unsafe { self.get_mut_string().remove_range(idx, len) };
        Ok(())
    }

    /// Removes all bytes which satisfy the provided clojure f.
    /// If the removal would create an illegal name it fails.
    fn retain<F: FnMut(u8) -> bool>(&mut self, f: F) -> Result<(), SemanticStringError> {
        let mut temp = *self.as_string();
        let f = temp.retain_impl(f);

        if Self::is_invalid_content(temp.as_bytes()) {
            fail!(from self, with SemanticStringError::InvalidName,
                "Unable to retain characters from string since it would result in the illegal name \"{}\".",
                temp);
        }

        unsafe { self.get_mut_string().retain(f) };
        Ok(())
    }

    /// Removes a prefix. If the prefix does not exist it returns false. If the removal would lead
    /// to an invalid string content it fails and returns [`SemanticStringError::InvalidName`].
    /// After a successful removal it returns true.
    fn strip_prefix(&mut self, bytes: &[u8]) -> Result<bool, SemanticStringError> {
        let mut temp = *self.as_string();
        if !temp.strip_prefix(bytes) {
            return Ok(false);
        }

        if Self::is_invalid_content(temp.as_bytes()) {
            let mut prefix = FixedSizeByteString::<123>::new();
            unsafe { prefix.insert_bytes_unchecked(0, bytes) };
            fail!(from self, with SemanticStringError::InvalidName,
                "Unable to strip prefix \"{}\" from string since it would result in the illegal name \"{}\".",
                prefix, temp);
        }

        unsafe { self.get_mut_string().strip_prefix(bytes) };

        Ok(true)
    }

    /// Removes a suffix. If the suffix does not exist it returns false. If the removal would lead
    /// to an invalid string content it fails and returns [`SemanticStringError::InvalidName`].
    /// After a successful removal it returns true.
    fn strip_suffix(&mut self, bytes: &[u8]) -> Result<bool, SemanticStringError> {
        let mut temp = *self.as_string();
        if !temp.strip_suffix(bytes) {
            return Ok(false);
        }

        if Self::is_invalid_content(temp.as_bytes()) {
            let mut prefix = FixedSizeByteString::<123>::new();
            unsafe { prefix.insert_bytes_unchecked(0, bytes) };
            fail!(from self, with SemanticStringError::InvalidName,
                "Unable to strip prefix \"{}\" from string since it would result in the illegal name \"{}\".",
                prefix, temp);
        }

        unsafe { self.get_mut_string().strip_suffix(bytes) };

        Ok(true)
    }

    /// Truncates the string to new_len.
    fn truncate(&mut self, new_len: usize) -> Result<(), SemanticStringError> {
        let mut temp = *self.as_string();
        temp.truncate(new_len);

        if Self::is_invalid_content(temp.as_bytes()) {
            fail!(from self, with SemanticStringError::InvalidName,
                "Unable to truncate characters to {} since it would result in the illegal name \"{}\".",
                new_len, temp);
        }

        unsafe { self.get_mut_string().truncate(new_len) };
        Ok(())
    }
}

#[macro_export(local_inner_macros)]
macro_rules! semantic_string {
    {$(#[$documentation:meta])*
     name: $string_name:ident, capacity: $capacity:expr,
     invalid_content: $invalid_content:expr, invalid_characters: $invalid_characters:expr,
     comparision: $comparision:expr} => {
        $(#[$documentation])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub struct $string_name {
            value: iceoryx2_bb_container::byte_string::FixedSizeByteString<$capacity>
        }

        impl $string_name {
            pub const unsafe fn new_unchecked(bytes: &[u8]) -> Self {
                Self {
                    value: iceoryx2_bb_container::byte_string::FixedSizeByteString::new_unchecked(bytes),
                }
            }

            pub const fn max_len() -> usize {
                $capacity
            }

            pub unsafe fn insert_bytes_unchecked(&mut self, idx: usize, bytes: &[u8]) {
                self.value.insert_bytes_unchecked(idx, bytes);
            }
        }

        impl std::fmt::Display for $string_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                std::write!(f, "{}", self.value)
            }
        }

        impl PartialEq<&[u8]> for $string_name {
            fn eq(&self, other: &&[u8]) -> bool {
                $comparision(self.value.as_bytes(), *other)
            }
        }

        impl PartialEq<&[u8]> for &$string_name {
            fn eq(&self, other: &&[u8]) -> bool {
                $comparision(self.value.as_bytes(), *other)
            }
        }

        impl<const CAPACITY: usize> PartialEq<[u8; CAPACITY]> for $string_name {
            fn eq(&self, other: &[u8; CAPACITY]) -> bool {
                $comparision(self.value.as_bytes(), other)
            }
        }

        impl<const CAPACITY: usize> PartialEq<&[u8; CAPACITY]> for $string_name {
            fn eq(&self, other: &&[u8; CAPACITY]) -> bool {
                $comparision(self.value.as_bytes(), *other)
            }
        }

        impl std::ops::Deref for $string_name {
            type Target = [u8];

            fn deref(&self) -> &Self::Target {
                self.value.as_bytes()
            }
        }

        impl iceoryx2_bb_container::semantic_string::SemanticStringAccessor<$capacity> for $string_name {
            unsafe fn new_empty() -> Self {
                Self {
                    value: iceoryx2_bb_container::byte_string::FixedSizeByteString::new(),
                }
            }

            fn as_string(&self) -> &iceoryx2_bb_container::byte_string::FixedSizeByteString<$capacity> {
                &self.value
            }

            unsafe fn get_mut_string(&mut self) -> &mut iceoryx2_bb_container::byte_string::FixedSizeByteString<$capacity> {
                &mut self.value
            }

            fn is_invalid_content(string: &[u8]) -> bool {
                $invalid_content(string)
            }

            fn does_contain_invalid_characters(string: &[u8]) -> bool {
                $invalid_characters(string)
            }
        }

        impl iceoryx2_bb_container::semantic_string::SemanticString<$capacity> for $string_name {}
    };
}
