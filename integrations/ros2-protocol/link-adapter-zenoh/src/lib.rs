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

//! A link adapter for ROS 2 systems that run on `rmw_zenoh_cpp`.
//!
//! The [`RmwZenohAdapter`] speaks the protocol of `rmw_zenoh_cpp` directly
//! over a zenoh session, so a gateway places `iceoryx2` services into a
//! ROS 2 graph without `rcl` and without a ROS 2 installation.

mod adapter;
mod endpoint_description;

pub use adapter::{OpenError, RmwZenohAdapter};
pub use endpoint_description::{EndpointDescription, EndpointName, EndpointSettings, MessageType};
