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

//! Conformance tests for the link, its backends and their parts.
//!
//! An instantiation names the suite, the iceoryx2 service variant, the
//! service the scenarios run over and the fixture:
//!
//! ```ignore
//! instantiate_conformance_tests!(
//!     iceoryx2_link_conformance_tests::tunnel_discovery,
//!     Ipc,
//!     PublishSubscribe<AnyName, FixedSizePayload<u64>, u64>,
//!     MyCarrierFixture
//! );
//! ```
//!
//! To test a carrier or a backend built from it:
//!
//! * Implement the corresponding contract of [`fixture`].
//! * Choose the services in [`parameters`] for the messaging pattern it
//!   carries.
//! * Instantiate the suites of its kind with each, in a module per
//!   pattern:
//!
//! ```ignore
//! mod publish_subscribe {
//!     use super::*;
//!
//!     instantiate_conformance_tests!(
//!         iceoryx2_link_conformance_tests::tunnel_discovery,
//!         Ipc,
//!         PublishSubscribe<AnyName, FixedSizePayload<u64>, u64>,
//!         MyCarrierFixture
//!     );
//!     instantiate_conformance_tests!(
//!         iceoryx2_link_conformance_tests::tunnel_publish_subscribe,
//!         Ipc,
//!         PublishSubscribe<AnyName, FixedSizePayload<u64>, u64>,
//!         MyCarrierFixture
//!     );
//!     instantiate_conformance_tests!(
//!         iceoryx2_link_conformance_tests::link_discovery,
//!         Ipc,
//!         PublishSubscribe<AnyName, FixedSizePayload<u64>, u64>,
//!         TunnelLinkFixture<MyCarrierFixture>
//!     );
//! }
//!
//! mod event {
//!     use super::*;
//!
//!     instantiate_conformance_tests!(
//!         iceoryx2_link_conformance_tests::tunnel_discovery,
//!         Ipc,
//!         Event<AnyName>,
//!         MyCarrierFixture
//!     );
//!     instantiate_conformance_tests!(
//!         iceoryx2_link_conformance_tests::tunnel_event,
//!         Ipc,
//!         Event<AnyName>,
//!         MyCarrierFixture
//!     );
//!     instantiate_conformance_tests!(
//!         iceoryx2_link_conformance_tests::link_discovery,
//!         Ipc,
//!         Event<AnyName>,
//!         TunnelLinkFixture<MyCarrierFixture>
//!     );
//! }
//! ```
//!
//! A fixture of each kind can be seen in `tests-common`.

#![cfg_attr(not(any(test, feature = "std")), no_std)]

extern crate alloc;

mod carrier;
pub use carrier::discovery as carrier_discovery;
pub use carrier::propagation as carrier_propagation;
pub use carrier::wake as carrier_wake;
pub mod fixture;
mod link;
pub use link::discovery as link_discovery;
pub use link::wake as link_wake;
pub mod parameters;
pub mod testing;
mod tunnel;
pub use tunnel::discovery as tunnel_discovery;
pub use tunnel::event as tunnel_event;
pub use tunnel::publish_subscribe as tunnel_publish_subscribe;
