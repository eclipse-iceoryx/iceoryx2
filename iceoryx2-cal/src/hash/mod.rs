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
//!     println!("Hash as hex: {}", hash.as_hex_string());
//! }
//! ```

pub mod sha1;

/// Interface to generate hashes.
pub trait Hash {
    /// Creates a new hash from `bytes`.
    fn new(bytes: &[u8]) -> Self;

    /// Converts the hash into as string of hex-characters
    fn as_hex_string(&self) -> String;
}
