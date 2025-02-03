// Copyright (c) 2025 Contributors to the Eclipse Foundation
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
//! let node = NodeBuilder::new().create::<ipc::Service>()?;
//! let request_response = node.service_builder(&"My/Funk/ServiceName".try_into()?)
//!     .request_response::<u64, u64>()
//!     .open_or_create()?;
//!
//! let client = request_response.client_builder()
//!                     .max_loaned_requests(6)
//!                     .unable_to_deliver_strategy(UnableToDeliverStrategy::DiscardSample)
//!                     .create()?;
//!
//! # Ok(())
//! # }
//! ```

use super::request_response::PortFactory;
use crate::{
    port::{client::Client, DegrationAction, DegrationCallback},
    prelude::UnableToDeliverStrategy,
    service,
};
use core::fmt::Debug;
use iceoryx2_bb_log::fail;

/// Defines a failure that can occur when a [`Client`] is created with
/// [`crate::service::port_factory::client::PortFactoryClient`].
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum ClientCreateError {
    /// The datasegment in which the payload of the [`Client`] is stored, could not be created.
    UnableToCreateDataSegment,
    /// The maximum amount of [`Client`]s that can connect to a
    /// [`Service`](crate::service::Service) is
    /// defined in [`crate::config::Config`]. When this is exceeded no more [`Client`]s
    /// can be created for a specific [`Service`](crate::service::Service).
    ExceedsMaxSupportedClients,
}

impl core::fmt::Display for ClientCreateError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        std::write!(f, "ClientCreateError::{:?}", self)
    }
}

impl core::error::Error for ClientCreateError {}

/// Factory to create a new [`Client`] port/endpoint for
/// [`MessagingPattern::RequestResponse`](crate::service::messaging_pattern::MessagingPattern::RequestResponse)
/// based communication.
#[derive(Debug)]
pub struct PortFactoryClient<
    'factory,
    Service: service::Service,
    RequestPayload: Debug,
    RequestHeader: Debug,
    ResponsePayload: Debug,
    ResponseHeader: Debug,
> {
    pub(crate) factory: &'factory PortFactory<
        Service,
        RequestPayload,
        RequestHeader,
        ResponsePayload,
        ResponseHeader,
    >,
    pub(crate) max_loaned_requests: usize,
    pub(crate) unable_to_deliver_strategy: UnableToDeliverStrategy,
    pub(crate) degration_callback: Option<DegrationCallback<'static>>,
}

impl<
        'factory,
        Service: service::Service,
        RequestPayload: Debug,
        RequestHeader: Debug,
        ResponsePayload: Debug,
        ResponseHeader: Debug,
    >
    PortFactoryClient<
        'factory,
        Service,
        RequestPayload,
        RequestHeader,
        ResponsePayload,
        ResponseHeader,
    >
{
    pub(crate) fn new(
        factory: &'factory PortFactory<
            Service,
            RequestPayload,
            RequestHeader,
            ResponsePayload,
            ResponseHeader,
        >,
    ) -> Self {
        let defs = &factory
            .service
            .__internal_state()
            .shared_node
            .config()
            .defaults
            .request_response;

        Self {
            factory,
            unable_to_deliver_strategy: defs.client_unable_to_deliver_strategy,
            max_loaned_requests: defs.client_max_loaned_requests,
        }
    }

    /// Sets the [`UnableToDeliverStrategy`].
    pub fn unable_to_deliver_strategy(mut self, value: UnableToDeliverStrategy) -> Self {
        self.unable_to_deliver_strategy = value;
        self
    }

    /// Defines how many requests the [`Client`] can loan in parallel.
    pub fn max_loaned_requests(mut self, value: usize) -> Self {
        self.max_loaned_requests = value;
        self
    }

    pub fn set_degration_callback<
        F: Fn(&service::static_config::StaticConfig, u128, u128) -> DegrationAction + 'static,
    >(
        mut self,
        callback: Option<F>,
    ) -> Self {
        match callback {
            Some(c) => self.degration_callback = Some(DegrationCallback::new(c)),
            None => self.degration_callback = None,
        }

        self
    }

    /// Creates a new [`Client`] or returns a [`ClientCreateError`] on failure.
    pub fn create(
        self,
    ) -> Result<
        Client<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>,
        ClientCreateError,
    > {
        let origin = format!("{:?}", self);
        Ok(fail!(from origin,
              when Client::new(self),
              "Failed to create new Client port."))
    }
}
