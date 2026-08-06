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

//! # iceoryx2-gateway
//!
//! Extends iceoryx2 communication beyond the boundary of a single host by
//! bridging local services over a network transport.
//!
//! The gateway automatically discovers local iceoryx2 services (currently only
//! publish-subscribe and event messaging patterns) and bridges them to remote
//! hosts through a pluggable backend. On the remote side, an equivalent gateway
//! ingests the forwarded data and re-publishes it into the local iceoryx2
//! system, making cross-host communication transparent to applications.
//!
//! ## Architecture
//!
//! The gateway is built around two core operations that are driven by the user:
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
//! The gateway is generic over the [`Backend`](iceoryx2_gateway_backend::traits::Backend)
//! trait and has no knowledge of the specifics of the transport being used.
//! A custom bridging mechanism can be provided by implementing the backend
//! traits.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use iceoryx2_gateway::Gateway;
//!
//! let mut gateway = Gateway::<Service, Backend>::new()
//!     .polled()
//!     .create()
//!     .expect("failed to create gateway");
//!
//! loop {
//!     gateway.discover().expect("discovery failed");
//!     gateway.propagate().expect("propagation failed");
//! }
//! ```

#![no_std]

extern crate alloc;

mod bridge;
mod builder;
mod discovery;
mod gateway;
mod ports;

pub use builder::{GatewayBuilder, Polled, Reactive, Unconfigured};
pub use gateway::*;
