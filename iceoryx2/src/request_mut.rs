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
use std::sync::Arc;

use iceoryx2_cal::shm_allocator::PointerOffset;

use crate::{
    active_request::ActiveRequest,
    port::{client::ClientBackend, details::outgoing_connections::OutgoingConnections, SendError},
    raw_sample::RawSampleMut,
    service,
};

pub struct RequestMut<
    Service: crate::service::Service,
    RequestPayload: Debug,
    RequestHeader: Debug,
    ResponsePayload: Debug,
    ResponseHeader: Debug,
> {
    pub(crate) ptr: RawSampleMut<
        service::header::request_response::RequestHeader,
        RequestHeader,
        RequestPayload,
    >,
    pub(crate) sample_size: usize,
    pub(crate) offset_to_chunk: PointerOffset,
    pub(crate) client_backend: Arc<ClientBackend<Service>>,
    pub(crate) _response_payload: PhantomData<ResponsePayload>,
    pub(crate) _response_header: PhantomData<ResponseHeader>,
}

impl<
        Service: crate::service::Service,
        RequestPayload: Debug,
        RequestHeader: Debug,
        ResponsePayload: Debug,
        ResponseHeader: Debug,
    > Drop for RequestMut<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>
{
    fn drop(&mut self) {
        self.client_backend
            .server_connections
            .return_loaned_sample(self.offset_to_chunk);
    }
}

impl<
        Service: crate::service::Service,
        RequestPayload: Debug,
        RequestHeader: Debug,
        ResponsePayload: Debug,
        ResponseHeader: Debug,
    > Debug
    for RequestMut<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "RequestMut<{}, {}, {}, {}, {}> {{ }}",
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
    > RequestMut<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>
{
    pub fn header(&self) -> &service::header::request_response::RequestHeader {
        self.ptr.as_header_ref()
    }

    pub fn user_header(&self) -> &RequestHeader {
        self.ptr.as_user_header_ref()
    }

    pub fn user_header_mut(&mut self) -> &mut RequestHeader {
        self.ptr.as_user_header_mut()
    }

    pub fn payload(&self) -> &RequestPayload {
        self.ptr.as_payload_ref()
    }

    pub fn payload_mut(&mut self) -> &mut RequestPayload {
        self.ptr.as_payload_mut()
    }

    pub fn send(
        self,
    ) -> Result<
        ActiveRequest<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>,
        SendError,
    > {
        match self
            .client_backend
            .send_request(self.offset_to_chunk, self.sample_size)
        {
            Ok(number_of_server_connections) => {
                let active_request = ActiveRequest {
                    number_of_server_connections,
                    request: self,
                    _service: PhantomData,
                    _response_payload: PhantomData,
                    _response_header: PhantomData,
                };
                Ok(active_request)
            }
            Err(e) => Err(e),
        }
    }
}
