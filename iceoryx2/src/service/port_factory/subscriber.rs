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
//! let subscriber = pubsub.subscriber()
//!                     .create()?;
//!
//! # Ok(())
//! # }
//! ```

use std::fmt::Debug;

use iceoryx2_bb_log::fail;

use crate::{
    port::subscriber::{Subscriber, SubscriberCreateError},
    service,
};

use super::publish_subscribe::PortFactory;

/// Factory to create a new [`Subscriber`] port/endpoint for
/// [`MessagingPattern::PublishSubscribe`](crate::service::messaging_pattern::MessagingPattern::PublishSubscribe) based
/// communication.
#[derive(Debug)]
pub struct PortFactorySubscriber<
    'factory,
    'config,
    Service: service::Details<'config>,
    MessageType: Debug,
> {
    pub(crate) factory: &'factory PortFactory<'config, Service, MessageType>,
}

impl<'factory, 'config, Service: service::Details<'config>, MessageType: Debug>
    PortFactorySubscriber<'factory, 'config, Service, MessageType>
{
    /// Creates a new [`Subscriber`] or returns a [`SubscriberCreateError`] on failure.
    pub fn create(
        &self,
    ) -> Result<Subscriber<'factory, 'config, Service, MessageType>, SubscriberCreateError> {
        Ok(
            fail!(from self, when Subscriber::new(&self.factory.service, self.factory.service.state().static_config.publish_subscribe()),
                "Failed to create new Subscriber port."),
        )
    }
}
