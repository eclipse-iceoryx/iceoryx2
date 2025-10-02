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

use iceoryx2_bb_log::fail;
use zenoh::{Config, Session, Wait};

use crate::{
    discovery::Discovery,
    relays::{event, publish_subscribe, Factory},
};

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum CreationError {
    FailedToCreateSession,
    FailedToCreateDiscovery,
}

#[derive(Debug)]
pub struct Transport {
    session: Session,
    discovery: Discovery,
}

impl iceoryx2_tunnel_traits::Transport for Transport {
    type Config = Config;
    type CreationError = CreationError;
    type Discovery = Discovery;

    type PublishSubscribeRelay = publish_subscribe::Relay;
    type EventRelay = event::Relay;

    type RelayFactory<'a>
        = Factory<'a>
    where
        Self: 'a;

    fn create(config: &Self::Config) -> Result<Self, Self::CreationError> {
        let session = zenoh::open(config.clone()).wait();
        let session = fail!(
            from "ZenohTransport::create()",
            when session,
            with Self::CreationError::FailedToCreateSession,
            "Failed to create zenoh session"
        );

        let discovery = Discovery::create(&session);
        let discovery = fail!(
            from "Tunnel::create()",
            when discovery,
            with CreationError::FailedToCreateDiscovery,
            "Failed to create zenoh discovery"
        );

        Ok(Self { session, discovery })
    }

    fn relay_builder(&self) -> Self::RelayFactory<'_> {
        Self::RelayFactory::new(&self.session)
    }

    fn discovery(&self) -> &impl iceoryx2_tunnel_traits::Discovery {
        &self.discovery
    }
}
