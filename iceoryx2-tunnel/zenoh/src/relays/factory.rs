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

use iceoryx2::service::{static_config::StaticConfig, Service};
use iceoryx2_tunnel_backend::traits::RelayFactory;
use zenoh::Session;

use crate::relays::{event, publish_subscribe};

/// Factory for creating relay builders.
///
/// The factory holds a reference to a Session and can be used in multiple builders.
pub struct Factory<'session, S: Service> {
    /// Reference to the Zenoh session. The session must outlive the Factory.
    session: &'session Session,
    _phantom: core::marker::PhantomData<S>,
}

impl<'session, S: Service> Factory<'session, S> {
    pub fn new(session: &'session Session) -> Self {
        Factory {
            session,
            _phantom: core::marker::PhantomData::default(),
        }
    }
}

impl<'session, S: Service> RelayFactory<S> for Factory<'session, S> {
    type PublishSubscribeRelay = publish_subscribe::Relay<S>;
    type EventRelay = event::Relay;

    type PublishSubscribeBuilder<'config>
        = publish_subscribe::Builder<'config, S>
    where
        Self: 'config;

    type EventBuilder<'config>
        = event::Builder<'config>
    where
        Self: 'config;

    fn publish_subscribe<'config>(
        &self,
        static_config: &'config StaticConfig,
    ) -> Self::PublishSubscribeBuilder<'config>
    where
        Self: 'config,
    {
        publish_subscribe::Builder::new(self.session, static_config)
    }

    fn event<'config>(&self, static_config: &'config StaticConfig) -> Self::EventBuilder<'config>
    where
        Self: 'config,
    {
        event::Builder::new(self.session, static_config)
    }
}
