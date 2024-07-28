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
//! ## Typed API
//!
//! ```
//! use iceoryx2::prelude::*;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let node = NodeBuilder::new().create::<ipc::Service>()?;
//! let pubsub = node.service_builder(&"My/Funk/ServiceName".try_into()?)
//!     .publish_subscribe::<u64>()
//!     .open_or_create()?;
//!
//! let publisher = pubsub.publisher_builder()
//!                     .max_loaned_samples(6)
//!                     .unable_to_deliver_strategy(UnableToDeliverStrategy::DiscardSample)
//!                     .create()?;
//!
//! # Ok(())
//! # }
//! ```
//!
//! ## Slice API
//!
//! ```
//! use iceoryx2::prelude::*;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let node = NodeBuilder::new().create::<ipc::Service>()?;
//! let pubsub = node.service_builder(&"My/Funk/ServiceName".try_into()?)
//!     .publish_subscribe::<[u64]>()
//!     .open_or_create()?;
//!
//! let publisher = pubsub.publisher_builder()
//!                     // allows to call Publisher::loan_slice() with up to 128 elements
//!                     .max_slice_len(128)
//!                     .create()?;
//!
//! let sample = publisher.loan_slice(50)?;
//!
//! # Ok(())
//! # }
//! ```

use std::fmt::Debug;

use iceoryx2_bb_log::fail;
use serde::{de::Visitor, Deserialize, Serialize};

use super::publish_subscribe::PortFactory;
use crate::{
    port::{
        port_identifiers::{UniquePublisherId, UniqueSubscriberId},
        publisher::Publisher,
        publisher::PublisherCreateError,
        DegrationAction, DegrationCallback,
    },
    service,
};

/// Defines the strategy the [`Publisher`] shall pursue in
/// [`crate::sample_mut::SampleMut::send()`] or
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
        formatter.write_str("a string containing either 'Block' or 'DiscardSample'")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match v {
            "Block" => Ok(UnableToDeliverStrategy::Block),
            "DiscardSample" => Ok(UnableToDeliverStrategy::DiscardSample),
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

#[derive(Debug)]
pub(crate) struct LocalPublisherConfig {
    pub(crate) max_loaned_samples: usize,
    pub(crate) unable_to_deliver_strategy: UnableToDeliverStrategy,
    pub(crate) degration_callback: Option<DegrationCallback<'static>>,
    pub(crate) max_slice_len: usize,
}

/// Factory to create a new [`Publisher`] port/endpoint for
/// [`MessagingPattern::PublishSubscribe`](crate::service::messaging_pattern::MessagingPattern::PublishSubscribe) based
/// communication.
#[derive(Debug)]
pub struct PortFactoryPublisher<
    'factory,
    Service: service::Service,
    Payload: Debug + ?Sized,
    UserHeader: Debug,
> {
    config: LocalPublisherConfig,
    pub(crate) factory: &'factory PortFactory<Service, Payload, UserHeader>,
}

impl<'factory, Service: service::Service, Payload: Debug + ?Sized, UserHeader: Debug>
    PortFactoryPublisher<'factory, Service, Payload, UserHeader>
{
    pub(crate) fn new(factory: &'factory PortFactory<Service, Payload, UserHeader>) -> Self {
        Self {
            config: LocalPublisherConfig {
                degration_callback: None,
                max_slice_len: 1,
                max_loaned_samples: factory
                    .service
                    .__internal_state()
                    .shared_node
                    .config()
                    .defaults
                    .publish_subscribe
                    .publisher_max_loaned_samples,
                unable_to_deliver_strategy: factory
                    .service
                    .__internal_state()
                    .shared_node
                    .config()
                    .defaults
                    .publish_subscribe
                    .unable_to_deliver_strategy,
            },
            factory,
        }
    }

    /// Defines how many [`crate::sample_mut::SampleMut`] the [`Publisher`] can loan with
    /// [`Publisher::loan()`] or
    /// [`Publisher::loan_uninit()`] in parallel.
    pub fn max_loaned_samples(mut self, value: usize) -> Self {
        self.config.max_loaned_samples = value;
        self
    }

    /// Sets the [`UnableToDeliverStrategy`].
    pub fn unable_to_deliver_strategy(mut self, value: UnableToDeliverStrategy) -> Self {
        self.config.unable_to_deliver_strategy = value;
        self
    }

    /// Sets the [`DegrationCallback`] of the [`Publisher`]. Whenever a connection to a
    /// [`crate::port::subscriber::Subscriber`] is corrupted or it seems to be dead, this callback
    /// is called and depending on the returned [`DegrationAction`] measures will be taken.
    pub fn set_degration_callback<
        F: Fn(
                service::static_config::StaticConfig,
                UniquePublisherId,
                UniqueSubscriberId,
            ) -> DegrationAction
            + 'static,
    >(
        mut self,
        callback: Option<F>,
    ) -> Self {
        match callback {
            Some(c) => self.config.degration_callback = Some(DegrationCallback::new(c)),
            None => self.config.degration_callback = None,
        }

        self
    }

    /// Creates a new [`Publisher`] or returns a [`PublisherCreateError`] on failure.
    pub fn create(self) -> Result<Publisher<Service, Payload, UserHeader>, PublisherCreateError> {
        let origin = format!("{:?}", self);
        Ok(
            fail!(from origin, when Publisher::new(&self.factory.service, self.factory.service.__internal_state().static_config.publish_subscribe(), self.config),
                "Failed to create new Publisher port."),
        )
    }
}

impl<'factory, Service: service::Service, Payload: Debug, UserHeader: Debug>
    PortFactoryPublisher<'factory, Service, [Payload], UserHeader>
{
    /// Sets the maximum slice length that a user can allocate with
    /// [`Publisher::loan_slice()`] or [`Publisher::loan_slice_uninit()`].
    pub fn max_slice_len(mut self, value: usize) -> Self {
        self.config.max_slice_len = value;
        self
    }
}
