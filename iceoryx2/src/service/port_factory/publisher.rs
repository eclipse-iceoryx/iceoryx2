// Copyright (c) 2023 Contributors to the Eclipse Foundation
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

//! # Example
//!
//! ```
//! use iceoryx2::prelude::*;
//! use iceoryx2::service::port_factory::publisher::UnableToDeliverStrategy;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let service_name = ServiceName::new("My/Funk/ServiceName")?;
//! let pubsub = zero_copy::Service::new(&service_name)
//!     .publish_subscribe()
//!     .open_or_create::<u64>()?;
//!
//! let publisher = pubsub.publisher()
//!                     .max_loaned_samples(6)
//!                     .unable_to_deliver_strategy(UnableToDeliverStrategy::DiscardSample)
//!                     .create()?;
//!
//! # Ok(())
//! # }
//! ```
use std::fmt::Debug;

use iceoryx2_bb_log::fail;
use serde::{de::Visitor, Deserialize, Serialize};

use crate::{
    port::publisher::{Publisher, PublisherCreateError},
    service,
};

use super::publish_subscribe::PortFactory;

/// Defines the strategy the [`Publisher`] shall pursue in [`Publisher::send()`] or
/// [`Publisher::send_copy()`] when the buffer of a
/// [`crate::port::subscriber::Subscriber`] is full and the service does not overflow.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum UnableToDeliverStrategy {
    /// Blocks until the [`crate::port::subscriber::Subscriber`] has consumed the
    /// [`crate::sample::Sample`] from the buffer and there is space again
    Block,
    /// Do not deliver the [`crate::sample::Sample`].
    DiscardSample,
}

impl Serialize for UnableToDeliverStrategy {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&std::format!("{:?}", self))
    }
}

struct UnableToDeliverStrategyVisitor;

impl<'de> Visitor<'de> for UnableToDeliverStrategyVisitor {
    type Value = UnableToDeliverStrategy;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a string containing either 'block' or 'discard_sample'")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match v {
            "block" => Ok(UnableToDeliverStrategy::Block),
            "discard_sample" => Ok(UnableToDeliverStrategy::DiscardSample),
            v => Err(E::custom(format!(
                "Invalid UnableToDeliverStrategy provided: \"{:?}\".",
                v
            ))),
        }
    }
}

impl<'de> Deserialize<'de> for UnableToDeliverStrategy {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(UnableToDeliverStrategyVisitor)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LocalPublisherConfig {
    pub(crate) max_loaned_samples: usize,
    pub(crate) unable_to_deliver_strategy: UnableToDeliverStrategy,
}

/// Factory to create a new [`Publisher`] port/endpoint for
/// [`MessagingPattern::PublishSubscribe`](crate::service::messaging_pattern::MessagingPattern::PublishSubscribe) based
/// communication.
#[derive(Debug)]
pub struct PortFactoryPublisher<
    'factory,
    'config,
    Service: service::Details<'config>,
    MessageType: Debug,
> {
    config: LocalPublisherConfig,
    pub(crate) factory: &'factory PortFactory<'config, Service, MessageType>,
}

impl<'factory, 'config, Service: service::Details<'config>, MessageType: Debug>
    PortFactoryPublisher<'factory, 'config, Service, MessageType>
{
    pub(crate) fn new(factory: &'factory PortFactory<'config, Service, MessageType>) -> Self {
        Self {
            config: LocalPublisherConfig {
                max_loaned_samples: factory
                    .service
                    .state()
                    .global_config
                    .defaults
                    .publish_subscribe
                    .publisher_max_loaned_samples,
                unable_to_deliver_strategy: factory
                    .service
                    .state()
                    .global_config
                    .defaults
                    .publish_subscribe
                    .unable_to_deliver_strategy,
            },
            factory,
        }
    }

    /// Defines how many [`crate::sample_mut::SampleMut`] the [`Publisher`] can loan with
    /// [`Publisher::loan()`] or [`Publisher::loan_uninit()`] in parallel.
    pub fn max_loaned_samples(mut self, value: usize) -> Self {
        self.config.max_loaned_samples = value;
        self
    }

    /// Sets the [`UnableToDeliverStrategy`].
    pub fn unable_to_deliver_strategy(mut self, value: UnableToDeliverStrategy) -> Self {
        self.config.unable_to_deliver_strategy = value;
        self
    }

    /// Creates a new [`Publisher`] or returns a [`PublisherCreateError`] on failure.
    pub fn create(
        self,
    ) -> Result<Publisher<'factory, 'config, Service, MessageType>, PublisherCreateError> {
        Ok(
            fail!(from self, when Publisher::new(&self.factory.service, self.factory.service.state().static_config.publish_subscribe(), &self.config),
                "Failed to create new Publisher port."),
        )
    }
}
