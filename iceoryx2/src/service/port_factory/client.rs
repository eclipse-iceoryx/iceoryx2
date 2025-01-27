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

use crate::{port::client::Client, service};

use super::request_response::PortFactory;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum ClientCreateError {}

impl core::fmt::Display for ClientCreateError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        std::write!(f, "ClientCreateError::{:?}", self)
    }
}

impl std::error::Error for ClientCreateError {}

#[derive(Debug)]
pub struct PortFactoryClient<
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
        Self { factory }
    }

    pub fn create(
        self,
    ) -> Result<
        Client<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>,
        ClientCreateError,
    > {
        Ok(fail!(from self,
              when Client::new(),
              "Failed to create new Client port."))
    }
}
