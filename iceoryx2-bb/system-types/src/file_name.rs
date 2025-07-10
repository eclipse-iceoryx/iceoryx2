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

//! Relocatable (inter-process shared memory compatible) [`semantic_string::SemanticString`] implementation for
//! [`FileName`]. All modification operations ensure that never an
//! invalid file or path name can be generated. All strings have a fixed size so that the maximum
//! path or file name length the system supports can be stored.
//!
//! # Example
//!
//! ```
//! use iceoryx2_bb_container::semantic_string::SemanticString;
//! use iceoryx2_bb_system_types::file_name::*;
//!
//! let name = FileName::new(b"some_file.txt");
//!
//! let invalid_name = FileName::new(b"no/path/allowed.txt");
//! assert!(invalid_name.is_err());
//! ```

pub use iceoryx2_bb_container::semantic_string::SemanticString;
use iceoryx2_bb_derive_macros::ZeroCopySend;
use iceoryx2_bb_elementary::static_assert::{static_assert_ge, static_assert_le};
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;

use core::hash::{Hash, Hasher};
use iceoryx2_bb_container::semantic_string;
use iceoryx2_pal_configuration::FILENAME_LENGTH;

fn invalid_characters(value: &[u8]) -> bool {
    for c in value {
        match c {
            // linux & windows
            0 => return true,
            b'/' => return true,
            // windows only
            1..=31 => return true,
            b':' => return true,
            b'\\' => return true,
            b'<' => return true,
            b'>' => return true,
            b'"' => return true,
            b'|' => return true,
            b'?' => return true,
            b'*' => return true,
            _ => (),
        }
    }
    false
}

fn invalid_content(value: &[u8]) -> bool {
    matches!(value, b"" | b"." | b"..")
}

fn normalize(this: &FileName) -> FileName {
    this.clone()
}

semantic_string! {
  /// Represents a file name. The restriction are choosen in a way that it is platform independent.
  /// This means characters/strings which would be legal on some platforms are forbidden as well.
  name: FileName,
  capacity: FILENAME_LENGTH,
  invalid_content: invalid_content,
  invalid_characters: invalid_characters,
  normalize: normalize
}

#[derive(Debug, Clone, Eq, ZeroCopySend)]
#[repr(C)]
pub struct RestrictedFileName<const CAPACITY: usize> {
    value: iceoryx2_bb_container::byte_string::FixedSizeByteString<CAPACITY>,
}

impl<const CAPACITY: usize>
    iceoryx2_bb_container::semantic_string::internal::SemanticStringAccessor<CAPACITY>
    for RestrictedFileName<CAPACITY>
{
    unsafe fn new_empty() -> Self {
        static_assert_le::<{ CAPACITY }, { FILENAME_LENGTH }>();
        static_assert_ge::<{ CAPACITY }, 1>();

        Self {
            value: iceoryx2_bb_container::byte_string::FixedSizeByteString::new(),
        }
    }

    unsafe fn get_mut_string(
        &mut self,
    ) -> &mut iceoryx2_bb_container::byte_string::FixedSizeByteString<CAPACITY> {
        &mut self.value
    }

    fn is_invalid_content(string: &[u8]) -> bool {
        if Self::does_contain_invalid_characters(string) {
            return true;
        }

        invalid_content(string)
    }

    fn does_contain_invalid_characters(string: &[u8]) -> bool {
        if core::str::from_utf8(string).is_err() {
            return true;
        }

        invalid_characters(string)
    }
}

// BEGIN: serde
pub(crate) mod visitor_type {
    pub(crate) struct RestrictedFileName<const CAPACITY: usize>;
}

impl<const CAPACITY: usize> serde::de::Visitor<'_> for visitor_type::RestrictedFileName<CAPACITY> {
    type Value = RestrictedFileName<CAPACITY>;

    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        formatter.write_str("a string containing the service name")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        RestrictedFileName::<CAPACITY>::new(v.as_bytes()).map_err(|e| {
            E::custom(alloc::format!(
                "invalid RestrictedFileName<{CAPACITY}> provided {v:?} ({e:?})."
            ))
        })
    }
}

impl<'de, const CAPACITY: usize> serde::Deserialize<'de> for RestrictedFileName<CAPACITY> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(visitor_type::RestrictedFileName::<CAPACITY>)
    }
}

impl<const CAPACITY: usize> serde::Serialize for RestrictedFileName<CAPACITY> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(core::str::from_utf8(self.as_bytes()).unwrap())
    }
}
// END: serde

impl<const CAPACITY: usize> iceoryx2_bb_container::semantic_string::SemanticString<CAPACITY>
    for RestrictedFileName<CAPACITY>
{
    fn as_string(&self) -> &iceoryx2_bb_container::byte_string::FixedSizeByteString<CAPACITY> {
        &self.value
    }

    fn normalize(&self) -> Self {
        self.clone()
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

impl<const CAPACITY: usize> RestrictedFileName<CAPACITY> {
    /// Returns the maximum length of [`$string`]
    pub const fn max_len() -> usize {
        CAPACITY
    }
}

impl<const CAPACITY: usize> core::fmt::Display for RestrictedFileName<CAPACITY> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl<const CAPACITY: usize> Hash for RestrictedFileName<CAPACITY> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.normalize().as_bytes().hash(state)
    }
}

impl<const CAPACITY: usize> From<RestrictedFileName<CAPACITY>> for FileName {
    fn from(value: RestrictedFileName<CAPACITY>) -> FileName {
        // SAFETY: It is ensured that the RestrictedFileName contains always a valid FileName, just
        // with less capacity
        unsafe { FileName::new_unchecked(value.as_bytes()) }
    }
}

impl<const CAPACITY: usize> From<RestrictedFileName<CAPACITY>> for String {
    fn from(value: RestrictedFileName<CAPACITY>) -> String {
        // SAFETY: It is ensured that the semantic string contains only valid utf-8 strings
        unsafe { String::from_utf8_unchecked(value.as_bytes().to_vec()) }
    }
}

impl<const CAPACITY: usize> From<&RestrictedFileName<CAPACITY>> for String {
    fn from(value: &RestrictedFileName<CAPACITY>) -> String {
        // SAFETY: It is ensured that the semantic string contains only valid utf-8 strings
        unsafe { String::from_utf8_unchecked(value.as_bytes().to_vec()) }
    }
}

impl<const CAPACITY: usize> core::convert::TryFrom<&str> for RestrictedFileName<CAPACITY> {
    type Error = iceoryx2_bb_container::semantic_string::SemanticStringError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value.as_bytes())
    }
}

impl<const CAPACITY: usize> core::convert::TryFrom<&FileName> for RestrictedFileName<CAPACITY> {
    type Error = iceoryx2_bb_container::semantic_string::SemanticStringError;

    fn try_from(value: &FileName) -> Result<Self, Self::Error> {
        Self::new(value.as_bytes())
    }
}

impl<const CAPACITY: usize> PartialEq<RestrictedFileName<CAPACITY>>
    for RestrictedFileName<CAPACITY>
{
    fn eq(&self, other: &RestrictedFileName<CAPACITY>) -> bool {
        *self.normalize().as_bytes() == *other.normalize().as_bytes()
    }
}

impl<const CAPACITY: usize> PartialEq<&[u8]> for RestrictedFileName<CAPACITY> {
    fn eq(&self, other: &&[u8]) -> bool {
        let other = match RestrictedFileName::<CAPACITY>::new(other) {
            Ok(other) => other,
            Err(_) => return false,
        };

        *self == other
    }
}

impl<const CAPACITY: usize> PartialEq<&[u8]> for &RestrictedFileName<CAPACITY> {
    fn eq(&self, other: &&[u8]) -> bool {
        let other = match RestrictedFileName::<CAPACITY>::new(other) {
            Ok(other) => other,
            Err(_) => return false,
        };

        **self == other
    }
}

impl<const CAPACITY: usize> PartialEq<[u8; CAPACITY]> for RestrictedFileName<CAPACITY> {
    fn eq(&self, other: &[u8; CAPACITY]) -> bool {
        let other = match RestrictedFileName::<CAPACITY>::new(other) {
            Ok(other) => other,
            Err(_) => return false,
        };

        *self == other
    }
}

impl<const CAPACITY: usize> PartialEq<&[u8; CAPACITY]> for RestrictedFileName<CAPACITY> {
    fn eq(&self, other: &&[u8; CAPACITY]) -> bool {
        // TODO: false positive from clippy
        #[allow(clippy::explicit_auto_deref)]
        let other = match RestrictedFileName::<CAPACITY>::new(*other) {
            Ok(other) => other,
            Err(_) => return false,
        };

        *self == other
    }
}

impl<const CAPACITY: usize> core::ops::Deref for RestrictedFileName<CAPACITY> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.value.as_bytes()
    }
}
