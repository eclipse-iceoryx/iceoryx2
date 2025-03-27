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
//! let server = request_response
//!                     .server_builder()
//!                     .create()?;
//!
//! # Ok(())
//! # }
//! ```

use super::request_response::PortFactory;
use crate::{
    port::{server::Server, DegradationAction, DegradationCallback},
    prelude::UnableToDeliverStrategy,
    service,
};
use core::fmt::Debug;
use iceoryx2_bb_log::fail;

/// Defines a failure that can occur when a [`Server`] is created with
/// [`crate::service::port_factory::server::PortFactoryServer`].
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum ServerCreateError {
    /// The maximum amount of [`Server`]s supported by the [`Service`](crate::service::Service)
    /// is already connected.
    ExceedsMaxSupportedServers,
}

impl core::fmt::Display for ServerCreateError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        std::write!(f, "ServerCreateError::{:?}", self)
    }
}

impl core::error::Error for ServerCreateError {}

/// Factory to create a new [`Server`] port/endpoint for
/// [`MessagingPattern::RequestResponse`](crate::service::messaging_pattern::MessagingPattern::RequestResponse)
/// based communication.
#[derive(Debug)]
pub struct PortFactoryServer<
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

    pub(crate) max_loaned_responses_per_request: usize,
    pub(crate) unable_to_deliver_strategy: UnableToDeliverStrategy,
    pub(crate) degradation_callback: Option<DegradationCallback<'static>>,
}

impl<
        'factory,
        Service: service::Service,
        RequestPayload: Debug,
        RequestHeader: Debug,
        ResponsePayload: Debug,
        ResponseHeader: Debug,
    >
    PortFactoryServer<
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
            max_loaned_responses_per_request: defs.server_max_loaned_responses_per_request,
            unable_to_deliver_strategy: defs.server_unable_to_deliver_strategy,
            degradation_callback: None,
        }
    }

    /// Sets the [`UnableToDeliverStrategy`] which defines how the [`Server`] shall behave
    /// when a [`Client`](crate::port::client::Client) cannot receive a
    /// [`Response`](crate::response::Response) since
    /// its internal buffer is full.
    pub fn unable_to_deliver_strategy(mut self, value: UnableToDeliverStrategy) -> Self {
        self.unable_to_deliver_strategy = value;
        self
    }

    /// Defines the maximum number of [`ResponseMut`](crate::response_mut::ResponseMut) that
    /// the [`Server`] can loan in parallel per
    /// [`ActiveRequest`](crate::active_request::ActiveRequest).
    pub fn max_loaned_responses_per_request(mut self, value: usize) -> Self {
        self.max_loaned_responses_per_request = value;
        self
    }

    /// Sets the [`DegradationCallback`] of the [`Server`]. Whenever a connection to a
    /// [`Client`](crate::port::client::Client) is corrupted or it seems to be dead, this callback
    /// is called and depending on the returned [`DegradationAction`] measures will be taken.
    pub fn set_degradation_callback<
        F: Fn(&service::static_config::StaticConfig, u128, u128) -> DegradationAction + 'static,
    >(
        mut self,
        callback: Option<F>,
    ) -> Self {
        match callback {
            Some(c) => self.degradation_callback = Some(DegradationCallback::new(c)),
            None => self.degradation_callback = None,
        }

        self
    }

    /// Creates a new [`Server`] or returns a [`ServerCreateError`] on failure.
    pub fn create(
        self,
    ) -> Result<
        Server<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>,
        ServerCreateError,
    > {
        let origin = format!("{:?}", self);
        Ok(fail!(from origin,
              when Server::new(self),
              "Failed to create new Server port."))
    }
}
