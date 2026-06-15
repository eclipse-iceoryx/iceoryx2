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
//! implemented on the [r2r_rcl](https://docs.rs/r2r_rcl) bindings to `rcl`.

// Work in progress. Not all APIs are implemented.
#![allow(dead_code)]
#![deny(unsafe_code)]

pub mod backend;
pub mod discovery;
pub(crate) mod keys;
pub mod relays;

#[allow(unsafe_code)]
pub(crate) mod payload;
#[allow(unsafe_code)]
pub(crate) mod rcl;
#[allow(unsafe_code)]
pub(crate) mod typesupport;

pub use backend::*;
