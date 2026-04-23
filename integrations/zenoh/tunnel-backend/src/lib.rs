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

//! # iceoryx2-integrations-zenoh-tunnel-backend
//!
//! A [Zenoh](https://zenoh.io)-based backend for the iceoryx2 tunnel service.
//!
//! This crate implements the tunnel
//! [`Backend`](iceoryx2_services_tunnel_backend::traits::Backend) trait,
//! providing a ready-to-use transport layer that forwards iceoryx2 communication
//! over the Zenoh protocol. Zenoh handles peer discovery, session management, and
//! network routing, so the tunnel can operate in a variety of network topologies
//! without additional configuration.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use iceoryx2_services_tunnel::{Config, Tunnel};
//! use iceoryx2_integrations_zenoh_tunnel_backend::ZenohBackend;
//!
//! let tunnel_config = Config::default();
//! let zenoh_config = zenoh::Config::default();
//! let iceoryx_config = iceoryx2::config::Config::default();
//!
//! let mut tunnel =
//!     Tunnel::<Service, ZenohBackend<Service>>::create(
//!         &tunnel_config, &iceoryx_config, &zenoh_config,
//!     ).expect("failed to create tunnel");
//!
//! loop {
//!     tunnel.discover().expect("discovery failed");
//!     tunnel.propagate().expect("propagation failed");
//! }
//! ```

pub mod backend;
pub mod discovery;
pub mod keys;
pub mod relays;

pub mod testing;

pub use backend::*;
