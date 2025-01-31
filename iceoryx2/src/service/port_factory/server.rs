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
use crate::{port::server::Server, service};
use core::fmt::Debug;
use iceoryx2_bb_log::fail;

/// Defines a failure that can occur when a [`Server`] is created with
/// [`crate::service::port_factory::server::PortFactoryServer`].
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum ServerCreateError {}

impl core::fmt::Display for ServerCreateError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        std::write!(f, "ServerCreateError::{:?}", self)
    }
}

impl std::error::Error for ServerCreateError {}

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
    _factory: &'factory PortFactory<
        Service,
        RequestPayload,
        RequestHeader,
        ResponsePayload,
        ResponseHeader,
    >,
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
        Self { _factory: factory }
    }

    /// Creates a new [`Server`] or returns a [`ServerCreateError`] on failure.
    pub fn create(
        self,
    ) -> Result<
        Server<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>,
        ServerCreateError,
    > {
        Ok(fail!(from self,
              when Server::new(),
              "Failed to create new Server port."))
    }
}
