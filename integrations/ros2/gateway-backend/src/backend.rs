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
use iceoryx2_gateway_backend::traits::{
    Backend, BackendBuilder, Mapping, Passthrough, ReactiveBackendBuilder, Translator,
};
use iceoryx2_gateway_backend::types::wake::WakeHandle;
use iceoryx2_log::fail;

use crate::NODE_NAME;
use crate::config::Config;
use crate::mapping::{PrefixMapping, TopicDescription};
use crate::{
    discovery::Discovery,
    rcl::{RclNode, RclNodeBuilder},
    relays::{Factory, event, publish_subscribe},
    typesupport,
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
pub struct Ros2Backend<
    S: Service,
    M: Mapping<EndpointDescription = TopicDescription> = PrefixMapping,
    T: Translator<EndpointDescription = TopicDescription> = Passthrough<TopicDescription>,
> {
    node: Rc<RclNode>,
    discovery: Discovery<S, M, T>,
    mapping: Rc<M>,
    translator: Rc<T>,
    wake: Option<Arc<WakeHandle<local_threadsafe::Service>>>,
    _phantom: core::marker::PhantomData<S>,
}

impl<
    S: Service,
    M: Mapping<EndpointDescription = TopicDescription>,
    T: Translator<EndpointDescription = TopicDescription>,
> Backend<S> for Ros2Backend<S, M, T>
{
    type Config = Config;
    type Translator = T;
    type Mapping = M;
    type CreationError = CreationError;

    type Builder<'a>
        = Builder<'a, S, M, T>
    where
        Self::Config: 'a;

    type Discovery = Discovery<S, M, T>;

    type PublishSubscribeRelay = publish_subscribe::Relay<S, T>;
    type EventRelay = event::Relay<S>;

    type RelayFactory<'b>
        = Factory<'b, S, M, T>
    where
        Self: 'b;

    fn builder(config: &Self::Config) -> Self::Builder<'_> {
        Builder::new(config)
    }

    fn relay_builder(&self) -> Self::RelayFactory<'_> {
        Factory::new(
            Rc::clone(&self.node),
            &self.mapping,
            Rc::clone(&self.translator),
            self.wake.clone(),
        )
    }

    fn discovery(&self) -> &impl iceoryx2_gateway_backend::traits::Discovery {
        &self.discovery
    }

    fn mapping(&self) -> &Self::Mapping {
        &self.mapping
    }
}

/// Builder for [`Ros2Backend`].
#[derive(Debug)]
pub struct Builder<
    'a,
    S: Service,
    M: Mapping<EndpointDescription = TopicDescription> = PrefixMapping,
    T: Translator<EndpointDescription = TopicDescription> = Passthrough<TopicDescription>,
> {
    config: &'a Config,
    mapping: M,
    translator: T,
    wake: Option<WakeHandle<local_threadsafe::Service>>,
    _phantom: core::marker::PhantomData<S>,
}

impl<
    'a,
    S: Service,
    M: Mapping<EndpointDescription = TopicDescription>,
    T: Translator<EndpointDescription = TopicDescription>,
> Builder<'a, S, M, T>
{
    pub fn new(config: &'a Config) -> Self {
        Self {
            config,
            mapping: M::default(),
            translator: T::default(),
            wake: None,
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<
    S: Service,
    M: Mapping<EndpointDescription = TopicDescription>,
    T: Translator<EndpointDescription = TopicDescription>,
> BackendBuilder<S> for Builder<'_, S, M, T>
{
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

        // Resolve the requested typesupport libraries during initialization.
        // Typesupport not loaded here will be lazily loaded.
        for type_name in &self.config.preload_types {
            fail!(from origin,
                when typesupport::load(type_name.as_str()),
                with CreationError::TypeSupport,
                "Failed to preload typesupport for type '{}'", type_name.as_str()
            );
        }

        let mapping = Rc::new(self.mapping);
        let translator = Rc::new(self.translator);
        let discovery = Discovery::new(
            Rc::clone(&node),
            &self.config.topics,
            Rc::clone(&mapping),
            Rc::clone(&translator),
        );

        Ok(Ros2Backend {
            node,
            discovery,
            mapping,
            translator,
            wake: self.wake.map(Arc::new),
            _phantom: core::marker::PhantomData,
        })
    }
}

impl<
    S: Service,
    M: Mapping<EndpointDescription = TopicDescription>,
    T: Translator<EndpointDescription = TopicDescription>,
> ReactiveBackendBuilder<S> for Builder<'_, S, M, T>
{
    type WakeService = local_threadsafe::Service;

    fn reactive(mut self, wake: WakeHandle<local_threadsafe::Service>) -> Self {
        self.wake = Some(wake);
        self
    }
}
