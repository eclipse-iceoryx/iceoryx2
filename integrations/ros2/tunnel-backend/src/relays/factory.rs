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

use std::rc::Rc;
use std::sync::Arc;

use iceoryx2::service::{Service, local_threadsafe};
use iceoryx2_services_tunnel_backend::traits::Mapping;
use iceoryx2_services_tunnel_backend::types::service_description::ServiceDescription;
use iceoryx2_services_tunnel_backend::{traits::RelayFactory, types::wake::WakeHandle};

use crate::mapping::TopicDescription;
use crate::rcl::RclNode;
use crate::relays::{event, publish_subscribe};
use crate::typesupport::TypeSupportRegistry;

/// Factory for creating relay builders.
#[derive(Debug)]
pub struct Factory<'a, S: Service, M: Mapping<EndpointDescription = TopicDescription>> {
    node: Rc<RclNode>,
    type_registry: &'a TypeSupportRegistry,
    mapping: &'a M,
    wake: Option<Arc<WakeHandle<local_threadsafe::Service>>>,
    _phantom: core::marker::PhantomData<S>,
}

impl<'a, S: Service, M: Mapping<EndpointDescription = TopicDescription>> Factory<'a, S, M> {
    pub fn new(
        node: Rc<RclNode>,
        type_registry: &'a TypeSupportRegistry,
        mapping: &'a M,
        wake: Option<Arc<WakeHandle<local_threadsafe::Service>>>,
    ) -> Self {
        Factory {
            node,
            type_registry,
            mapping,
            wake,
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<S: Service, M: Mapping<EndpointDescription = TopicDescription>> RelayFactory<S>
    for Factory<'_, S, M>
{
    type PublishSubscribeRelay = publish_subscribe::Relay<S>;
    type EventRelay = event::Relay<S>;

    type PublishSubscribeBuilder<'a>
        = publish_subscribe::Builder<'a, S, M>
    where
        Self: 'a;

    type EventBuilder<'a>
        = event::Builder<'a, S>
    where
        Self: 'a;

    fn publish_subscribe<'a>(
        &self,
        description: &'a ServiceDescription,
    ) -> Self::PublishSubscribeBuilder<'a>
    where
        Self: 'a,
    {
        publish_subscribe::Builder::new(
            description,
            Rc::clone(&self.node),
            self.type_registry,
            self.mapping,
            self.wake.clone(),
        )
    }

    fn event<'a>(&self, description: &'a ServiceDescription) -> Self::EventBuilder<'a>
    where
        Self: 'a,
    {
        event::Builder::new(description, self.wake.clone())
    }
}
