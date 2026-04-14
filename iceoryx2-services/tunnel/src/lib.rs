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

//! # iceoryx2-services-tunnel
//!
//! Extends iceoryx2 communication beyond the boundary of a single host by
//! tunneling local services over a network transport.
//!
//! The tunnel automatically discovers local iceoryx2 services (currently only
//! publish-subscribe and event messaging patterns) and bridges them to remote
//! hosts through a pluggable backend. On the remote side, an equivalent tunnel
//! ingests the forwarded data and re-publishes it into the local iceoryx2
//! system, making cross-host communication transparent to applications.
//!
//! ## Architecture
//!
//! The tunnel is built around two core operations that are driven by the user:
//!
//! - **Discovery** – detects new services on the local host and on remote hosts
//!   (via the backend), then sets up the necessary iceoryx2 ports and backend
//!   relays for each discovered service.
//! - **Propagation** – forwards payloads and events between the local iceoryx2
//!   ports and the backend relays in both directions (send and ingest).
//!
//! The implementation does not spawn any threads, giving the user complete
//! control over scheduling and execution.
//!
//! ## Backend abstraction
//!
//! The tunnel is generic over the [`Backend`](iceoryx2_services_tunnel_backend::traits::Backend)
//! trait and has no knowledge of the specifics of the transport being used.
//! A custom tunneling mechanism can be provided by implementing the backend
//! traits.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use iceoryx2_services_tunnel::{Config, Tunnel};
//!
//! let tunnel_config = Config::default();
//! let backend_config = Backend::Config::default();
//! let iceoryx_config = iceoryx2::config::Config::default();
//!
//! let mut tunnel =
//!     Tunnel::<Service, Backend>::create(&tunnel_config, &iceoryx_config, &backend_config)
//!         .expect("failed to create tunnel");
//!
//! loop {
//!     tunnel.discover().expect("discovery failed");
//!     tunnel.propagate().expect("propagation failed");
//! }
//! ```

#![no_std]

extern crate alloc;

mod discovery;
mod ports;
mod tunnel;

pub use tunnel::*;
