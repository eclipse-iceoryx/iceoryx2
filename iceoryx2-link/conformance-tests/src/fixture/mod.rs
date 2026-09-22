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
//! * [`AdapterFixture`] for the adapter suites, additionally implementing
//!   [`GatewayFixture`] for the gateway suites.
//!
//! The link suites take either fixture, passed as [`TunnelLinkFixture`]
//! or [`GatewayLinkFixture`] to specify which of its backends the link
//! runs over.
//!
//! The adapter and gateway fixtures also hand out remote endpoints, the
//! application on the far side that a scenario sends to and receives from.
//! What they put on the wire is the fixture's own encoding, not the
//! adapter's, so a scenario checks the adapter against it rather than
//! against itself.
//!
//! A carrier fixture and a middleware fixture:
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
//!
//! impl AdapterFixture for MyMiddlewareFixture {
//!     type Adapter = MyAdapter;
//!     type RemoteEndpoints = MyRemoteMessageEndpoints;
//!
//!     fn new() -> Self { ... }
//!     fn adapter(&mut self) -> MyAdapter { ... }
//!     fn remote_endpoints(&mut self) -> MyRemoteMessageEndpoints { ... }
//! }
//!
//! impl<S: Service> GatewayFixture<S> for MyMiddlewareFixture {
//!     type Mapping = MyMapping;
//!     type Translator = MyTranslator;
//!     type RemoteEndpoints = MyRemotePayloadEndpoints;
//!
//!     fn mapping(&self) -> MyMapping { ... }
//!     fn translator(&self) -> MyTranslator { ... }
//!     fn remote_endpoints_on(&mut self, service: &ServiceDescription) -> MyRemotePayloadEndpoints { ... }
//! }
//! ```
//!
//! And the suites over them:
//!
//! ```ignore
//! instantiate_conformance_tests!(
//!     iceoryx2_link_conformance_tests::carrier_sample,
//!     MyCarrierFixture
//! );
//! instantiate_conformance_tests!(
//!     iceoryx2_link_conformance_tests::tunnel_publish_subscribe,
//!     Ipc,
//!     PublishSubscribe<AnyName, FixedSizePayload<u64>, u64>,
//!     MyCarrierFixture
//! );
//! instantiate_conformance_tests!(
//!     iceoryx2_link_conformance_tests::adapter_publish_subscribe,
//!     MyMiddlewareFixture
//! );
//! instantiate_conformance_tests!(
//!     iceoryx2_link_conformance_tests::gateway_publish_subscribe_payload,
//!     Ipc,
//!     PublishSubscribe<AnyName, FixedSizePayload<u64>, u64>,
//!     MyMiddlewareFixture
//! );
//! instantiate_conformance_tests!(
//!     iceoryx2_link_conformance_tests::link_discovery,
//!     Ipc,
//!     PublishSubscribe<AnyName, FixedSizePayload<u64>, u64>,
//!     TunnelLinkFixture<MyCarrierFixture>
//! );
//! instantiate_conformance_tests!(
//!     iceoryx2_link_conformance_tests::link_discovery,
//!     Ipc,
//!     PublishSubscribe<AnyName, FixedSizePayload<u64>, u64>,
//!     GatewayLinkFixture<MyMiddlewareFixture>
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
