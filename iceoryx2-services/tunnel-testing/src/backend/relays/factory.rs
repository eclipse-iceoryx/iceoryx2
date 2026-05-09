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
use iceoryx2_services_tunnel_backend::traits::RelayFactory;

use crate::backend::{
    relays::{event, publish_subscribe},
    session::Session,
};

#[derive(Debug)]
pub struct Factory<S: Service> {
    session: Rc<Session>,
    _phantom: core::marker::PhantomData<S>,
}

impl<S: Service> Factory<S> {
    pub fn new(session: Rc<Session>) -> Self {
        Self {
            session,
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<S: Service> RelayFactory<S> for Factory<S> {
    type PublishSubscribeRelay = publish_subscribe::Relay<S>;
    type EventRelay = event::Relay<S>;

    type PublishSubscribeBuilder<'a>
        = publish_subscribe::Builder<'a, S>
    where
        Self: 'a;
    type EventBuilder<'a>
        = event::Builder<'a, S>
    where
        Self: 'a;

    fn publish_subscribe<'a>(
        &self,
        static_config: &'a iceoryx2::service::static_config::StaticConfig,
    ) -> Self::PublishSubscribeBuilder<'a>
    where
        Self: 'a,
    {
        publish_subscribe::Builder::new(self.session.clone(), static_config)
    }

    fn event<'a>(
        &self,
        static_config: &'a iceoryx2::service::static_config::StaticConfig,
    ) -> Self::EventBuilder<'a>
    where
        Self: 'a,
    {
        event::Builder::new(self.session.clone(), static_config)
    }
}
