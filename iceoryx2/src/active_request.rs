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

use core::{fmt::Debug, marker::PhantomData};

use crate::{port::ReceiveError, request_mut::RequestMut, response::Response, service};

pub struct ActiveRequest<
    Service: crate::service::Service,
    RequestPayload: Debug,
    RequestHeader: Debug,
    ResponsePayload: Debug,
    ResponseHeader: Debug,
> {
    pub(crate) request:
        RequestMut<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>,
    pub(crate) number_of_server_connections: usize,
    pub(crate) _service: PhantomData<Service>,
    pub(crate) _response_payload: PhantomData<ResponsePayload>,
    pub(crate) _response_header: PhantomData<ResponseHeader>,
}

impl<
        Service: crate::service::Service,
        RequestPayload: Debug,
        RequestHeader: Debug,
        ResponsePayload: Debug,
        ResponseHeader: Debug,
    > Debug
    for ActiveRequest<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ActiveRequest<{}, {}, {}, {}, {}> {{ }}",
            core::any::type_name::<Service>(),
            core::any::type_name::<RequestPayload>(),
            core::any::type_name::<RequestHeader>(),
            core::any::type_name::<ResponsePayload>(),
            core::any::type_name::<ResponseHeader>()
        )
    }
}

impl<
        Service: crate::service::Service,
        RequestPayload: Debug,
        RequestHeader: Debug,
        ResponsePayload: Debug,
        ResponseHeader: Debug,
    > ActiveRequest<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>
{
    pub fn header(&self) -> &service::header::request_response::RequestHeader {
        self.request.header()
    }

    pub fn user_header(&self) -> &RequestHeader {
        self.request.user_header()
    }

    pub fn payload(&self) -> &RequestPayload {
        self.request.payload()
    }

    pub fn number_of_server_connections(&self) -> usize {
        self.number_of_server_connections
    }

    pub fn receive(
        &self,
    ) -> Result<Option<Response<Service, ResponsePayload, ResponseHeader>>, ReceiveError> {
        todo!()
    }
}
