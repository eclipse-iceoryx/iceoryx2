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

use core::{fmt::Debug, marker::PhantomData, mem::MaybeUninit};
use std::sync::Arc;

use iceoryx2_cal::shm_allocator::PointerOffset;

use crate::{
    port::details::outgoing_connections::OutgoingConnections, raw_sample::RawSampleMut,
    request_mut::RequestMut, service,
};

#[repr(transparent)]
pub struct RequestMutUninit<
    Service: crate::service::Service,
    RequestPayload: Debug,
    RequestHeader: Debug,
    ResponsePayload: Debug,
    ResponseHeader: Debug,
> {
    request: RequestMut<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>,
}

impl<
        Service: crate::service::Service,
        RequestPayload: Debug,
        RequestHeader: Debug,
        ResponsePayload: Debug,
        ResponseHeader: Debug,
    > Debug
    for RequestMutUninit<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "RequestMutUninit<{}, {}, {}, {}, {}> {{ }}",
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
    > RequestMutUninit<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>
{
    pub(crate) fn new(
        raw_sample: RawSampleMut<
            service::header::request_response::RequestHeader,
            RequestHeader,
            RequestPayload,
        >,
        offset_to_chunk: PointerOffset,
        sample_size: usize,
        server_connections: Arc<OutgoingConnections<Service>>,
    ) -> Self {
        Self {
            request: RequestMut {
                ptr: raw_sample,
                _response_payload: PhantomData,
                _response_header: PhantomData,
                offset_to_chunk,
                sample_size,
                server_connections,
            },
        }
    }

    pub fn header(&self) -> &service::header::request_response::RequestHeader {
        self.request.header()
    }

    pub fn user_header(&self) -> &RequestHeader {
        self.request.user_header()
    }

    pub fn user_header_mut(&mut self) -> &mut RequestHeader {
        self.request.user_header_mut()
    }

    pub fn payload(&self) -> &RequestPayload {
        self.request.payload()
    }

    pub fn payload_mut(&mut self) -> &mut RequestPayload {
        self.request.payload_mut()
    }
}

impl<
        Service: crate::service::Service,
        RequestPayload: Debug,
        RequestHeader: Debug,
        ResponsePayload: Debug,
        ResponseHeader: Debug,
    >
    RequestMutUninit<
        Service,
        MaybeUninit<RequestPayload>,
        RequestHeader,
        ResponsePayload,
        ResponseHeader,
    >
{
    pub fn write_payload(
        mut self,
        value: RequestPayload,
    ) -> RequestMut<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader> {
        self.payload_mut().write(value);
        unsafe { self.assume_init() }
    }

    pub unsafe fn assume_init(
        self,
    ) -> RequestMut<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader> {
        // the transmute is not nice but safe since MaybeUninit is #[repr(transparent)] to the inner type
        core::mem::transmute(self.request)
    }
}
