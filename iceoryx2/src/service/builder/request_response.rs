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

use crate::service;
use crate::service::builder;

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
            _request_payload: PhantomData,
            _request_header: PhantomData,
            _response_payload: PhantomData,
            _response_header: PhantomData,
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

    pub fn enable_safe_overflow_for_requests(mut self, value: bool) -> Self {
        todo!()
    }

    pub fn enable_safe_overflow_for_responses(mut self, value: bool) -> Self {
        todo!()
    }

    pub fn max_active_requests(mut self, value: usize) -> Self {
        todo!()
    }

    pub fn max_borrowed_responses(mut self, value: usize) -> Self {
        todo!()
    }

    pub fn response_buffer_size(mut self, value: usize) -> Self {
        todo!()
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
