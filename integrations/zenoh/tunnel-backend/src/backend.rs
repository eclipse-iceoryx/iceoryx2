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
use iceoryx2_log::{fail, trace};
use iceoryx2_services_tunnel_backend::traits::{Backend, BackendBuilder, ReactiveBackendBuilder};
use iceoryx2_services_tunnel_backend::types::wake::WakeHandle;

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
pub struct ZenohBackend<S: Service> {
    session: Session,
    discovery: Discovery,
    /// `Some` when constructed in reactive mode. Cloned into each relay's
    /// subscriber callback so that incoming network data signals the wake.
    wake: Option<Arc<WakeHandle<local_threadsafe::Service>>>,
    _phantom: core::marker::PhantomData<S>,
}

impl<S: Service> Backend<S> for ZenohBackend<S> {
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
        = Factory<'b, S>
    where
        Self: 'b;

    fn builder(config: &Self::Config) -> Self::Builder<'_> {
        Builder::new(config)
    }

    fn relay_builder(&self) -> Self::RelayFactory<'_> {
        Self::RelayFactory::new(&self.session, self.wake.clone())
    }

    fn discovery(&self) -> &impl iceoryx2_services_tunnel_backend::traits::Discovery {
        &self.discovery
    }
}

/// Builder for [`ZenohBackend`].
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
    type Backend = ZenohBackend<S>;
    type CreationError = CreationError;

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

        let discovery = Discovery::create(&session);
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
