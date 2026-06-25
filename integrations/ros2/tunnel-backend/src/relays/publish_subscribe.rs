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
use iceoryx2_log::fail;
use iceoryx2_services_tunnel_backend::traits::{PublishSubscribeRelay, RelayBuilder};
use iceoryx2_services_tunnel_backend::types::publish_subscribe::{LoanFn, Sample, SampleMut};
use iceoryx2_services_tunnel_backend::types::wake::WakeHandle;

use crate::typesupport::TypeSupportRegistry;
use crate::{keys, payload, rcl};

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum CreationError {
    InvalidServiceName,
    InvalidTypeName,
    InvalidTopic,
    TypeSupport,
    Publisher,
}

impl core::fmt::Display for CreationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "CreationError::{self:?}")
    }
}

impl core::error::Error for CreationError {}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum SendError {
    Publish,
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
        let origin = "publish_subscribe::Relay::send";

        fail!(from origin,
            when self.publisher.publish(payload::as_bytes(sample.payload())),
            with SendError::Publish,
            "Failed to relay sample to ROS 2"
        );

        Ok(())
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
pub struct Builder<'a, S: Service> {
    node: rcl::NodeHandle,
    type_registry: &'a TypeSupportRegistry,
    static_config: &'a StaticConfig,
    wake: Option<Arc<WakeHandle<local_threadsafe::Service>>>,
    _phantom: core::marker::PhantomData<S>,
}

impl<'a, S: Service> Builder<'a, S> {
    pub fn new(
        node: rcl::NodeHandle,
        type_registry: &'a TypeSupportRegistry,
        static_config: &'a StaticConfig,
        wake: Option<Arc<WakeHandle<local_threadsafe::Service>>>,
    ) -> Self {
        Self {
            node,
            type_registry,
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
        let origin = "publish_subscribe::Relay::create";

        let topic = fail!(from origin,
            when keys::topic(self.static_config.name().as_str()).ok_or(CreationError::InvalidServiceName),
            "Failed to map service name to a ROS 2 topic"
        );

        // The payload type name carries the ROS 2 type name.
        let type_name = fail!(from origin,
            when core::str::from_utf8(
                self.static_config
                    .publish_subscribe()
                    .message_type_details()
                    .payload
                    .type_name(),
            ),
            with CreationError::InvalidTypeName,
            "Failed to read the payload type name as UTF-8"
        );

        let type_support = fail!(from origin,
            when self.type_registry.load(type_name),
            with CreationError::TypeSupport,
            "Failed to load typesupport for '{}'",
            type_name
        );
        let topic_name = fail!(from origin,
            when rcl::TopicName::new(topic),
            with CreationError::InvalidTopic,
            "Invalid ROS 2 topic name '{}'",
            topic
        );
        let publisher = fail!(from origin,
            when self.node.publisher_builder(topic_name, type_support).create(),
            with CreationError::Publisher,
            "Failed to create ROS 2 publisher for topic '{}'",
            topic
        );

        Ok(Relay {
            publisher,
            _phantom: core::marker::PhantomData,
        })
    }
}
