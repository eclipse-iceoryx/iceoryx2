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
//! # fn main() -> Result<(), Box<dyn core::error::Error>> {
//! let node = NodeBuilder::new().create::<ipc::Service>()?;
//! let pubsub = node.service_builder(&"My/Funk/ServiceName".try_into()?)
//!     .publish_subscribe::<u64>()
//!     .open_or_create()?;
//!
//! let subscriber = pubsub.subscriber_builder()
//!                     .create()?;
//!
//! # Ok(())
//! # }
//! ```

use core::fmt::Debug;

use alloc::format;

use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use iceoryx2_log::fail;

use crate::{
    port::{
        DegradationAction, DegradationFn, DegradationHandler,
        port_name::PortName,
        subscriber::{Subscriber, SubscriberCreateError},
    },
    service,
};

use super::publish_subscribe::PortFactory;

#[derive(Debug)]
pub(crate) struct SubscriberConfig {
    pub(crate) buffer_size: Option<usize>,
    pub(crate) history_request: Option<usize>,
    pub(crate) degradation_handler: DegradationHandler<'static>,
    pub(crate) port_name: PortName,
}

/// Factory to create a new [`Subscriber`] port/endpoint for
/// [`MessagingPattern::PublishSubscribe`](crate::service::messaging_pattern::MessagingPattern::PublishSubscribe) based
/// communication.
#[derive(Debug)]
pub struct PortFactorySubscriber<
    'factory,
    Service: service::Service,
    PayloadType: Debug + ?Sized,
    UserHeader: Debug + ZeroCopySend,
> {
    config: SubscriberConfig,
    pub(crate) factory: &'factory PortFactory<Service, PayloadType, UserHeader>,
}

unsafe impl<Service: service::Service, Payload: Debug + ?Sized, UserHeader: Debug + ZeroCopySend>
    Send for PortFactorySubscriber<'_, Service, Payload, UserHeader>
{
}

impl<
    'factory,
    Service: service::Service,
    PayloadType: Debug + ?Sized,
    UserHeader: Debug + ZeroCopySend,
> PortFactorySubscriber<'factory, Service, PayloadType, UserHeader>
{
    #[doc(hidden)]
    /// # Safety
    ///
    ///   * does not clone the degradation callback
    pub unsafe fn __internal_partial_clone(&self) -> Self {
        Self {
            config: SubscriberConfig {
                buffer_size: self.config.buffer_size,
                history_request: self.config.history_request,
                degradation_handler: DegradationHandler::new_with(DegradationAction::Warn),
                port_name: self.config.port_name,
            },
            factory: self.factory,
        }
    }

    pub(crate) fn new(factory: &'factory PortFactory<Service, PayloadType, UserHeader>) -> Self {
        Self {
            config: SubscriberConfig {
                buffer_size: None,
                history_request: None,
                degradation_handler: DegradationHandler::new_with(DegradationAction::Warn),
                port_name: PortName::new_empty(),
            },
            factory,
        }
    }

    /// Defines the buffer size of the [`Subscriber`]. Smallest possible value is `1`.
    pub fn buffer_size(mut self, value: usize) -> Self {
        self.config.buffer_size = Some(value.max(1));
        self
    }

    /// Defines the amount of requested history samples. By default the value defined with the
    /// service's `history_size` is used.
    pub fn history_request(mut self, value: usize) -> Self {
        self.config.history_request = Some(value);
        self
    }

    /// Sets the [`DegradationHandler`] of the [`Subscriber`]. Whenever a connection to a
    /// [`crate::port::subscriber::Subscriber`] is corrupted or it seems to be dead, this handler
    /// is called and depending on the returned [`DegradationAction`] measures will be taken.
    pub fn set_degradation_handler<F: DegradationFn + 'static>(mut self, handler: F) -> Self {
        self.config.degradation_handler = DegradationHandler::new(handler);

        self
    }

    /// Sets the [`PortName`] of the  [`Subscriber`].
    pub fn name(mut self, name: &PortName) -> Self {
        self.config.port_name = *name;
        self
    }

    /// Creates a new [`Subscriber`] or returns a [`SubscriberCreateError`] on failure.
    pub fn create(
        self,
    ) -> Result<Subscriber<Service, PayloadType, UserHeader>, SubscriberCreateError> {
        let origin = format!("{self:?}");
        Ok(
            fail!(from origin, when Subscriber::new(self.factory.service.clone(), self.factory.service.static_config().publish_subscribe(), self.config),
                "Failed to create new Subscriber port."),
        )
    }
}
