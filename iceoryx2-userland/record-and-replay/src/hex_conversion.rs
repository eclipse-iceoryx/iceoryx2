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

use iceoryx2_bb_log::debug;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Failures that can occur when calling [`hex_string_to_bytes()`].
pub enum HexToBytesConversionError {
    /// The string contained an invalid hex symbol
    InvalidHexCode,
}

impl core::fmt::Display for HexToBytesConversionError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "HexToBytesConversionError::{self:?}")
    }
}

impl core::error::Error for HexToBytesConversionError {}

/// Converts a given string into bytes ([`Vec<u8>`]). The provided string must be created
/// with [`bytes_to_hex_string()`] or at least follow strictly the semantics.
/// 1. Every byte is white space separated, eg `af fe de ad`
/// 2. A byte consists of two hex digits, eg `01` or `ab`
pub fn hex_string_to_bytes(hex_string: &str) -> Result<Vec<u8>, HexToBytesConversionError> {
    hex_string
        .split_ascii_whitespace()
        .map(|hex| {
            u8::from_str_radix(hex, 16).map_err(|e| {
                debug!(from "hex_string_to_raw_data()",
                        "Unable convert \"{hex}\" to hex-code ({e:?}).");
                HexToBytesConversionError::InvalidHexCode
            })
        })
        .collect::<Result<Vec<u8>, HexToBytesConversionError>>()
}

/// Converts bytes into a hex string. Can be converted back with [`hex_string_to_bytes()`].
pub fn bytes_to_hex_string(raw_data: &[u8]) -> String {
    use std::fmt::Write;

    let mut ret_val = String::with_capacity(3 * raw_data.len());
    for byte in raw_data {
        let _ = write!(&mut ret_val, "{byte:0>2x} ");
    }

    ret_val
}
