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

use std::sync::Arc;

use iceoryx2::service::{Service, local_threadsafe};
use iceoryx2_log::fail;
use iceoryx2_services_tunnel_backend::traits::{Backend, BackendBuilder, ReactiveBackendBuilder};
use iceoryx2_services_tunnel_backend::types::wake::WakeHandle;

use crate::NODE_NAME;
use crate::{
    discovery::Discovery,
    rcl,
    relays::{Factory, event, publish_subscribe},
    typesupport::TypeSupportRegistry,
};

/// A ROS 2 topic to bridge.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TopicConfig {
    /// Fully-qualified ROS 2 topic name, e.g. `/Camera/FrontRight`.
    pub(crate) topic: rcl::TopicName,
    /// ROS 2 type name, e.g. `geometry_msgs/msg/Twist`.
    pub(crate) type_name: rcl::TypeName,
}

impl TopicConfig {
    /// Creates a config entry, validating the ROS 2 topic name and message
    /// type name.
    ///
    /// * The topic must be a valid ROS 2 topic name
    /// * The type name must have the form `package/msg/Message`
    pub fn new(topic: &str, type_name: &str) -> Result<Self, TopicConfigError> {
        let origin = "TopicConfig::new";

        let topic = fail!(from origin,
            when rcl::TopicName::new(topic),
            with TopicConfigError::InvalidTopic,
            "Failed to create topic config from invalid topic name '{}'",
            topic
        );
        let type_name = fail!(from origin,
            when rcl::TypeName::new(type_name),
            with TopicConfigError::InvalidTypeName,
            "Failed to create topic config from invalid type name '{}'",
            type_name
        );

        Ok(Self { topic, type_name })
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum TopicConfigError {
    /// The topic is not a valid ROS 2 topic name.
    InvalidTopic,
    /// The type name is not of the form `package/msg/Message`.
    InvalidTypeName,
}

impl core::fmt::Display for TopicConfigError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "TopicConfigError::{self:?}")
    }
}

impl core::error::Error for TopicConfigError {}

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
    node: rcl::NodeHandle,
    /// Typesupport for all configured topics, loaded on initialization.
    type_registry: TypeSupportRegistry,
    discovery: Discovery<S>,
    /// `Some` when constructed in reactive mode. Cloned into each relay so
    /// that incoming ROS 2 data signals the wake.
    wake: Option<Arc<WakeHandle<local_threadsafe::Service>>>,
    _phantom: core::marker::PhantomData<S>,
}

impl<S: Service> Backend<S> for Ros2Backend<S> {
    type Config = Config;
    type CreationError = CreationError;

    type Builder<'a>
        = Builder<'a, S>
    where
        Self::Config: 'a;

    type Discovery = Discovery<S>;

    type PublishSubscribeRelay = publish_subscribe::Relay<S>;
    type EventRelay = event::Relay<S>;

    type RelayFactory<'b>
        = Factory<'b, S>
    where
        Self: 'b;

    fn builder(config: &Self::Config) -> Self::Builder<'_> {
        Builder::new(config)
    }

    fn relay_builder(&self) -> Self::RelayFactory<'_> {
        Factory::new(self.node.clone(), &self.type_registry, self.wake.clone())
    }

    fn discovery(&self) -> &impl iceoryx2_services_tunnel_backend::traits::Discovery {
        &self.discovery
    }
}

/// Builder for [`Ros2Backend`].
#[derive(Debug)]
pub struct Builder<'a, S: Service> {
    config: &'a Config,
    wake: Option<WakeHandle<local_threadsafe::Service>>,
    _phantom: core::marker::PhantomData<S>,
}

impl<'a, S: Service> Builder<'a, S> {
    pub fn new(config: &'a Config) -> Self {
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
            when rcl::NodeHandle::new(NODE_NAME).create(),
            with CreationError::Node,
            "Failed to create ROS 2 node"
        );

        // Load all typesupport libraries for configured topics during
        // initialization.
        let type_registry = TypeSupportRegistry::default();
        for topic in &self.config.topics {
            fail!(from origin,
                when type_registry.load(topic.type_name.as_str()),
                with CreationError::TypeSupport,
                "Failed to load typesupport for configured topic '{}'", topic.type_name.as_str()
            );
        }

        let discovery = Discovery::new(node.clone(), &self.config.topics);

        Ok(Ros2Backend {
            node,
            type_registry,
            discovery,
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
