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

use std::sync::Arc;

use iceoryx2::service::{Service, local_threadsafe, static_config::StaticConfig};
use iceoryx2_services_tunnel_backend::{traits::RelayFactory, types::wake::WakeHandle};

use crate::rcl;
use crate::relays::{event, publish_subscribe};
use crate::typesupport::TypeSupportRegistry;

/// Factory for creating relay builders.
#[derive(Debug)]
pub struct Factory<'a, S: Service> {
    node: rcl::NodeHandle,
    type_registry: &'a TypeSupportRegistry,
    /// Wake handle to be signaled by relays when new data arrives.
    /// `None` when the backend was constructed in polled mode.
    wake: Option<Arc<WakeHandle<local_threadsafe::Service>>>,
    _phantom: core::marker::PhantomData<S>,
}

impl<'a, S: Service> Factory<'a, S> {
    pub fn new(
        node: rcl::NodeHandle,
        type_registry: &'a TypeSupportRegistry,
        wake: Option<Arc<WakeHandle<local_threadsafe::Service>>>,
    ) -> Self {
        Factory {
            node,
            type_registry,
            wake,
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<S: Service> RelayFactory<S> for Factory<'_, S> {
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
        static_config: &'a StaticConfig,
    ) -> Self::PublishSubscribeBuilder<'a>
    where
        Self: 'a,
    {
        publish_subscribe::Builder::new(
            self.node.clone(),
            self.type_registry,
            static_config,
            self.wake.clone(),
        )
    }

    fn event<'a>(&self, static_config: &'a StaticConfig) -> Self::EventBuilder<'a>
    where
        Self: 'a,
    {
        event::Builder::new(static_config, self.wake.clone())
    }
}
