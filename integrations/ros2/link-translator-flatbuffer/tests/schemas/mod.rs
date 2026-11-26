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

pub const CHANNEL_FLOAT32: &[u8] = include_bytes!("channel_float32.bfbs");
pub const CHANNEL_FLOAT32_MISMATCHED: &[u8] = include_bytes!("channel_float32_mismatched.bfbs");
pub const HEADER: &[u8] = include_bytes!("header.bfbs");
pub const HEADER_MISMATCHED: &[u8] = include_bytes!("header_mismatched.bfbs");
pub const HEADER_STRUCT: &[u8] = include_bytes!("header_struct.bfbs");
pub const JOINT_STATE: &[u8] = include_bytes!("joint_state.bfbs");
pub const MULTI_ARRAY_DIMENSION: &[u8] = include_bytes!("multi_array_dimension.bfbs");
pub const POINT_CLOUD: &[u8] = include_bytes!("point_cloud.bfbs");
pub const POSE_ARRAY: &[u8] = include_bytes!("pose_array.bfbs");
pub const POSE_ARRAY_STRUCT: &[u8] = include_bytes!("pose_array_struct.bfbs");
pub const POSE_STAMPED: &[u8] = include_bytes!("pose_stamped.bfbs");
pub const POSE_WITH_COVARIANCE: &[u8] = include_bytes!("pose_with_covariance.bfbs");
pub const POSE_WITH_COVARIANCE_STAMPED: &[u8] = include_bytes!("pose_with_covariance_stamped.bfbs");
pub const TIME: &[u8] = include_bytes!("time.bfbs");
pub const TIME_MISMATCHED: &[u8] = include_bytes!("time_mismatched.bfbs");
pub const TIME_RENAMED: &[u8] = include_bytes!("time_renamed.bfbs");
pub const UINT8_ENUM: &[u8] = include_bytes!("uint8_enum.bfbs");
pub const WSTRING: &[u8] = include_bytes!("wstring.bfbs");

#[path = "channel_float32_generated.rs"]
#[allow(clippy::all)]
#[rustfmt::skip]
pub mod channel_float32;

#[path = "header_generated.rs"]
#[allow(clippy::all)]
#[rustfmt::skip]
pub mod header;

#[path = "header_struct_generated.rs"]
#[allow(clippy::all)]
#[rustfmt::skip]
pub mod header_struct;

#[path = "joint_state_generated.rs"]
#[allow(clippy::all)]
#[rustfmt::skip]
pub mod joint_state;

#[path = "multi_array_dimension_generated.rs"]
#[allow(clippy::all)]
#[rustfmt::skip]
pub mod multi_array_dimension;

#[path = "point_cloud_generated.rs"]
#[allow(clippy::all)]
#[rustfmt::skip]
pub mod point_cloud;

#[path = "pose_array_generated.rs"]
#[allow(clippy::all)]
#[rustfmt::skip]
pub mod pose_array;

#[path = "pose_array_struct_generated.rs"]
#[allow(clippy::all)]
#[rustfmt::skip]
pub mod pose_array_struct;

#[path = "pose_stamped_generated.rs"]
#[allow(clippy::all)]
#[rustfmt::skip]
pub mod pose_stamped;

#[path = "pose_with_covariance_generated.rs"]
#[allow(clippy::all)]
#[rustfmt::skip]
pub mod pose_with_covariance;

#[path = "pose_with_covariance_stamped_generated.rs"]
#[allow(clippy::all)]
#[rustfmt::skip]
pub mod pose_with_covariance_stamped;

#[path = "time_generated.rs"]
#[allow(clippy::all)]
#[rustfmt::skip]
pub mod time;

#[path = "uint8_enum_generated.rs"]
#[allow(clippy::all)]
#[rustfmt::skip]
pub mod uint8_enum;
