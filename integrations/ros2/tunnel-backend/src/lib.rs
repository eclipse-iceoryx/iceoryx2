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

//! Tunnel backend connecting native iceoryx2 applications with ROS 2 nodes,
//! implemented on top of [r2r](https://github.com/sequenceplanner/r2r).

// Stub phase: builder fields are stored but not yet read
#![allow(dead_code)]

pub mod backend;
pub mod discovery;
pub mod relays;

pub use backend::*;
