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

#![warn(clippy::alloc_instead_of_core)]
#![warn(clippy::std_instead_of_alloc)]
#![warn(clippy::std_instead_of_core)]

use alloc::rc::Rc;
use iceoryx2::service::Service;
use iceoryx2_services_tunnel_backend::traits::{Backend, BackendBuilder};

use crate::backend::{
    discovery::Discovery,
    relays::{self, factory::Factory},
    session::{self, Session},
};

#[derive(Debug, Clone, Default)]
pub struct Config;

#[derive(Debug)]
pub enum CreationError {
    CreateSession(session::CreationError),
}

impl core::fmt::Display for CreationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "CreationError::{self:?}")
    }
}

impl core::error::Error for CreationError {}

#[derive(Debug)]
pub struct TestBackend<S: Service> {
    session: Rc<Session>,
    discovery: Discovery,
    _phantom: core::marker::PhantomData<S>,
}

impl<S: Service> Backend<S> for TestBackend<S> {
    type Config = Config;
    type CreationError = CreationError;
    type Builder<'config>
        = Builder<'config, S>
    where
        Self::Config: 'config;

    type Discovery = Discovery;

    type PublishSubscribeRelay = relays::publish_subscribe::Relay<S>;
    type EventRelay = relays::event::Relay<S>;

    type RelayFactory<'a>
        = Factory<S>
    where
        Self: 'a;

    fn builder(config: &Self::Config) -> Self::Builder<'_> {
        Builder::new(config)
    }

    fn discovery(&self) -> &impl iceoryx2_services_tunnel_backend::traits::Discovery {
        &self.discovery
    }

    fn relay_builder(&self) -> Self::RelayFactory<'_> {
        Factory::new(self.session.clone())
    }
}

/// Builder for [`TestBackend`].
#[derive(Debug)]
pub struct Builder<'config, S: Service> {
    _config: &'config Config,
    _phantom: core::marker::PhantomData<S>,
}

impl<'config, S: Service> Builder<'config, S> {
    pub fn new(config: &'config Config) -> Self {
        Self {
            _config: config,
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<S: Service> BackendBuilder<S> for Builder<'_, S> {
    type Backend = TestBackend<S>;
    type CreationError = CreationError;

    fn create(self) -> Result<Self::Backend, Self::CreationError> {
        let session = Rc::new(Session::create().map_err(CreationError::CreateSession)?);
        let discovery = Discovery::new(session.clone());

        Ok(TestBackend {
            session,
            discovery,
            _phantom: core::marker::PhantomData,
        })
    }
}
