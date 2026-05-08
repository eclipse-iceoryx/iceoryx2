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

use iceoryx2::service::Service;
use iceoryx2_services_tunnel_backend::traits::Backend;

use crate::backend::{
    discovery::Discovery,
    relays::{self, factory::Factory},
};

#[derive(Debug, Clone)]
pub struct Config;

impl Default for Config {
    fn default() -> Self {
        Self {}
    }
}

#[derive(Debug)]
pub enum CreationError {}

impl core::fmt::Display for CreationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "CreationError::{self:?}")
    }
}

impl core::error::Error for CreationError {}

#[derive(Debug)]
pub struct TestBackend<S: Service> {
    discovery: Discovery,
    _phantom: core::marker::PhantomData<S>,
}

impl<S: Service> Backend<S> for TestBackend<S> {
    type Config = Config;
    type CreationError = CreationError;

    type Discovery = Discovery;

    type PublishSubscribeRelay = relays::publish_subscribe::Relay<S>;
    type EventRelay = relays::event::Relay<S>;

    type RelayFactory<'a>
        = Factory<S>
    where
        Self: 'a;

    fn create(config: &Self::Config) -> Result<Self, Self::CreationError> {
        let discovery = Discovery::new();

        Ok(Self {
            discovery,
            _phantom: core::marker::PhantomData,
        })
    }

    fn discovery(&self) -> &impl iceoryx2_services_tunnel_backend::traits::Discovery {
        &self.discovery
    }

    fn relay_builder(&self) -> Self::RelayFactory<'_> {
        Factory::new()
    }
}
