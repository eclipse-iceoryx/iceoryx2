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

use iceoryx2::service::Service;
use iceoryx2_bb_log::{fail, trace};
use iceoryx2_tunnel_backend::traits::Backend;
use zenoh::{Config, Session, Wait};

use crate::{
    discovery::Discovery,
    relays::{event, publish_subscribe, Factory},
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
    _phantom: core::marker::PhantomData<S>,
}

impl<S: Service> Backend<S> for ZenohBackend<S> {
    type Config = Config;
    type CreationError = CreationError;
    type Discovery = Discovery;

    type PublishSubscribeRelay = publish_subscribe::Relay<S>;
    type EventRelay = event::Relay<S>;

    type RelayFactory<'a>
        = Factory<'a, S>
    where
        Self: 'a;

    fn create(config: &Self::Config) -> Result<Self, Self::CreationError> {
        let origin = "ZenohBackend::create";

        trace!(
            from origin,
            "Initializing Zenoh backend"
        );

        let session = zenoh::open(config.clone()).wait();
        let session = fail!(
            from origin,
            when session,
            with Self::CreationError::Session,
            "Failed to create zenoh session"
        );

        let discovery = Discovery::create(&session);
        let discovery = fail!(
            from origin,
            when discovery,
            with CreationError::Discovery,
            "Failed to create zenoh discovery"
        );

        Ok(Self {
            session,
            discovery,
            _phantom: core::marker::PhantomData,
        })
    }

    fn relay_builder(&self) -> Self::RelayFactory<'_> {
        Self::RelayFactory::new(&self.session)
    }

    fn discovery(&self) -> &impl iceoryx2_tunnel_backend::traits::Discovery {
        &self.discovery
    }
}
