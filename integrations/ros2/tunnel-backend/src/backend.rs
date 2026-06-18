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

use std::rc::Rc;
use std::sync::Arc;

use iceoryx2::service::{Service, local_threadsafe};
use iceoryx2_log::fail;
use iceoryx2_services_tunnel_backend::traits::{Backend, BackendBuilder, ReactiveBackendBuilder};
use iceoryx2_services_tunnel_backend::types::wake::WakeHandle;

use crate::{
    discovery::Discovery,
    rcl,
    relays::{Factory, event, publish_subscribe},
    typesupport::TypeSupportRegistry,
};

/// The name of the ROS 2 node representing the tunnel.
const NODE_NAME: &str = "iceoryx2_tunnel";

/// A ROS 2 topic to bridge.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TopicConfig {
    /// Fully-qualified ROS 2 topic name, e.g. `/Camera/FrontRight`.
    pub topic: String,
    /// ROS 2 type name, e.g. `geometry_msgs/msg/Twist`.
    pub type_name: String,
}

/// Configuration for the [`Ros2Backend`].
#[derive(Debug, Default)]
pub struct Config {
    /// The topics to bridge. Typesupport for every entry is resolved during
    /// backend creation, which fails if any cannot be resolved.
    pub topics: Vec<TopicConfig>,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum CreationError {
    Node,
    TypeSupport,
}

impl core::fmt::Display for CreationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "CreationError::{self:?}")
    }
}

impl core::error::Error for CreationError {}

#[derive(Debug)]
pub struct Ros2Backend<S: Service> {
    node: rcl::Node,
    /// Typesupport for all configured topics, loaded dynamically on
    /// initialization.
    type_support: Rc<TypeSupportRegistry>,
    discovery: Discovery,
    /// `Some` when constructed in reactive mode. Cloned into each relay so
    /// that incoming ROS 2 data signals the wake.
    wake: Option<Arc<WakeHandle<local_threadsafe::Service>>>,
    _phantom: core::marker::PhantomData<S>,
}

impl<S: Service> Backend<S> for Ros2Backend<S> {
    type Config = Config;
    type CreationError = CreationError;
    type Builder<'config>
        = Builder<'config, S>
    where
        Self::Config: 'config;
    type Discovery = Discovery;

    type PublishSubscribeRelay = publish_subscribe::Relay<S>;
    type EventRelay = event::Relay<S>;

    type RelayFactory<'b>
        = Factory<S>
    where
        Self: 'b;

    fn builder(config: &Self::Config) -> Self::Builder<'_> {
        Builder::new(config)
    }

    fn relay_builder(&self) -> Self::RelayFactory<'_> {
        Factory::new(
            self.node.clone(),
            self.type_support.clone(),
            self.wake.clone(),
        )
    }

    fn discovery(&self) -> &impl iceoryx2_services_tunnel_backend::traits::Discovery {
        &self.discovery
    }
}

/// Builder for [`Ros2Backend`].
#[derive(Debug)]
pub struct Builder<'config, S: Service> {
    config: &'config Config,
    wake: Option<WakeHandle<local_threadsafe::Service>>,
    _phantom: core::marker::PhantomData<S>,
}

impl<'config, S: Service> Builder<'config, S> {
    pub fn new(config: &'config Config) -> Self {
        Self {
            config,
            wake: None,
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<S: Service> BackendBuilder<S> for Builder<'_, S> {
    type Backend = Ros2Backend<S>;
    type CreationError = CreationError;

    fn create(self) -> Result<Self::Backend, Self::CreationError> {
        let origin = "Ros2Backend::create";

        let node = fail!(from origin,
            when rcl::Node::new(NODE_NAME).create(),
            with CreationError::Node,
            "Failed to create ROS 2 node"
        );

        // Load all typesupport libraries for configured topics during
        // initialization.
        let type_support = Rc::new(TypeSupportRegistry::default());
        for topic in &self.config.topics {
            fail!(from origin,
                when type_support.load(&topic.type_name),
                with CreationError::TypeSupport,
                "Failed to load typesupport for configured topic '{}'", topic.type_name
            );
        }

        Ok(Ros2Backend {
            node,
            type_support,
            discovery: Discovery {},
            wake: self.wake.map(Arc::new),
            _phantom: core::marker::PhantomData,
        })
    }
}

impl<S: Service> ReactiveBackendBuilder<S> for Builder<'_, S> {
    type WakeService = local_threadsafe::Service;

    fn reactive(mut self, wake: WakeHandle<local_threadsafe::Service>) -> Self {
        self.wake = Some(wake);
        self
    }
}
