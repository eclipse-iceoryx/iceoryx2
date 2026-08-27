// Copyright (c) 2025 Contributors to the Eclipse Foundation
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
use iceoryx2_gateway_backend::traits::{
    Backend, BackendBuilder, Identity, Mapping, Passthrough, ReactiveBackendBuilder, Translator,
};
use iceoryx2_gateway_backend::types::service_description::ServiceDescription;
use iceoryx2_gateway_backend::types::wake::WakeHandle;
use iceoryx2_log::{fail, trace};

use zenoh::{Config, Session, Wait};

use crate::{
    discovery::Discovery,
    relays::{Factory, event, publish_subscribe},
};

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum CreationError {
    Session,
    Discovery,
}

impl core::fmt::Display for CreationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "CreationError::{self:?}")
    }
}

impl core::error::Error for CreationError {}

#[derive(Debug)]
pub struct ZenohBackend<
    S: Service,
    M: Mapping<EndpointDescription = ServiceDescription> = Identity,
    T: Translator<EndpointDescription = <M as Mapping>::EndpointDescription> = Passthrough<
        <M as Mapping>::EndpointDescription,
    >,
> {
    session: Session,
    discovery: Discovery<S, M>,
    /// `Some` when constructed in reactive mode. Cloned into each relay's
    /// subscriber callback so that incoming network data signals the wake.
    wake: Option<Arc<WakeHandle<local_threadsafe::Service>>>,
    mapping: Arc<M>,
    #[allow(dead_code)]
    translator: T,
    _phantom: core::marker::PhantomData<S>,
}

impl<
    S: Service,
    M: Mapping<EndpointDescription = ServiceDescription>,
    T: Translator<EndpointDescription = M::EndpointDescription>,
> Backend<S> for ZenohBackend<S, M, T>
{
    type Config = Config;
    type Mapping = M;
    type Translator = T;
    type CreationError = CreationError;
    type Builder<'config>
        = Builder<'config, S, M, T>
    where
        Self::Config: 'config;
    type Discovery = Discovery<S, M>;

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
        Self::RelayFactory::new(&self.session, self.wake.clone())
    }

    fn discovery(&mut self) -> &mut impl iceoryx2_gateway_backend::traits::Discovery {
        &mut self.discovery
    }

    fn mapping(&self) -> &Self::Mapping {
        &self.mapping
    }
}

/// Builder for [`ZenohBackend`].
#[derive(Debug)]
pub struct Builder<
    'config,
    S: Service,
    M: Mapping<EndpointDescription = ServiceDescription> = Identity,
    T: Translator<EndpointDescription = <M as Mapping>::EndpointDescription> = Passthrough<
        <M as Mapping>::EndpointDescription,
    >,
> {
    config: &'config Config,
    wake: Option<WakeHandle<local_threadsafe::Service>>,
    mapping: M,
    translator: T,
    _phantom: core::marker::PhantomData<S>,
}

impl<
    'config,
    S: Service,
    M: Mapping<EndpointDescription = ServiceDescription>,
    T: Translator<EndpointDescription = M::EndpointDescription>,
> Builder<'config, S, M, T>
{
    pub fn new(config: &'config Config) -> Self {
        Self {
            config,
            wake: None,
            mapping: M::default(),
            translator: T::default(),
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<
    S: Service,
    M: Mapping<EndpointDescription = ServiceDescription>,
    T: Translator<EndpointDescription = M::EndpointDescription>,
> BackendBuilder<S> for Builder<'_, S, M, T>
{
    type Backend = ZenohBackend<S, M, T>;
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
        let origin = "ZenohBackend::Builder::create";

        trace!(
            from origin,
            "Initializing Zenoh backend"
        );

        let session = zenoh::open(self.config.clone()).wait();
        let session = fail!(
            from origin,
            when session,
            with CreationError::Session,
            "Failed to create zenoh session"
        );

        let mapping = Arc::new(self.mapping);
        let discovery = Discovery::create(&session, Arc::clone(&mapping));
        let discovery = fail!(
            from origin,
            when discovery,
            with CreationError::Discovery,
            "Failed to create zenoh discovery"
        );

        Ok(ZenohBackend {
            session,
            discovery,
            wake: self.wake.map(Arc::new),
            mapping,
            translator: self.translator,
            _phantom: core::marker::PhantomData,
        })
    }
}

impl<
    S: Service,
    M: Mapping<EndpointDescription = ServiceDescription>,
    T: Translator<EndpointDescription = M::EndpointDescription>,
> ReactiveBackendBuilder<S> for Builder<'_, S, M, T>
{
    type WakeService = local_threadsafe::Service;

    fn reactive(mut self, wake: WakeHandle<local_threadsafe::Service>) -> Self {
        self.wake = Some(wake);
        self
    }
}
