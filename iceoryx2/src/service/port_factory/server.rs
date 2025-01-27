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

use core::fmt::Debug;

use iceoryx2_bb_log::fail;

use crate::{port::server::Server, service};

use super::request_response::PortFactory;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum ServerCreateError {}

impl core::fmt::Display for ServerCreateError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        std::write!(f, "ServerCreateError::{:?}", self)
    }
}

impl std::error::Error for ServerCreateError {}

#[derive(Debug)]
pub struct PortFactoryServer<
    'factory,
    Service: service::Service,
    RequestPayload: Debug,
    RequestHeader: Debug,
    ResponsePayload: Debug,
    ResponseHeader: Debug,
> {
    factory: &'factory PortFactory<
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
        Self { factory }
    }

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
