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

//! The [`SemanticString`](crate::semantic_string::SemanticString) is a trait for
//! [`FixedSizeByteString`](crate::byte_string::FixedSizeByteString) to create
//! strong string types with semantic content contracts. They can be created
//! with the help of the [`semantic_string`](crate::semantic_string!) macro.
//!
//! # Example, create a string that can contain a posix group name
//!
//! ```
//! extern crate alloc;
//!
//! pub use iceoryx2_bb_container::semantic_string::SemanticString;
//! use iceoryx2_bb_derive_macros::ZeroCopySend;
//! use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
//!
//! use core::hash::{Hash, Hasher};
//! use iceoryx2_bb_container::semantic_string;
//!
//! const GROUP_NAME_LENGTH: usize = 31;
//! semantic_string! {
//!   // Name of the type
//!   name: GroupName,
//!   // The underlying capacity of the FixedSizeByteString
//!   capacity: GROUP_NAME_LENGTH,
//!   // Callable that shall return true when the provided string contains invalid content
//!   invalid_content: |string: &[u8]| {
//!     if string.is_empty() {
//!         // group names are not allowed to be empty
//!         return true;
//!     }
//!
//!     // group names are not allowed to start with a number or -
//!     matches!(string[0], b'-' | b'0'..=b'9')
//!   },
//!   // Callable that shall return true when the provided string contains invalid characters
//!   invalid_characters: |string: &[u8]| {
//!     for value in string {
//!         match value {
//!             // only non-capital letters, numbers and - is allowed
//!             b'a'..=b'z' | b'0'..=b'9' | b'-' => (),
//!             _ => return true,
//!         }
//!     }
//!
//!     false
//!   },
//!   // When a SemanticString has multiple representations of the same semantic content, this
//!   // callable shall convert the content to a uniform representation.
//!   // Example: The path to `/tmp` can be also expressed as `/tmp/` or `////tmp////`
//!   normalize: |this: &GroupName| {
//!       this.clone()
//!   }
//! }
//! ```

use crate::byte_string::FixedSizeByteStringModificationError;
use crate::byte_string::{as_escaped_string, strnlen, FixedSizeByteString};
use core::fmt::{Debug, Display};
use core::hash::Hash;
use core::ops::Deref;
use iceoryx2_bb_log::fail;

/// Failures that can occur when a [`SemanticString`] is created or modified
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SemanticStringError {
    /// The modification would lead to a [`SemanticString`] with invalid content.
    InvalidContent,
    /// The added content would exceed the maximum capacity of the [`SemanticString`]
    ExceedsMaximumLength,
}

impl From<FixedSizeByteStringModificationError> for SemanticStringError {
    fn from(_value: FixedSizeByteStringModificationError) -> Self {
        SemanticStringError::ExceedsMaximumLength
    }
}

impl core::fmt::Display for SemanticStringError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::write!(f, "SemanticStringError::{self:?}")
    }
}

impl core::error::Error for SemanticStringError {}

#[doc(hidden)]
pub mod internal {
    use super::*;

    pub trait SemanticStringAccessor<const CAPACITY: usize> {
        unsafe fn new_empty() -> Self;
        unsafe fn get_mut_string(&mut self) -> &mut FixedSizeByteString<CAPACITY>;
        fn is_invalid_content(string: &[u8]) -> bool;
        fn does_contain_invalid_characters(string: &[u8]) -> bool;
    }
}

/// Trait that defines the methods a [`FixedSizeByteString`] with context semantics, a
/// [`SemanticString`] shares. A new [`SemanticString`] can be created with the [`crate::semantic_string!`]
/// macro. For the usage, see [`mod@crate::semantic_string`].
pub trait SemanticString<const CAPACITY: usize>:
    internal::SemanticStringAccessor<CAPACITY>
    + Debug
    + Display
    + Sized
    + Deref<Target = [u8]>
    + PartialEq
    + Eq
    + Hash
{
    /// Returns a reference to the underlying [`FixedSizeByteString`]
    fn as_string(&self) -> &FixedSizeByteString<CAPACITY>;

    /// Creates a new content. If it contains invalid characters or exceeds the maximum supported
    /// length of the system or contains illegal strings it fails.
    fn new(value: &[u8]) -> Result<Self, SemanticStringError> {
        let msg = "Unable to create SemanticString";
        let origin = "SemanticString::new()";

        let mut new_self =
            unsafe { <Self as internal::SemanticStringAccessor<CAPACITY>>::new_empty() };
        fail!(from origin, when new_self.push_bytes(value),
            "{} due to an invalid value \"{}\".", msg, as_escaped_string(value));

        Ok(new_self)
    }

    /// Creates a new content but does not verify that it does not contain invalid characters.
    ///
    /// # Safety
    ///
    ///   * The slice must contain only valid characters.
    ///   * The slice must have a length that is less or equal CAPACITY
    ///   * The slice must not contain invalid UTF-8 characters
    ///
    unsafe fn new_unchecked(bytes: &[u8]) -> Self;

    /// Creates a new content from a given ptr. The user has to ensure that it is null-terminated.
    ///
    /// # Safety
    ///
    ///   * The pointer must be '\0' (null) terminated
    ///   * The pointer must be valid and non-null
    ///   * The contents must have a length that is less or equal CAPACITY
    ///   * The contents must not contain invalid UTF-8 characters
    ///
    unsafe fn from_c_str(ptr: *const core::ffi::c_char) -> Result<Self, SemanticStringError> {
        Self::new(core::slice::from_raw_parts(
            ptr.cast(),
            strnlen(ptr, CAPACITY + 1),
        ))
    }

    /// Returns the contents as a slice
    fn as_bytes(&self) -> &[u8] {
        self.as_string().as_bytes()
    }

    /// Returns a zero terminated slice of the underlying bytes
    fn as_c_str(&self) -> *const core::ffi::c_char {
        self.as_string().as_c_str()
    }

    /// Returns the capacity of the file system type
    fn capacity(&self) -> usize {
        CAPACITY
    }

    /// Finds the first occurrence of a  byte string in the given string. If the byte string was
    /// found the start position of the byte string is returned, otherwise [`None`].
    fn find(&self, bytes: &[u8]) -> Option<usize> {
        self.as_string().find(bytes)
    }

    /// Finds the last occurrence of a byte string in the given string. If the byte string was
    /// found the start position of the byte string is returned, otherwise [`None`].
    fn rfind(&self, bytes: &[u8]) -> Option<usize> {
        self.as_string().find(bytes)
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
    /// illegal character or the content would result in an illegal content it fails.
    fn insert(&mut self, idx: usize, byte: u8) -> Result<(), SemanticStringError> {
        self.insert_bytes(idx, &[byte; 1])
    }

    /// Inserts a byte slice at a specific position. When the capacity is exceeded, the byte slice contains
    /// illegal characters or the content would result in an illegal content it fails.
    fn insert_bytes(&mut self, idx: usize, bytes: &[u8]) -> Result<(), SemanticStringError> {
        let msg = "Unable to insert byte string";
        fail!(from self, when unsafe { self.get_mut_string().insert_bytes(idx, bytes) },
                with SemanticStringError::ExceedsMaximumLength,
                    "{} \"{}\" since it would exceed the maximum allowed length of {}.",
                        msg, as_escaped_string(bytes), CAPACITY);

        if Self::is_invalid_content(self.as_bytes()) {
            unsafe { self.get_mut_string().remove_range(idx, bytes.len()) };
            fail!(from self, with SemanticStringError::InvalidContent,
                "{} \"{}\" since it would result in an illegal content.",
                msg, as_escaped_string(bytes));
        }

        Ok(())
    }

    /// Adds bytes to the string without checking if they only contain valid characters or
    /// would result in a valid result.
    ///
    /// # Safety
    ///
    ///   * The user must ensure that the bytes contain only valid characters.
    ///   * The user must ensure that the result, after the bytes were added, is valid.
    ///   * The slice must have a length that is less or equal CAPACITY
    ///   * The slice is not contain invalid UTF-8 characters
    ///
    unsafe fn insert_bytes_unchecked(&mut self, idx: usize, bytes: &[u8]);

    /// Normalizes the string. This function is used as basis for [`core::hash::Hash`] and
    /// [`PartialEq`]. Normalizing a [`SemanticString`] means to bring it to some format so that it
    /// contains still the same semantic content but in an uniform way so that strings, with the
    /// same semantic content but different representation compare as equal.
    fn normalize(&self) -> Self;

    /// Removes the last character. If the string is empty it returns [`None`].
    /// If the removal would create an illegal content it fails.
    fn pop(&mut self) -> Result<Option<u8>, SemanticStringError> {
        if self.len() == 0 {
            return Ok(None);
        }

        Ok(Some(self.remove(self.len() - 1)?))
    }

    /// Adds a single byte at the end. When the capacity is exceeded, the byte is an
    /// illegal character or the content would result in an illegal content it fails.
    fn push(&mut self, byte: u8) -> Result<(), SemanticStringError> {
        self.insert(self.len(), byte)
    }

    /// Adds a byte slice at the end. When the capacity is exceeded, the byte slice contains
    /// illegal characters or the content would result in an illegal content it fails.
    fn push_bytes(&mut self, bytes: &[u8]) -> Result<(), SemanticStringError> {
        self.insert_bytes(self.len(), bytes)
    }

    /// Removes a byte at a specific position and returns it.
    /// If the removal would create an illegal content it fails.
    fn remove(&mut self, idx: usize) -> Result<u8, SemanticStringError> {
        let value = unsafe { self.get_mut_string().remove(idx) };

        if Self::is_invalid_content(self.as_bytes()) {
            unsafe { self.get_mut_string().insert(idx, value).unwrap() };
            fail!(from self, with SemanticStringError::InvalidContent,
                "Unable to remove character at position {} since it would result in an illegal content.",
                idx);
        }

        Ok(value)
    }

    /// Removes a range.
    /// If the removal would create an illegal content it fails.
    fn remove_range(&mut self, idx: usize, len: usize) -> Result<(), SemanticStringError> {
        let mut temp = *self.as_string();
        temp.remove_range(idx, len);
        if Self::is_invalid_content(temp.as_bytes()) {
            fail!(from self, with SemanticStringError::InvalidContent,
                "Unable to remove range from {} with lenght {} since it would result in the illegal content \"{}\".",
                    idx, len, temp);
        }

        unsafe { self.get_mut_string().remove_range(idx, len) };
        Ok(())
    }

    /// Removes all bytes which satisfy the provided clojure f.
    /// If the removal would create an illegal content it fails.
    fn retain<F: FnMut(u8) -> bool>(&mut self, f: F) -> Result<(), SemanticStringError> {
        let mut temp = *self.as_string();
        let f = temp.retain_impl(f);

        if Self::is_invalid_content(temp.as_bytes()) {
            fail!(from self, with SemanticStringError::InvalidContent,
                "Unable to retain characters from string since it would result in the illegal content \"{}\".",
                temp);
        }

        unsafe { self.get_mut_string().retain(f) };
        Ok(())
    }

    /// Removes a prefix. If the prefix does not exist it returns false. If the removal would lead
    /// to an invalid string content it fails and returns [`SemanticStringError::InvalidContent`].
    /// After a successful removal it returns true.
    fn strip_prefix(&mut self, bytes: &[u8]) -> Result<bool, SemanticStringError> {
        let mut temp = *self.as_string();
        if !temp.strip_prefix(bytes) {
            return Ok(false);
        }

        if Self::is_invalid_content(temp.as_bytes()) {
            let mut prefix = FixedSizeByteString::<123>::new();
            unsafe { prefix.insert_bytes_unchecked(0, bytes) };
            fail!(from self, with SemanticStringError::InvalidContent,
                "Unable to strip prefix \"{}\" from string since it would result in the illegal content \"{}\".",
                prefix, temp);
        }

        unsafe { self.get_mut_string().strip_prefix(bytes) };

        Ok(true)
    }

    /// Removes a suffix. If the suffix does not exist it returns false. If the removal would lead
    /// to an invalid string content it fails and returns [`SemanticStringError::InvalidContent`].
    /// After a successful removal it returns true.
    fn strip_suffix(&mut self, bytes: &[u8]) -> Result<bool, SemanticStringError> {
        let mut temp = *self.as_string();
        if !temp.strip_suffix(bytes) {
            return Ok(false);
        }

        if Self::is_invalid_content(temp.as_bytes()) {
            let mut prefix = FixedSizeByteString::<123>::new();
            unsafe { prefix.insert_bytes_unchecked(0, bytes) };
            fail!(from self, with SemanticStringError::InvalidContent,
                "Unable to strip prefix \"{}\" from string since it would result in the illegal content \"{}\".",
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
            fail!(from self, with SemanticStringError::InvalidContent,
                "Unable to truncate characters to {} since it would result in the illegal content \"{}\".",
                new_len, temp);
        }

        unsafe { self.get_mut_string().truncate(new_len) };
        Ok(())
    }
}

/// Helper macro to create a new [`SemanticString`]. Usage example can be found here:
/// [`mod@crate::semantic_string`].
#[macro_export(local_inner_macros)]
macro_rules! semantic_string {
    {$(#[$documentation:meta])*
     /// Name of the struct
     name: $string_name:ident,
     /// Capacity of the underlying FixedSizeByteString
     capacity: $capacity:expr,
     /// Callable that gets a [`&[u8]`] as input and shall return true when the slice contains
     /// invalid content.
     invalid_content: $invalid_content:expr,
     /// Callable that gets a [`&[u8]`] as input and shall return true when the slice contains
     /// invalid characters.
     invalid_characters: $invalid_characters:expr,
     /// Normalizes the content. Required when the same semantical content has multiple
     /// representations like paths for instance (`/tmp` == `/tmp/`)
     normalize: $normalize:expr} => {
        $(#[$documentation])*
        #[repr(C)]
        #[derive(Debug, Clone, Eq, PartialOrd, Ord, ZeroCopySend)]
        pub struct $string_name {
            value: iceoryx2_bb_container::byte_string::FixedSizeByteString<$capacity>
        }

        // BEGIN: serde
        pub(crate) mod VisitorType {
            pub(crate) struct $string_name;
        }

        impl<'de> serde::de::Visitor<'de> for VisitorType::$string_name {
            type Value = $string_name;

            fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
                formatter.write_str("a string containing the service name")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match $string_name::new(v.as_bytes()) {
                    Ok(v) => Ok(v),
                    Err(v) => Err(E::custom(alloc::format!("invalid {} provided {:?}.", core::stringify!($string_name), v))),
                }
            }
        }

        impl<'de> serde::Deserialize<'de> for $string_name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                deserializer.deserialize_str(VisitorType::$string_name)
            }
        }

        impl serde::Serialize for $string_name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                serializer.serialize_str(core::str::from_utf8(self.as_bytes()).unwrap())
            }
        }
        // END: serde

        impl iceoryx2_bb_container::semantic_string::SemanticString<$capacity> for $string_name {
            fn as_string(&self) -> &iceoryx2_bb_container::byte_string::FixedSizeByteString<$capacity> {
                &self.value
            }

            fn normalize(&self) -> Self {
                $normalize(self)
            }

            unsafe fn new_unchecked(bytes: &[u8]) -> Self {
                Self {
                    value: iceoryx2_bb_container::byte_string::FixedSizeByteString::new_unchecked(bytes),
                }
            }

            unsafe fn insert_bytes_unchecked(&mut self, idx: usize, bytes: &[u8]) {
                self.value.insert_bytes_unchecked(idx, bytes);
            }
        }

        impl $string_name {
            /// Returns the maximum length of [`$string`]
            pub const fn max_len() -> usize {
                $capacity
            }
        }

        impl core::fmt::Display for $string_name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                core::write!(f, "{}", self.value)
            }
        }

        impl Hash for $string_name {
            fn hash<H: Hasher>(&self, state: &mut H) {
                self.normalize().as_bytes().hash(state)
            }
        }

        impl From<$string_name> for String {
            fn from(value: $string_name) -> String {
                // SAFETY: It is ensured that the semantic string contains only valid utf-8 strings
                unsafe { String::from_utf8_unchecked(value.as_bytes().to_vec()) }
            }
        }

        impl From<&$string_name> for String {
            fn from(value: &$string_name) -> String {
                // SAFETY: It is ensured that the semantic string contains only valid utf-8 strings
                unsafe { String::from_utf8_unchecked(value.as_bytes().to_vec()) }
            }
        }

        impl core::convert::TryFrom<&str> for $string_name {
            type Error = iceoryx2_bb_container::semantic_string::SemanticStringError;

            fn try_from(value: &str) -> Result<Self, Self::Error> {
                Self::new(value.as_bytes())
            }
        }

        impl PartialEq<$string_name> for $string_name {
            fn eq(&self, other: &$string_name) -> bool {
                *self.normalize().as_bytes() == *other.normalize().as_bytes()
            }
        }

        impl PartialEq<&[u8]> for $string_name {
            fn eq(&self, other: &&[u8]) -> bool {
                let other = match $string_name::new(other) {
                    Ok(other) => other,
                    Err(_) => return false,
                };

                *self == other
            }
        }

        impl PartialEq<&[u8]> for &$string_name {
            fn eq(&self, other: &&[u8]) -> bool {
                let other = match $string_name::new(other) {
                    Ok(other) => other,
                    Err(_) => return false,
                };

                **self == other
            }
        }

        impl<const CAPACITY: usize> PartialEq<[u8; CAPACITY]> for $string_name {
            fn eq(&self, other: &[u8; CAPACITY]) -> bool {
                let other = match $string_name::new(other) {
                    Ok(other) => other,
                    Err(_) => return false,
                };

                *self == other
            }
        }

        impl<const CAPACITY: usize> PartialEq<&[u8; CAPACITY]> for $string_name {
            fn eq(&self, other: &&[u8; CAPACITY]) -> bool {
                let other = match $string_name::new(*other) {
                    Ok(other) => other,
                    Err(_) => return false,
                };

                *self == other
            }
        }

        impl PartialEq<&str> for &$string_name {
            fn eq(&self, other: &&str) -> bool {
                let other = match $string_name::new(other.as_bytes()) {
                    Ok(other) => other,
                    Err(_) => return false,
                };

                **self == other
            }
        }

        impl core::ops::Deref for $string_name {
            type Target = [u8];

            fn deref(&self) -> &Self::Target {
                self.value.as_bytes()
            }
        }

        impl iceoryx2_bb_container::semantic_string::internal::SemanticStringAccessor<$capacity> for $string_name {
            unsafe fn new_empty() -> Self {
                Self {
                    value: iceoryx2_bb_container::byte_string::FixedSizeByteString::new(),
                }
            }

            unsafe fn get_mut_string(&mut self) -> &mut iceoryx2_bb_container::byte_string::FixedSizeByteString<$capacity> {
                &mut self.value
            }

            fn is_invalid_content(string: &[u8]) -> bool {
                if Self::does_contain_invalid_characters(string) {
                    return true;
                }

                $invalid_content(string)
            }

            fn does_contain_invalid_characters(string: &[u8]) -> bool {
                if core::str::from_utf8(string).is_err() {
                    return true;
                }

                $invalid_characters(string)
            }
        }

    };
}
