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

#![doc = include_str!("../README.md")]
//!
//! ## Running a link
//!
//! A [`Link`] is built from a node and a backend and then leaves the user
//! to decide how it is driven.
//!
//! [Bridges](Bridges) are the component that extend the communication of one
//! service from its `iceoryx2` ports into the backend.
//!
//! [`Link::discover`] opens and closes bridges to match what both sides
//! offer, and [`Link::propagate`] moves the samples and notifications
//! pending on every bridge.
//!
//! Polled manually in a loop:
//!
//! ```ignore
//! let node = NodeBuilder::new().create::<ipc::Service>()?;
//! let mut link = Link::new(node, Tunnel::new(carrier, &config));
//!
//! loop {
//!     link.discover()?;
//!     link.propagate()?;
//!     sleep(Duration::from_millis(100));
//! }
//! ```
//!
//! Reactive, waking up whenever the backend says one may have something to
//! do, for backends that are [`Reactive`](iceoryx2_link_backend::Reactive):
//!
//! ```ignore
//! let node = NodeBuilder::new().create::<ipc::Service>()?;
//! let mut link = Link::new(node, Tunnel::new(carrier, &config));
//! let listener = link.listener()?;
//!
//! loop {
//!     listener.blocking_wait(|_| {})?;
//!     link.discover()?;
//!     link.propagate()?;
//! }
//! ```
//!
//! * [`Link::bridges`] tells what is bridged, by service hash, both the
//!   local services the link exports and the remote ones it mirrors.
//! * [`Link::node`] is the node representing the link in the local system.
//! * [`Link::with_filter`] restricts the link to the services a filter
//!   admits by name, those it exports and the mirrors it creates alike.
//! * [`Link::with_monitoring`] reports at trace level what every bridge
//!   moved in each direction, once per propagation.
//! * A failed cycle reports a [`DiscoveryError`] or a [`PropagateError`]
//!   but leaves the link usable for the next cycle.
//!
//! ## Extending a link
//!
//! * A new carrier implements the contract of `iceoryx2-link-carrier` and
//!   runs under the tunnel.
//! * A new kind of backend implements [`Backend`](iceoryx2_link_backend::Backend)
//!   itself.
//!
//! Implementations should instantiate the suites of
//! `iceoryx2-link-conformance-tests` to verify the required contract
//! is upheld.

#![no_std]

extern crate alloc;

mod announcement_state;
mod bridge;
mod diagnostics;
mod discovery_state;
mod link;
mod ports;
mod wake;

pub use bridge::{Bridges, PropagateError};
pub use link::{DiscoveryError, Link};
pub use wake::WakeCreationError;
