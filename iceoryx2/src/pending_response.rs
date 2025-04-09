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
//! # let request = request.write_payload(0);
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

use core::sync::atomic::Ordering;
use core::{fmt::Debug, marker::PhantomData};

use iceoryx2_bb_log::fail;

use crate::port::details::chunk::Chunk;
use crate::port::details::chunk_details::ChunkDetails;
use crate::port::update_connections::ConnectionFailure;
use crate::raw_sample::RawSample;
use crate::{port::ReceiveError, request_mut::RequestMut, response::Response, service};

/// Represents an active connection to all [`Server`](crate::port::server::Server)
/// that received the [`RequestMut`]. The
/// [`Client`](crate::port::client::Client) can use it to receive the corresponding
/// [`Response`]s.
///
/// As soon as it goes out of scope, the connections are closed and the
/// [`Server`](crate::port::server::Server)s are informed.
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
    > Drop
    for PendingResponse<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>
{
    fn drop(&mut self) {
        self.request
            .client_shared_state
            .active_request_counter
            .fetch_sub(1, Ordering::Relaxed);
        self.close();
    }
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
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "PendingResponse<{}, {}, {}, {}, {}> {{ number_of_server_connections: {} }}",
            core::any::type_name::<Service>(),
            core::any::type_name::<RequestPayload>(),
            core::any::type_name::<RequestHeader>(),
            core::any::type_name::<ResponsePayload>(),
            core::any::type_name::<ResponseHeader>(),
            self.number_of_server_connections
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
    fn close(&self) {
        self.request
            .client_shared_state
            .response_receiver
            .invalidate_channel_state(self.request.channel_id, self.request.header().request_id);
    }

    pub fn is_connected(&self) -> bool {
        self.request
            .client_shared_state
            .response_receiver
            .has_at_least_one_channel_the_state(
                self.request.channel_id,
                self.request.header().request_id,
            )
    }

    /// Returns a reference to the iceoryx2 internal
    /// [`service::header::request_response::RequestHeader`] of the corresponding
    /// [`RequestMut`]
    pub fn header(&self) -> &service::header::request_response::RequestHeader {
        self.request.header()
    }

    /// Returns a reference to the user defined request header of the corresponding
    /// [`RequestMut`]
    pub fn user_header(&self) -> &RequestHeader {
        self.request.user_header()
    }

    /// Returns a reference to the request payload of the corresponding
    /// [`RequestMut`]
    pub fn payload(&self) -> &RequestPayload {
        self.request.payload()
    }

    /// Returns how many [`Server`](crate::port::server::Server)s received the corresponding
    /// [`RequestMut`] initially.
    pub fn number_of_server_connections(&self) -> usize {
        self.number_of_server_connections
    }

    pub fn has_response(&self) -> Result<bool, ConnectionFailure> {
        fail!(from self, when self.request.client_shared_state.update_connections(),
                "Some samples are not being received since not all connections to publishers could be established.");
        self.request
            .client_shared_state
            .response_receiver
            .has_samples(self.request.channel_id)
    }

    fn receive_impl(&self) -> Result<Option<(ChunkDetails<Service>, Chunk)>, ReceiveError> {
        let msg = "Unable to receive response";
        fail!(from self, when self.request.client_shared_state.update_connections(),
                "{msg} since the connections could not be updated.");

        self.request
            .client_shared_state
            .response_receiver
            .receive(self.request.channel_id)
    }

    pub fn receive(
        &self,
    ) -> Result<Option<Response<Service, ResponsePayload, ResponseHeader>>, ReceiveError> {
        loop {
            match self.receive_impl()? {
                None => return Ok(None),
                Some((details, chunk)) => {
                    let response = Response {
                        details,
                        channel_id: self.request.channel_id,
                        ptr: unsafe {
                            RawSample::new_unchecked(
                                chunk.header.cast(),
                                chunk.user_header.cast(),
                                chunk.payload.cast::<ResponsePayload>(),
                            )
                        },
                    };

                    if response.header().request_id != self.request.header().request_id {
                        continue;
                    }

                    return Ok(Some(response));
                }
            }
        }
    }
}
