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

use std::fmt::Debug;
use std::marker::PhantomData;

use iceoryx2_bb_elementary::alignment::Alignment;
use iceoryx2_bb_log::fatal_panic;

use crate::service::builder;
use crate::service::{self, static_config};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RequestResponseOpenError {}

impl core::fmt::Display for RequestResponseOpenError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        std::write!(f, "RequestResponseOpenError::{:?}", self)
    }
}

impl std::error::Error for RequestResponseOpenError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RequestResponseCreateError {}

impl core::fmt::Display for RequestResponseCreateError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        std::write!(f, "RequestResponseCreateError::{:?}", self)
    }
}

impl std::error::Error for RequestResponseCreateError {}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum RequestResponseOpenOrCreateError {
    RequestResponseOpenError(RequestResponseOpenError),
    RequestResponseCreateError(RequestResponseCreateError),
}

impl From<RequestResponseOpenError> for RequestResponseOpenOrCreateError {
    fn from(value: RequestResponseOpenError) -> Self {
        RequestResponseOpenOrCreateError::RequestResponseOpenError(value)
    }
}

impl From<RequestResponseCreateError> for RequestResponseOpenOrCreateError {
    fn from(value: RequestResponseCreateError) -> Self {
        RequestResponseOpenOrCreateError::RequestResponseCreateError(value)
    }
}

impl core::fmt::Display for RequestResponseOpenOrCreateError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        std::write!(f, "RequestResponseOpenOrCreateError::{:?}", self)
    }
}

impl std::error::Error for RequestResponseOpenOrCreateError {}

#[derive(Debug)]
pub struct BuilderRequest<RequestPayload: Debug, ServiceType: service::Service> {
    base: builder::BuilderWithServiceType<ServiceType>,
    _request_payload: PhantomData<RequestPayload>,
}

impl<RequestPayload: Debug, ServiceType: service::Service>
    BuilderRequest<RequestPayload, ServiceType>
{
    pub(crate) fn new(base: builder::BuilderWithServiceType<ServiceType>) -> Self {
        Self {
            base,
            _request_payload: PhantomData,
        }
    }

    pub fn response<ResponsePayload: Debug>(
        self,
    ) -> Builder<RequestPayload, (), ResponsePayload, (), ServiceType> {
        Builder::new(self.base)
    }
}

#[derive(Debug)]
pub struct Builder<
    RequestPayload: Debug,
    RequestHeader: Debug,
    ResponsePayload: Debug,
    ResponseHeader: Debug,
    ServiceType: service::Service,
> {
    base: builder::BuilderWithServiceType<ServiceType>,
    override_request_alignment: Option<usize>,
    override_response_alignment: Option<usize>,
    verify_enable_safe_overflow_for_requests: bool,
    verify_enable_safe_overflow_for_responses: bool,
    verify_max_active_requests: bool,
    verify_max_borrowed_responses: bool,
    verify_max_response_buffer_size: bool,

    _request_payload: PhantomData<RequestPayload>,
    _request_header: PhantomData<RequestHeader>,
    _response_payload: PhantomData<ResponsePayload>,
    _response_header: PhantomData<ResponseHeader>,
}

impl<
        RequestPayload: Debug,
        RequestHeader: Debug,
        ResponsePayload: Debug,
        ResponseHeader: Debug,
        ServiceType: service::Service,
    > Builder<RequestPayload, RequestHeader, ResponsePayload, ResponseHeader, ServiceType>
{
    fn new(base: builder::BuilderWithServiceType<ServiceType>) -> Self {
        Self {
            base,
            override_request_alignment: None,
            override_response_alignment: None,
            verify_enable_safe_overflow_for_requests: false,
            verify_enable_safe_overflow_for_responses: false,
            verify_max_active_requests: false,
            verify_max_borrowed_responses: false,
            verify_max_response_buffer_size: false,
            _request_payload: PhantomData,
            _request_header: PhantomData,
            _response_payload: PhantomData,
            _response_header: PhantomData,
        }
    }

    fn config_details_mut(&mut self) -> &mut static_config::request_response::StaticConfig {
        match self.base.service_config.messaging_pattern {
            static_config::messaging_pattern::MessagingPattern::RequestResponse(ref mut v) => v,
            _ => {
                fatal_panic!(from self, "This should never happen! Accessing wrong messaging pattern in RequestResponse builder!");
            }
        }
    }

    fn config_details(&self) -> &static_config::request_response::StaticConfig {
        match self.base.service_config.messaging_pattern {
            static_config::messaging_pattern::MessagingPattern::RequestResponse(ref v) => v,
            _ => {
                fatal_panic!(from self, "This should never happen! Accessing wrong messaging pattern in RequestResponse builder!");
            }
        }
    }

    pub fn request_header<M: Debug>(
        self,
    ) -> Builder<RequestPayload, M, ResponsePayload, ResponseHeader, ServiceType> {
        unsafe {
            core::mem::transmute::<
                Self,
                Builder<RequestPayload, M, ResponsePayload, ResponseHeader, ServiceType>,
            >(self)
        }
    }

    pub fn response_header<M: Debug>(
        self,
    ) -> Builder<RequestPayload, RequestHeader, ResponsePayload, M, ServiceType> {
        unsafe {
            core::mem::transmute::<
                Self,
                Builder<RequestPayload, RequestHeader, ResponsePayload, M, ServiceType>,
            >(self)
        }
    }

    pub fn request_payload_alignment(mut self, alignment: Alignment) -> Self {
        self.override_request_alignment = Some(alignment.value());
        self
    }

    pub fn response_payload_alignment(mut self, alignment: Alignment) -> Self {
        self.override_response_alignment = Some(alignment.value());
        self
    }

    pub fn enable_safe_overflow_for_requests(mut self, value: bool) -> Self {
        self.config_details_mut().enable_safe_overflow_for_requests = value;
        self.verify_enable_safe_overflow_for_requests = true;
        self
    }

    pub fn enable_safe_overflow_for_responses(mut self, value: bool) -> Self {
        self.config_details_mut().enable_safe_overflow_for_responses = value;
        self.verify_enable_safe_overflow_for_responses = true;
        self
    }

    pub fn max_active_requests(mut self, value: usize) -> Self {
        self.config_details_mut().max_active_requests = value;
        self.verify_max_active_requests = true;
        self
    }

    pub fn max_borrowed_responses(mut self, value: usize) -> Self {
        self.config_details_mut().max_borrowed_responses = value;
        self.verify_max_borrowed_responses = true;
        self
    }

    pub fn max_response_buffer_size(mut self, value: usize) -> Self {
        self.config_details_mut().max_response_buffer_size = value;
        self.verify_max_response_buffer_size = true;
        self
    }

    pub fn max_servers(mut self, value: usize) -> Self {
        todo!()
    }

    pub fn max_clients(mut self, value: usize) -> Self {
        todo!()
    }

    pub fn max_nodes(mut self, value: usize) -> Self {
        todo!()
    }
}
