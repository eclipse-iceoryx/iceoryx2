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

use crate::{response_mut::ResponseMut, service};
use core::{fmt::Debug, mem::MaybeUninit};

pub struct ResponseMutUninit<
    Service: service::Service,
    ResponsePayload: Debug,
    ResponseHeader: Debug,
> {
    pub(crate) response: ResponseMut<Service, ResponsePayload, ResponseHeader>,
}

impl<Service: crate::service::Service, ResponsePayload: Debug, ResponseHeader: Debug> Debug
    for ResponseMutUninit<Service, ResponsePayload, ResponseHeader>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ResponseMut {{ response: {:?} }}", self.response)
    }
}

impl<Service: crate::service::Service, ResponsePayload: Debug, ResponseHeader: Debug>
    ResponseMutUninit<Service, ResponsePayload, ResponseHeader>
{
    pub fn header(&self) -> &service::header::request_response::ResponseHeader {
        self.response.header()
    }

    pub fn user_header(&self) -> &ResponseHeader {
        self.response.user_header()
    }

    pub fn user_header_mut(&mut self) -> &mut ResponseHeader {
        self.response.user_header_mut()
    }

    pub fn payload(&self) -> &ResponsePayload {
        self.response.payload()
    }

    pub fn payload_mut(&mut self) -> &mut ResponsePayload {
        self.response.payload_mut()
    }
}

impl<Service: crate::service::Service, ResponsePayload: Debug, ResponseHeader: Debug>
    ResponseMutUninit<Service, MaybeUninit<ResponsePayload>, ResponseHeader>
{
    pub fn write_payload(
        mut self,
        value: ResponsePayload,
    ) -> ResponseMut<Service, ResponsePayload, ResponseHeader> {
        self.payload_mut().write(value);
        unsafe { self.assume_init() }
    }

    pub unsafe fn assume_init(self) -> ResponseMut<Service, ResponsePayload, ResponseHeader> {
        // the transmute is not nice but safe since MaybeUninit is #[repr(transparent)] to the inner type
        core::mem::transmute(self.response)
    }
}
