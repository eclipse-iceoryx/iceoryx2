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

//! # iceoryx2-services-common
//!
//! Common types and utilities shared across the `iceoryx2-services` crates.

#![no_std]

use iceoryx2::{
    prelude::ZeroCopySend,
    service::{service_hash::ServiceHash, static_config::StaticConfig},
};

extern crate alloc;

/// Events emitted by the service discovery service.
#[derive(Debug, ZeroCopySend, serde::Serialize, serde::Deserialize)]
#[allow(dead_code)] // Fields used by subscribers
#[repr(C)]
pub enum DiscoveryEvent {
    /// A service has been added to the system.
    ///
    /// Contains the static configuration of the newly added service.
    Added(StaticConfig),

    /// A service has been removed from the system.
    ///
    /// Contains the hash identifying the removed service.
    Removed(ServiceHash),
}
