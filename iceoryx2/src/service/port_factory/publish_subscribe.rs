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
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let service_name = ServiceName::new("My/Funk/ServiceName")?;
//! let pubsub = zero_copy::Service::new(&service_name)
//!     .publish_subscribe()
//!     .open_or_create::<u64>()?;
//!
//! println!("{}", pubsub);
//!
//! let publisher = pubsub.publisher().create()?;
//! let subscriber = pubsub.subscriber().create()?;
//!
//! # Ok(())
//! # }
//! ```

use std::ops::Deref;
use std::{fmt::Debug, fmt::Display, fmt::Formatter, fmt::Result, marker::PhantomData};

use iceoryx2_cal::dynamic_storage::DynamicStorage;
use service::Service;

use crate::service::service_name::ServiceName;
use crate::service::{self, dynamic_config, static_config};

use super::{publisher::PortFactoryPublisher, subscriber::PortFactorySubscriber};

/// The factory for
/// [`MessagingPattern::PublishSubscribe`](crate::service::messaging_pattern::MessagingPattern::PublishSubscribe).
/// It can acquire dynamic and static service informations and create
/// [`crate::port::publisher::Publisher`]
/// or [`crate::port::subscriber::Subscriber`] ports.
#[derive(Debug)]
pub struct PortFactory<'config, Service: service::Details<'config>, MessageType: Debug> {
    pub(crate) service: Service,
    _phantom_message_type: PhantomData<MessageType>,
    _phantom_lifetime_b: PhantomData<&'config ()>,
}

unsafe impl<'config, Service: service::Details<'config>, MessageType: Debug> Send
    for PortFactory<'config, Service, MessageType>
{
}
unsafe impl<'config, Service: service::Details<'config>, MessageType: Debug> Sync
    for PortFactory<'config, Service, MessageType>
{
}

impl<'config, Service: service::Details<'config>, MessageType: Debug>
    PortFactory<'config, Service, MessageType>
{
    pub(crate) fn new(service: Service) -> Self {
        Self {
            service,
            _phantom_message_type: PhantomData,
            _phantom_lifetime_b: PhantomData,
        }
    }

    /// Returns the [`ServiceName`] of the service
    pub fn name(&self) -> &ServiceName {
        self.service.state().static_config.service_name()
    }

    /// Returns the uuid of the [`crate::service::Service`]
    pub fn uuid(&self) -> &str {
        self.service.state().static_config.uuid()
    }

    /// Returns the [`static_config::event::StaticConfig`] of the [`crate::service::Service`].
    /// Contains all settings that never change during the lifetime of the service.
    pub fn static_config(&self) -> &static_config::publish_subscribe::StaticConfig {
        self.service.state().static_config.publish_subscribe()
    }

    /// Returns the [`dynamic_config::event::DynamicConfig`] of the [`crate::service::Service`].
    /// Contains all dynamic settings, like the current participants etc..
    pub fn dynamic_config(&self) -> &dynamic_config::publish_subscribe::DynamicConfig {
        self.service
            .state()
            .dynamic_storage
            .get()
            .publish_subscribe()
    }

    /// Returns a [`PortFactorySubscriber`] to create a new
    /// [`crate::port::subscriber::Subscriber`] port.
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let service_name = ServiceName::new("My/Funk/ServiceName")?;
    /// let pubsub = zero_copy::Service::new(&service_name)
    ///     .publish_subscribe()
    ///     .open_or_create::<u64>()?;
    ///
    /// let subscriber = pubsub.subscriber().create()?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn subscriber<'a>(&'a self) -> PortFactorySubscriber<'a, 'config, Service, MessageType> {
        PortFactorySubscriber { factory: self }
    }

    /// Returns a [`PortFactoryPublisher`] to create a new
    /// [`crate::port::publisher::Publisher`] port.
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    /// use iceoryx2::service::port_factory::publisher::UnableToDeliverStrategy;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let service_name = ServiceName::new("My/Funk/ServiceName")?;
    /// let pubsub = zero_copy::Service::new(&service_name)
    ///     .publish_subscribe()
    ///     .open_or_create::<u64>()?;
    ///
    /// let publisher = pubsub.publisher()
    ///                     .max_loaned_samples(6)
    ///                     .unable_to_deliver_strategy(UnableToDeliverStrategy::DiscardSample)
    ///                     .create()?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn publisher<'a>(&'a self) -> PortFactoryPublisher<'a, 'config, Service, MessageType> {
        PortFactoryPublisher::new(self)
    }
}


impl<'config, S, M> Display for PortFactory<'config, S, M> where S: Service + service::Details<'config>, M: Debug {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_struct(&format!("{}", core::any::type_name::<S>()))
        .field("name", &String::from(self.name().deref()))
        .field("uuid", &self.uuid())
        .field("type name", &self.static_config().type_name())
        .field(
            "max publishers",
            &self.static_config().max_supported_publishers()
        )
        .field(
            "max subscribers",
            &self.static_config().max_supported_subscribers()
        )
        .field(
            "subscriber buffer size",
            &self.static_config().subscriber_max_buffer_size()
        )
        .field(
            "history size",
            &self.static_config().history_size()
        )
        .field(
            "subscriber max borrowed samples",
            &self.static_config().subscriber_max_borrowed_samples()
        )
        .field(
            "safe overflow",
            &self.static_config().has_safe_overflow()
        )
        .field(
            "number of active publishers",
            &self.dynamic_config().number_of_publishers()
        )
        .field(
            "number of active subscribers",
            &self.dynamic_config().number_of_subscribers()
        )
        .finish()
    }
}


#[cfg(test)]
mod tests {
    use crate::prelude::*;
    #[test]
    fn print_service() {
        let service_name = ServiceName::new("My/Funk/ServiceName").unwrap();
        let pubsub = zero_copy::Service::new(&service_name)
            .publish_subscribe()
            .open_or_create::<u64>().unwrap();
        println!("{}", pubsub);
        assert!(format!("{}", pubsub).contains("ServiceName"))
    }
}
