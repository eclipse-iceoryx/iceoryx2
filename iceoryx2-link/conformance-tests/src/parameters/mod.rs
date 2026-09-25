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

//! The service a suite runs over, one type per messaging pattern.
//!
//! * [`PublishSubscribe<N, P, H>`], services named by `N`, of the payload
//!   `P` under the user header `H`.
//! * [`Event<N>`], services named by `N`.
//!
//! A carrier or tunnel fixture carries anything, so it passes [`AnyName`]
//! with whatever payload and header it likes:
//!
//! ```ignore
//! instantiate_conformance_tests!(
//!     iceoryx2_link_conformance_tests::tunnel_publish_subscribe,
//!     Ipc,
//!     PublishSubscribe<AnyName, FixedSizePayload<u64>, u64>,
//!     MyCarrierFixture
//! );
//! instantiate_conformance_tests!(
//!     iceoryx2_link_conformance_tests::tunnel_publish_subscribe,
//!     Ipc,
//!     PublishSubscribe<AnyName, SlicePayload<u8>, ()>,
//!     MyCarrierFixture
//! );
//! instantiate_conformance_tests!(
//!     iceoryx2_link_conformance_tests::tunnel_event,
//!     Ipc,
//!     Event<AnyName>,
//!     MyCarrierFixture
//! );
//! ```
//!
//! An adapter or gateway fixture must pass services its mapping and
//! translator accept, and supplies three things for that.
//!
//! * `N`, a source of names its mapping covers. [`AnyName`] when the
//!   mapping covers any name, else a type of its own implementing
//!   [`PublishSubscribeName`] or [`EventName`] depending on the supported
//!   pattern.
//! * `P`, [`FixedSizePayload<T>`] or [`SlicePayload<T>`], `T` being the
//!   payload type of the iceoryx2 services its translator translates.
//! * `H`, the user header its translator accepts, `()` for none.
//!
//! For a translator carrying a C struct under a mapping with configured
//! names:
//!
//! ```ignore
//! #[repr(C)]
//! #[derive(Debug, Clone, Copy, PartialEq, ZeroCopySend)]
//! #[type_name("my_msgs/msg/Counter")]
//! struct Counter {
//!     value: u64,
//! }
//!
//! impl From<u64> for Counter {
//!     fn from(value: u64) -> Self {
//!         Self { value }
//!     }
//! }
//!
//! struct MyMapping;
//!
//! impl PublishSubscribeName for MyMapping {
//!     fn service_name() -> ServiceName {
//!         ServiceName::new("my/configured/service").expect("a valid service name")
//!     }
//! }
//! ```
//!
//! And the gateway suites over it:
//!
//! ```ignore
//! instantiate_conformance_tests!(
//!     iceoryx2_link_conformance_tests::gateway_discovery,
//!     Ipc,
//!     PublishSubscribe<MyMapping, FixedSizePayload<Counter>, ()>,
//!     MyMiddlewareFixture
//! );
//! instantiate_conformance_tests!(
//!     iceoryx2_link_conformance_tests::gateway_publish_subscribe_payload,
//!     Ipc,
//!     PublishSubscribe<MyMapping, FixedSizePayload<Counter>, ()>,
//!     MyMiddlewareFixture
//! );
//! ```
//!
//! The scenarios run against [`AnyService`], [`PublishSubscribeService`]
//! and [`EventService`], which [`PublishSubscribe`] and [`Event`]
//! implement. A fixture writes the name source and nothing more.

pub mod event;
pub mod payload;
pub mod publish_subscribe;

pub use event::{Event, EventName, EventService};
pub use payload::{FixedSizePayload, PayloadShape, SliceElement, SlicePayload};
pub(crate) use publish_subscribe::Foreign;
pub use publish_subscribe::{
    Header, Payload, PublishSubscribe, PublishSubscribeName, PublishSubscribePayload,
    PublishSubscribeService, UserHeader, Value,
};

use alloc::vec::Vec;

use iceoryx2::config::Config;
use iceoryx2::node::Node;
use iceoryx2::service::Service;
use iceoryx2::service::messaging_pattern::MessagingPattern;
use iceoryx2::service::service_name::ServiceName;
use iceoryx2_link_backend::service_description::ServiceDescription;

/// A name source for fixtures whose mapping covers any name, each one
/// generated fresh.
#[derive(Debug, Default, Clone, Copy)]
pub struct AnyName;

/// A service of any messaging pattern the suites run over.
pub trait AnyService {
    /// The pattern the service is of.
    const PATTERN: MessagingPattern;
    /// The created service, alive until dropped.
    type Handle<S: Service>;
    /// A port on the service.
    type Port<S: Service>;

    /// A fresh name the mapping covers.
    fn service_name() -> ServiceName;

    /// The description a service of this pattern has under `config`.
    fn describe<S: Service>(name: &ServiceName, config: &Config) -> ServiceDescription;

    /// Creates the service on `node`.
    fn create<S: Service>(node: &Node<S>, name: &ServiceName) -> Self::Handle<S>;

    /// Opens the service on `node`.
    fn open<S: Service>(node: &Node<S>, name: &ServiceName) -> Self::Handle<S>;

    /// A port on `service`.
    fn create_port<S: Service>(service: &Self::Handle<S>) -> Self::Port<S>;

    /// Every port slot `config` allows on `service`, held so the link
    /// cannot open its own.
    fn occupy_ports<S: Service>(service: &Self::Handle<S>, config: &Config) -> Vec<Self::Port<S>>;
}
