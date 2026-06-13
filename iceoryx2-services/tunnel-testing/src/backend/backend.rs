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
use iceoryx2_services_tunnel_backend::traits::{Backend, BackendBuilder, Passthrough, Translator};

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
pub struct TestBackend<S: Service, T: Translator = Passthrough> {
    session: Rc<Session>,
    discovery: Discovery,
    #[allow(dead_code)]
    translator: T,
    _phantom: core::marker::PhantomData<S>,
}

impl<S: Service, T: Translator> Backend<S> for TestBackend<S, T> {
    type Config = Config;
    type Translator = T;
    type CreationError = CreationError;
    type Builder<'config>
        = Builder<'config, S, T>
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
pub struct Builder<'config, S: Service, T: Translator = Passthrough> {
    _config: &'config Config,
    translator: T,
    _phantom: core::marker::PhantomData<S>,
}

impl<'config, S: Service, T: Translator> Builder<'config, S, T> {
    pub fn new(config: &'config Config) -> Self {
        Self {
            _config: config,
            translator: T::default(),
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<S: Service, T: Translator> BackendBuilder<S> for Builder<'_, S, T> {
    type Backend = TestBackend<S, T>;
    type CreationError = CreationError;

    fn translator(mut self, translator: T) -> Self {
        self.translator = translator;
        self
    }

    fn create(self) -> Result<Self::Backend, Self::CreationError> {
        let session = Rc::new(Session::create().map_err(CreationError::CreateSession)?);
        let discovery = Discovery::new(session.clone());

        Ok(TestBackend {
            session,
            discovery,
            translator: self.translator,
            _phantom: core::marker::PhantomData,
        })
    }
}
