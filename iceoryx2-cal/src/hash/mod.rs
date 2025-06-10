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

//! Creates hashes from arbitrary byte slices.
//!
//! # Example
//!
//! ```
//! use iceoryx2_cal::hash::*;
//!
//! fn create_hash<H: Hash>() {
//!     let some_text = "Hello World".to_string();
//!     let hash = H::new(some_text.as_bytes());
//!
//!     println!("Hash value: {:?}", hash.value());
//! }
//! ```

use iceoryx2_bb_container::semantic_string::{SemanticString, SemanticStringError};
use iceoryx2_bb_system_types::base64url::Base64Url;

pub mod recommended;
pub mod sha1;

/// Represents the value of the hash.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HashValue {
    value: Base64Url,
}

impl HashValue {
    pub(crate) fn new(bytes: &[u8]) -> Result<HashValue, SemanticStringError> {
        Ok(Self {
            value: Base64Url::new(bytes)?,
        })
    }

    /// Returns the base64url representation of the [`HashValue`]
    pub fn as_base64url(&self) -> &Base64Url {
        &self.value
    }
}

impl From<HashValue> for String {
    fn from(value: HashValue) -> Self {
        value.as_base64url().into()
    }
}

impl From<&HashValue> for String {
    fn from(value: &HashValue) -> Self {
        value.as_base64url().into()
    }
}

/// Interface to generate hashes.
pub trait Hash {
    /// Creates a new hash from `bytes`.
    fn new(bytes: &[u8]) -> Self;

    /// Returns the value of the [`Hash`]
    fn value(&self) -> HashValue;
}
