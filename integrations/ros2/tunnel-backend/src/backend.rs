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
use iceoryx2_services_tunnel_backend::traits::{
    Backend, BackendBuilder, Identity, Mapping, Passthrough, ReactiveBackendBuilder, Translator,
};
use iceoryx2_services_tunnel_backend::types::wake::WakeHandle;

use crate::NODE_NAME;
use crate::config::Config;
use crate::{
    discovery::Discovery,
    rcl::{RclNode, RclNodeBuilder},
    relays::{Factory, event, publish_subscribe},
    typesupport::TypeSupportRegistry,
};

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
pub struct Ros2Backend<S: Service, M: Mapping = Identity, T: Translator = Passthrough> {
    node: Rc<RclNode>,
    /// Typesupport for all configured topics, loaded on initialization.
    type_registry: TypeSupportRegistry,
    discovery: Discovery<S>,
    /// `Some` when constructed in reactive mode. Cloned into each relay so
    /// that incoming ROS 2 data signals the wake.
    wake: Option<Arc<WakeHandle<local_threadsafe::Service>>>,
    mapping: M,
    #[allow(dead_code)]
    translator: T,
    _phantom: core::marker::PhantomData<S>,
}

impl<S: Service, M: Mapping, T: Translator> Backend<S> for Ros2Backend<S, M, T> {
    type Config = Config;
    type Mapping = M;
    type Translator = T;
    type CreationError = CreationError;

    type Builder<'a>
        = Builder<'a, S, M, T>
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
        Factory::new(
            Rc::clone(&self.node),
            &self.type_registry,
            self.wake.clone(),
        )
    }

    fn discovery(&self) -> &impl iceoryx2_services_tunnel_backend::traits::Discovery {
        &self.discovery
    }

    fn mapping(&self) -> &Self::Mapping {
        &self.mapping
    }
}

/// Builder for [`Ros2Backend`].
#[derive(Debug)]
pub struct Builder<'a, S: Service, M: Mapping = Identity, T: Translator = Passthrough> {
    config: &'a Config,
    wake: Option<WakeHandle<local_threadsafe::Service>>,
    mapping: M,
    translator: T,
    _phantom: core::marker::PhantomData<S>,
}

impl<'a, S: Service, M: Mapping, T: Translator> Builder<'a, S, M, T> {
    pub fn new(config: &'a Config) -> Self {
        Self {
            config,
            wake: None,
            mapping: M::default(),
            translator: T::default(),
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<S: Service, M: Mapping, T: Translator> BackendBuilder<S> for Builder<'_, S, M, T> {
    type Backend = Ros2Backend<S, M, T>;
    type CreationError = CreationError;

    fn translator(mut self, translator: T) -> Self {
        self.translator = translator;
        self
    }

    fn mapping(mut self, mapping: M) -> Self {
        self.mapping = mapping;
        self
    }

    fn create(self) -> Result<Self::Backend, Self::CreationError> {
        let origin = "Ros2Backend::create";

        let node = Rc::new(fail!(from origin,
            when RclNodeBuilder::new(NODE_NAME).create(),
            with CreationError::Node,
            "Failed to create ROS 2 node"
        ));

        // Load all typesupport libraries for configured topics during
        // initialization.
        let type_registry = TypeSupportRegistry::default();
        for topic in &self.config.topics {
            let type_name = topic.type_name.as_str();
            fail!(from origin,
                when type_registry.load(type_name),
                with CreationError::TypeSupport,
                "Failed to load typesupport for configured topic '{}'", type_name
            );
        }

        let discovery = Discovery::new(Rc::clone(&node), &self.config.topics);

        Ok(Ros2Backend {
            node,
            type_registry,
            discovery,
            wake: self.wake.map(Arc::new),
            mapping: self.mapping,
            translator: self.translator,
            _phantom: core::marker::PhantomData,
        })
    }
}

impl<S: Service, M: Mapping, T: Translator> ReactiveBackendBuilder<S> for Builder<'_, S, M, T> {
    type WakeService = local_threadsafe::Service;

    fn reactive(mut self, wake: WakeHandle<local_threadsafe::Service>) -> Self {
        self.wake = Some(wake);
        self
    }
}
