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
//! # let request = client.loan_uninit()?;
//! # let request = request.write_payload(counter);
//!
//! let pending_response = request.send()?;
//!
//! println!("send request to {} server",
//!           pending_response.number_of_server_connections());
//!
//! // We are no longer interested in the responses from the server and
//! // drop the object. This informs the corresponding servers, that hold
//! // an ActiveRequest that the connection was terminated from the client
//! // side so that they can stop sending responses.
//! drop(pending_response);
//!
//! # Ok(())
//! # }
//! ```

use core::{fmt::Debug, marker::PhantomData};

use crate::{port::ReceiveError, request_mut::RequestMut, response::Response, service};

/// Represents an active connection to all [`Server`](crate::port::server::Server)
/// that received the [`RequestMut`](crate::request_mut::RequestMut). The
/// [`Client`] can use it to receive the corresponding
/// [`Response`](crate::response::Response)s.
///
/// As soon as it goes out of scope, the connections are closed and the [`Server`]
/// is informed.
pub struct PendingResponse<
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
    for PendingResponse<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>
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
    > PendingResponse<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>
{
    /// Returns a reference to the iceoryx2 internal
    /// [`service::header::request_response::RequestHeader`] of the corresponding
    /// [`RequestMut`](crate::request_mut::RequestMut)
    pub fn header(&self) -> &service::header::request_response::RequestHeader {
        self.request.header()
    }

    /// Returns a reference to the user defined request header of the corresponding
    /// [`RequestMut`](crate::request_mut::RequestMut)
    pub fn user_header(&self) -> &RequestHeader {
        self.request.user_header()
    }

    /// Returns a reference to the request payload of the corresponding
    /// [`RequestMut`](crate::request_mut::RequestMut)
    pub fn payload(&self) -> &RequestPayload {
        self.request.payload()
    }

    /// Returns how many server received the corresponding
    /// [`RequestMut`](crate::port::request_mut::RequestMut) initially.
    pub fn number_of_server_connections(&self) -> usize {
        self.number_of_server_connections
    }

    /// todo
    pub fn receive(
        &self,
    ) -> Result<Option<Response<Service, ResponsePayload, ResponseHeader>>, ReceiveError> {
        todo!()
    }
}
