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

use iceoryx2::service::{Service, local_threadsafe, static_config::StaticConfig};
use iceoryx2_services_tunnel_backend::traits::{PublishSubscribeRelay, RelayBuilder};
use iceoryx2_services_tunnel_backend::types::publish_subscribe::{LoanFn, Sample, SampleMut};
use iceoryx2_services_tunnel_backend::types::wake::WakeHandle;

use crate::typesupport::{self, TypeSupportRegistry};
use crate::{keys, payload, rcl};

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum CreationError {
    InvalidServiceName,
    InvalidTypeName,
    TypeSupport(typesupport::LoadError),
    Publisher(rcl::publisher::CreationError),
}

impl core::fmt::Display for CreationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "CreationError::{self:?}")
    }
}

impl core::error::Error for CreationError {}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum SendError {
    Publish(rcl::publisher::PublishError),
}

impl core::fmt::Display for SendError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "SendError::{self:?}")
    }
}

impl core::error::Error for SendError {}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum ReceiveError {}

impl core::fmt::Display for ReceiveError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ReceiveError::{self:?}")
    }
}

impl core::error::Error for ReceiveError {}

/// Relays publish-subscribe payloads between iceoryx2 and a ROS 2 topic.
#[derive(Debug)]
pub struct Relay<S: Service> {
    publisher: rcl::Publisher,
    _phantom: core::marker::PhantomData<S>,
}

impl<S: Service> PublishSubscribeRelay<S> for Relay<S> {
    type SendError = SendError;
    type ReceiveError = ReceiveError;

    fn send(&self, sample: Sample<S>) -> Result<(), Self::SendError> {
        self.publisher
            .publish(payload::as_bytes(sample.payload()))
            .map_err(SendError::Publish)
    }

    fn receive<LoanError>(
        &self,
        _loan: &mut LoanFn<'_, S, LoanError>,
    ) -> Result<Option<SampleMut<S>>, Self::ReceiveError> {
        Ok(None)
    }
}

/// Builder for publish-subscribe [`Relay`]s.
#[derive(Debug)]
pub struct Builder<'config, S: Service> {
    node: Rc<rcl::Node>,
    type_support: Rc<TypeSupportRegistry>,
    static_config: &'config StaticConfig,
    wake: Option<Arc<WakeHandle<local_threadsafe::Service>>>,
    _phantom: core::marker::PhantomData<S>,
}

impl<'config, S: Service> Builder<'config, S> {
    pub fn new(
        node: Rc<rcl::Node>,
        type_support: Rc<TypeSupportRegistry>,
        static_config: &'config StaticConfig,
        wake: Option<Arc<WakeHandle<local_threadsafe::Service>>>,
    ) -> Self {
        Self {
            node,
            type_support,
            static_config,
            wake,
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<S: Service> RelayBuilder for Builder<'_, S> {
    type CreationError = CreationError;
    type Relay = Relay<S>;

    // Endpoint creation simultaneously announces over the DDS SEDP.
    fn create(self) -> Result<Self::Relay, Self::CreationError> {
        let topic = keys::topic(self.static_config.name().as_str())
            .ok_or(CreationError::InvalidServiceName)?;
        // The payload type name carries the ROS 2 type name.
        let type_name = core::str::from_utf8(
            self.static_config
                .publish_subscribe()
                .message_type_details()
                .payload
                .type_name(),
        )
        .map_err(|_| CreationError::InvalidTypeName)?;

        let type_support = self
            .type_support
            .load(type_name)
            .map_err(CreationError::TypeSupport)?;
        let publisher = rcl::Publisher::create(&self.node, topic, type_support)
            .map_err(CreationError::Publisher)?;

        Ok(Relay {
            publisher,
            _phantom: core::marker::PhantomData,
        })
    }
}
