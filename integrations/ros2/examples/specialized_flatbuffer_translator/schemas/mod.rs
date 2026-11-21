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

#[path = "joint_readings_generated.rs"]
#[allow(clippy::all)]
#[rustfmt::skip]
mod joint_readings_generated;

#[path = "pose_generated.rs"]
#[allow(clippy::all)]
#[rustfmt::skip]
mod pose_generated;

pub use joint_readings_generated::robot::*;
pub use pose_generated::robot::*;

pub const JOINT_READINGS_SCHEMA: &[u8] = include_bytes!("joint_readings.bfbs");
pub const POSE_SCHEMA: &[u8] = include_bytes!("pose.bfbs");
