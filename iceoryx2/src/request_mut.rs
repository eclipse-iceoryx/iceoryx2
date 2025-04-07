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
//! # let node = NodeBuilder::new().create::<ipc::Service>()?;
//! #
//! # let service = node
//! #    .service_builder(&"My/Funk/ServiceName".try_into()?)
//! #    .request_response::<u64, u64>()
//! #    .open_or_create()?;
//! #
//! # let client = service.client_builder().create()?;
//!
//! let request = client.loan_uninit()?;
//! let request = request.write_payload(9219);
//!
//! println!("client port id: {:?}", request.header().client_port_id());
//! let pending_response = request.send()?;
//!
//! # Ok(())
//! # }
//! ```

extern crate alloc;

use alloc::sync::Arc;
use core::{fmt::Debug, marker::PhantomData};
use core::{
    ops::{Deref, DerefMut},
    sync::atomic::Ordering,
};

use iceoryx2_cal::shm_allocator::PointerOffset;
use iceoryx2_pal_concurrency_sync::iox_atomic::IoxAtomicBool;

use crate::{
    pending_response::PendingResponse,
    port::client::{ClientBackend, RequestSendError},
    raw_sample::RawSampleMut,
    service,
};

/// The [`RequestMut`] represents the object that contains the payload that the
/// [`Client`](crate::port::client::Client) sends to the
/// [`Server`](crate::port::server::Server).
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
    pub(crate) was_sample_sent: IoxAtomicBool,
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
            .request_sender
            .release_sample(self.offset_to_chunk);
        if !self.was_sample_sent.load(Ordering::Relaxed) {
            self.client_backend
                .request_sender
                .loan_counter
                .fetch_sub(1, Ordering::Relaxed);
        }
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
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
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
    > Deref
    for RequestMut<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>
{
    type Target = RequestPayload;
    fn deref(&self) -> &Self::Target {
        self.ptr.as_payload_ref()
    }
}

impl<
        Service: crate::service::Service,
        RequestPayload: Debug,
        RequestHeader: Debug,
        ResponsePayload: Debug,
        ResponseHeader: Debug,
    > DerefMut
    for RequestMut<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.ptr.as_payload_mut()
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
    /// Returns a reference to the iceoryx2 internal
    /// [`service::header::request_response::RequestHeader`]
    pub fn header(&self) -> &service::header::request_response::RequestHeader {
        self.ptr.as_header_ref()
    }

    /// Returns a reference to the user defined request header.
    pub fn user_header(&self) -> &RequestHeader {
        self.ptr.as_user_header_ref()
    }

    /// Returns a mutable reference to the user defined request header.
    pub fn user_header_mut(&mut self) -> &mut RequestHeader {
        self.ptr.as_user_header_mut()
    }

    /// Returns a reference to the user defined request payload.
    pub fn payload(&self) -> &RequestPayload {
        self.ptr.as_payload_ref()
    }

    /// Returns a mutable reference to the user defined request payload.
    pub fn payload_mut(&mut self) -> &mut RequestPayload {
        self.ptr.as_payload_mut()
    }

    /// Sends the [`RequestMut`] to all connected
    /// [`Server`](crate::port::server::Server)s of the
    /// [`Service`](crate::service::Service).
    pub fn send(
        self,
    ) -> Result<
        PendingResponse<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>,
        RequestSendError,
    > {
        match self
            .client_backend
            .send_request(self.offset_to_chunk, self.sample_size)
        {
            Ok(number_of_server_connections) => {
                self.was_sample_sent.store(true, Ordering::Relaxed);
                self.client_backend
                    .request_sender
                    .loan_counter
                    .fetch_sub(1, Ordering::Relaxed);
                let active_request = PendingResponse {
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
