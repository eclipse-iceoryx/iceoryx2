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

//! Simplifies the kind of serialization which shall be used. The implementation has two methods
//!  * [`Serialize::serialize()`] - serialize a given object
//!  * [`Serialize::deserialize()`] - deserialize a given byte reference into the source object
//!
//! # Example
//!
//! ```
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
//! struct TestStruct {
//!     value: u64
//! };
//!
//! fn example<S: iceoryx2_cal::serialize::Serialize>() {
//!     let data_orig = TestStruct { value: 1234 };
//!
//!     let serialized = S::serialize::<TestStruct>(&data_orig)
//!                                 .expect("serialization failed.");
//!
//!     let data: TestStruct = S::deserialize(serialized.as_slice())
//!                           .expect("deserialization failed.");
//!
//!     assert_eq!(data, data_orig);
//! }
//! ```

pub mod postcard;
pub mod recommended;

#[cfg(feature = "std")]
pub mod toml;

use core::fmt::Debug;

use alloc::vec::Vec;

/// Failure emitted by [`Serialize::serialize()`]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SerializeError {
    InternalError,
}

impl core::fmt::Display for SerializeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "SerializeError::{self:?}")
    }
}

impl core::error::Error for SerializeError {}

/// Failure emitted by [`Serialize::deserialize()`]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum DeserializeError {
    InternalError,
}

impl core::fmt::Display for DeserializeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "DeserializeError::{self:?}")
    }
}

impl core::error::Error for DeserializeError {}

/// Serialize and deserialize constructs which implement [`serde::Serialize`] and
/// [`serde::de::DeserializeOwned`]
pub trait Serialize: Debug {
    /// Serializes a value
    fn serialize<T: serde::Serialize>(value: &T) -> Result<Vec<u8>, SerializeError>;

    /// Deserialize a value from a given byte slice
    fn deserialize<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T, DeserializeError>;
}
