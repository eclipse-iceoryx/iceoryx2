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

//! The contracts a fixture implements, one per suite family.
//!
//! * [`CarrierFixture`] for the carrier suites, additionally implementing
//!   [`TunnelFixture`] for the tunnel suites.
//!
//! The link suites take the tunnel fixture, passed as [`TunnelLinkFixture`]
//! to run the link over its tunnels.
//!
//! A carrier fixture:
//!
//! ```ignore
//! impl CarrierFixture for MyCarrierFixture {
//!     type Carrier = MyCarrier;
//!
//!     fn new() -> Self { ... }
//!     fn carrier(&mut self) -> MyCarrier { ... }
//! }
//!
//! impl TunnelFixture for MyCarrierFixture {}
//! ```
//!
//! And the suites over it:
//!
//! ```ignore
//! instantiate_conformance_tests!(
//!     iceoryx2_link_conformance_tests::carrier_propagation,
//!     MyCarrierFixture
//! );
//! instantiate_conformance_tests!(
//!     iceoryx2_link_conformance_tests::tunnel_publish_subscribe,
//!     Ipc,
//!     PublishSubscribe<AnyName, FixedSizePayload<u64>, u64>,
//!     MyCarrierFixture
//! );
//! instantiate_conformance_tests!(
//!     iceoryx2_link_conformance_tests::link_discovery,
//!     Ipc,
//!     PublishSubscribe<AnyName, FixedSizePayload<u64>, u64>,
//!     TunnelLinkFixture<MyCarrierFixture>
//! );
//! ```

pub mod adapter;
pub mod carrier;
pub mod gateway;
pub mod link;
pub mod tunnel;

pub use adapter::{AdapterFixture, MessageEndpoints};
pub use carrier::CarrierFixture;
pub use gateway::{DiscoverableEndpoints, GatewayFixture, PayloadEndpoints, SampleEndpoints};
pub use link::{GatewayLinkFixture, LinkFixture, TunnelLinkFixture};
pub use tunnel::TunnelFixture;
