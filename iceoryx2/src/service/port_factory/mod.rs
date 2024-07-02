// Copyright (c) 2023 Contributors to the Eclipse Foundation
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

use crate::config::Config;
use crate::node::{NodeListFailure, NodeState};

use super::dynamic_config::DynamicConfig;
use super::{attribute::AttributeSet, service_name::ServiceName};

/// Factory to create the endpoints of
/// [`MessagingPattern::Event`](crate::service::messaging_pattern::MessagingPattern::Event) based
/// communication and to acquire static and dynamic service information
pub mod event;

/// Factory to create a [`Listener`](crate::port::listener::Listener)
pub mod listener;

/// Factory to create a [`Notifier`](crate::port::notifier::Notifier)
pub mod notifier;

/// Factory to create the endpoints of
/// [`MessagingPattern::PublishSubscribe`](crate::service::messaging_pattern::MessagingPattern::PublishSubscribe) based
/// communication and to acquire static and dynamic service information
pub mod publish_subscribe;

/// Factory to create a [`Publisher`](crate::port::publisher::Publisher)
pub mod publisher;

/// Factory to create a [`Subscriber`](crate::port::subscriber::Subscriber)
pub mod subscriber;

/// The trait that contains the interface of all port factories for any kind of
/// [`crate::service::messaging_pattern::MessagingPattern`].
pub trait PortFactory {
    /// The underlying [`crate::service::Service`] of the port factory.
    type Service: crate::service::Service;

    /// The underlying type that is used for all static configurations, meaning properties that
    /// never change during the lifetime.
    type StaticConfig;

    /// The underlying type that is used for all dynamic configurations, meaning properties that
    /// change during the lifetime.
    type DynamicConfig;

    /// Returns the [`ServiceName`] of the service
    fn name(&self) -> &ServiceName;

    /// Returns the uuid of the [`crate::service::Service`]
    fn uuid(&self) -> &str;

    /// Returns the attributes defined in the [`crate::service::Service`]
    fn attributes(&self) -> &AttributeSet;

    /// Returns the StaticConfig of the [`crate::service::Service`].
    /// Contains all settings that never change during the lifetime of the service.
    fn static_config(&self) -> &Self::StaticConfig;

    /// Returns the DynamicConfig of the [`crate::service::Service`].
    /// Contains all dynamic settings, like the current participants etc..
    fn dynamic_config(&self) -> &Self::DynamicConfig;

    /// Iterates over all [`Node`](crate::node::Node)s of the [`Service`](crate::service::Service)
    /// and calls for every [`Node`](crate::node::Node) the provided callback. If an error occurs
    /// while acquiring the [`Node`](crate::node::Node)s corresponding [`NodeState`] the error is
    /// forwarded to the callback as input argument.
    fn nodes<F: FnMut(Result<NodeState<Self::Service>, NodeListFailure>)>(&self, callback: F);
}

pub(crate) fn nodes<
    Service: crate::service::Service,
    F: FnMut(Result<NodeState<Service>, NodeListFailure>),
>(
    dynamic_config: &DynamicConfig,
    config: &Config,
    mut callback: F,
) {
    dynamic_config.list_node_ids(|node_id| {
        match crate::node::NodeState::<Service>::new(node_id, config) {
            Ok(Some(node_state)) => callback(Ok(node_state)),
            Ok(None) => (),
            Err(e) => callback(Err(e)),
        }
    });
}
