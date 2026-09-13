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

pub mod event;
pub mod payload;
pub mod publish_subscribe;

pub use event::{Event, EventName, EventService};
pub use payload::{FixedSizePayload, PayloadShape, SlicePayload};
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
use iceoryx2_link_backend::description::ServiceDescription;

/// A name source for fixtures that accept any name, each one generated
/// fresh.
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

    /// A fresh service name.
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
