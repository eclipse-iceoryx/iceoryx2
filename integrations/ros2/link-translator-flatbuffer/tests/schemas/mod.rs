// Copyright (c) 2026 Contributors to the Eclipse Foundation
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

//! The binary schemas and their Rust bindings Must be generated before running tests.

pub const MULTI_ARRAY_DIMENSION: &[u8] = include_bytes!("multi_array_dimension.bfbs");
pub const TIME: &[u8] = include_bytes!("time.bfbs");
pub const TIME_MISMATCHED: &[u8] = include_bytes!("time_mismatched.bfbs");
pub const TIME_RENAMED: &[u8] = include_bytes!("time_renamed.bfbs");
pub const UINT8_ENUM: &[u8] = include_bytes!("uint8_enum.bfbs");
pub const WSTRING: &[u8] = include_bytes!("wstring.bfbs");

#[path = "multi_array_dimension_generated.rs"]
#[allow(clippy::all)]
#[rustfmt::skip]
pub mod multi_array_dimension;

#[path = "time_generated.rs"]
#[allow(clippy::all)]
#[rustfmt::skip]
pub mod time;

#[path = "uint8_enum_generated.rs"]
#[allow(clippy::all)]
#[rustfmt::skip]
pub mod uint8_enum;
