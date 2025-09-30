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

use iceoryx2::service::static_config::StaticConfig;
use zenoh::Session;

use crate::relays::{event, publish_subscribe};

/// Factory for creating relay builders.
///
/// The factory holds a reference to a Session and can be used in multiple builders.
pub struct Factory<'session> {
    /// Reference to the Zenoh session. The session must outlive the Factory.
    session: &'session Session,
}

impl<'session> Factory<'session> {
    pub fn new(session: &'session Session) -> Self {
        Factory { session }
    }
}

impl<'session> iceoryx2_tunnel_traits::RelayFactory for Factory<'session> {
    type PublishSubscribeBuilder<'config>
        = publish_subscribe::Builder<'config>
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
