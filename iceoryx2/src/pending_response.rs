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
//! # fn main() -> Result<(), Box<dyn core::error::Error>> {
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
//! // we receive a stream of responses from the server and are interested in 5 of them
//! for i in 0..5 {
//!     if !pending_response.is_connected() {
//!         println!("server terminated connection - abort");
//!         break;
//!     }
//!
//!     if let Some(response) = pending_response.receive()? {
//!         println!("received response: {}", *response);
//!     }
//! }
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

use core::ops::Deref;
use core::sync::atomic::Ordering;
use core::{fmt::Debug, marker::PhantomData};

use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use iceoryx2_bb_log::fail;
use iceoryx2_cal::arc_sync_policy::ArcSyncPolicy;

use crate::port::client::ClientSharedState;
use crate::port::details::chunk::Chunk;
use crate::port::details::chunk_details::ChunkDetails;
use crate::raw_sample::RawSample;
use crate::service::builder::CustomPayloadMarker;
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
    RequestPayload: Debug + ZeroCopySend + ?Sized,
    RequestHeader: Debug + ZeroCopySend,
    ResponsePayload: Debug + ZeroCopySend + ?Sized,
    ResponseHeader: Debug + ZeroCopySend,
> {
    pub(crate) request:
        RequestMut<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>,
    pub(crate) number_of_server_connections: usize,
    pub(crate) _service: PhantomData<Service>,
    pub(crate) _response_payload: PhantomData<ResponsePayload>,
    pub(crate) _response_header: PhantomData<ResponseHeader>,
}

unsafe impl<
        Service: crate::service::Service,
        RequestPayload: Debug + ZeroCopySend + ?Sized,
        RequestHeader: Debug + ZeroCopySend,
        ResponsePayload: Debug + ZeroCopySend + ?Sized,
        ResponseHeader: Debug + ZeroCopySend,
    > Send
    for PendingResponse<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>
where
    Service::ArcThreadSafetyPolicy<ClientSharedState<Service>>: Send + Sync,
{
}

impl<
        Service: crate::service::Service,
        RequestPayload: Debug + ZeroCopySend + ?Sized,
        RequestHeader: Debug + ZeroCopySend,
        ResponsePayload: Debug + ZeroCopySend + ?Sized,
        ResponseHeader: Debug + ZeroCopySend,
    > Drop
    for PendingResponse<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>
{
    fn drop(&mut self) {
        self.request
            .client_shared_state
            .lock()
            .active_request_counter
            .fetch_sub(1, Ordering::Relaxed);
        self.close();
    }
}

impl<
        Service: crate::service::Service,
        RequestPayload: Debug + ZeroCopySend + ?Sized,
        RequestHeader: Debug + ZeroCopySend,
        ResponsePayload: Debug + ZeroCopySend + ?Sized,
        ResponseHeader: Debug + ZeroCopySend,
    > Deref
    for PendingResponse<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>
{
    type Target = RequestPayload;
    fn deref(&self) -> &Self::Target {
        self.request.payload()
    }
}

impl<
        Service: crate::service::Service,
        RequestPayload: Debug + ZeroCopySend + ?Sized,
        RequestHeader: Debug + ZeroCopySend,
        ResponsePayload: Debug + ZeroCopySend + ?Sized,
        ResponseHeader: Debug + ZeroCopySend,
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
        RequestPayload: Debug + ZeroCopySend + ?Sized,
        RequestHeader: Debug + ZeroCopySend,
        ResponsePayload: Debug + ZeroCopySend + ?Sized,
        ResponseHeader: Debug + ZeroCopySend,
    > PendingResponse<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>
{
    fn close(&self) {
        self.request
            .client_shared_state
            .lock()
            .response_receiver
            .invalidate_channel_state(self.request.channel_id, self.request.header().request_id);
    }

    /// Marks the connection state that the [`Client`](crate::port::client::Client) wants to gracefully
    /// disconnect. When the [`Server`](crate::port::server::Server) reads this, it can send the last
    /// [`Response`] and drop the corresponding [`ActiveRequest`](crate::active_request::ActiveRequest) to
    /// terminate the connection ensuring that no [`Response`] is lost on the
    /// [`Client`](crate::port::client::Client) side.
    pub fn set_disconnect_hint(&self) {
        self.request
            .client_shared_state
            .lock()
            .response_receiver
            .set_disconnect_hint(self.request.channel_id, self.request.header().request_id);
    }

    /// Returns [`true`] until the [`ActiveRequest`](crate::active_request::ActiveRequest)
    /// goes out of scope on the [`Server`](crate::port::server::Server)s side indicating that the
    /// [`Server`](crate::port::server::Server) will no longer send [`Response`]s.
    /// It also returns [`false`] when there are no [`Server`](crate::port::server::Server)s.
    pub fn is_connected(&self) -> bool {
        self.request
            .client_shared_state
            .lock()
            .response_receiver
            .at_least_one_channel_has_state(
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

    /// Returns [`true`] when a [`Server`](crate::port::server::Server) has sent a [`Response`]
    /// otherwise [`false`].
    pub fn has_response(&self) -> bool {
        self.request
            .client_shared_state
            .lock()
            .response_receiver
            .has_samples(self.request.channel_id)
    }

    fn receive_impl(&self) -> Result<Option<(ChunkDetails, Chunk)>, ReceiveError> {
        let client_shared_state = self.request.client_shared_state.lock();
        let msg = "Unable to receive response";
        fail!(from self, when client_shared_state.update_connections(),
                "{msg} since the connections could not be updated.");

        client_shared_state
            .response_receiver
            .receive(self.request.channel_id)
    }
}

impl<
        Service: crate::service::Service,
        RequestPayload: Debug + ZeroCopySend + ?Sized,
        RequestHeader: Debug + ZeroCopySend,
        ResponsePayload: Debug + ZeroCopySend + Sized,
        ResponseHeader: Debug + ZeroCopySend,
    > PendingResponse<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>
{
    /// Receives a [`Response`] from one of the [`Server`](crate::port::server::Server)s that
    /// received the [`RequestMut`].
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    ///
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// # let service = node
    /// #    .service_builder(&"My/Funk/ServiceName".try_into()?)
    /// #    .request_response::<u64, u64>()
    /// #    .open_or_create()?;
    /// #
    /// # let client = service.client_builder().create()?;
    ///
    /// # let request = client.loan_uninit()?;
    /// # let request = request.write_payload(0);
    ///
    /// let pending_response = request.send()?;
    ///
    /// if let Some(response) = pending_response.receive()? {
    ///     println!("received response: {}", *response);
    /// }
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn receive(
        &self,
    ) -> Result<Option<Response<Service, ResponsePayload, ResponseHeader>>, ReceiveError> {
        loop {
            match self.receive_impl()? {
                None => return Ok(None),
                Some((details, chunk)) => {
                    let response = Response {
                        details,
                        client_shared_state: self.request.client_shared_state.clone(),
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

impl<
        Service: crate::service::Service,
        RequestPayload: Debug + ZeroCopySend + ?Sized,
        RequestHeader: Debug + ZeroCopySend,
        ResponsePayload: Debug + ZeroCopySend,
        ResponseHeader: Debug + ZeroCopySend,
    > PendingResponse<Service, RequestPayload, RequestHeader, [ResponsePayload], ResponseHeader>
{
    /// Receives a [`Response`] from one of the [`Server`](crate::port::server::Server)s that
    /// received the [`RequestMut`].
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    ///
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// # let service = node
    /// #    .service_builder(&"My/Funk/ServiceName".try_into()?)
    /// #    .request_response::<u64, [usize]>()
    /// #    .open_or_create()?;
    /// #
    /// # let client = service.client_builder().create()?;
    ///
    /// # let request = client.loan_uninit()?;
    /// # let request = request.write_payload(0);
    ///
    /// let pending_response = request.send()?;
    ///
    /// if let Some(response) = pending_response.receive()? {
    ///     println!("received response: {:?}", response);
    /// }
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn receive(
        &self,
    ) -> Result<Option<Response<Service, [ResponsePayload], ResponseHeader>>, ReceiveError> {
        loop {
            match self.receive_impl()? {
                None => return Ok(None),
                Some((details, chunk)) => {
                    let header = unsafe {
                        &*(chunk.header as *const service::header::request_response::ResponseHeader)
                    };

                    let response = Response {
                        details,
                        channel_id: self.request.channel_id,
                        client_shared_state: self.request.client_shared_state.clone(),
                        ptr: unsafe {
                            RawSample::new_slice_unchecked(
                                chunk.header.cast(),
                                chunk.user_header.cast(),
                                core::slice::from_raw_parts(
                                    chunk.payload.cast::<ResponsePayload>(),
                                    header.number_of_elements() as _,
                                ),
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

impl<
        Service: crate::service::Service,
        RequestHeader: Debug + ZeroCopySend,
        ResponseHeader: Debug + ZeroCopySend,
    >
    PendingResponse<
        Service,
        [CustomPayloadMarker],
        RequestHeader,
        [CustomPayloadMarker],
        ResponseHeader,
    >
{
    #[doc(hidden)]
    pub unsafe fn receive_custom_payload(
        &self,
    ) -> Result<Option<Response<Service, [CustomPayloadMarker], ResponseHeader>>, ReceiveError>
    {
        loop {
            match self.receive_impl()? {
                None => return Ok(None),
                Some((details, chunk)) => {
                    let header = unsafe {
                        &*(chunk.header as *const service::header::request_response::ResponseHeader)
                    };

                    let number_of_elements = (*header).number_of_elements();
                    let number_of_bytes = number_of_elements as usize
                        * self
                            .request
                            .client_shared_state
                            .lock()
                            .response_receiver
                            .payload_size();

                    let response = Response {
                        details,
                        channel_id: self.request.channel_id,
                        client_shared_state: self.request.client_shared_state.clone(),
                        ptr: unsafe {
                            RawSample::new_slice_unchecked(
                                chunk.header.cast(),
                                chunk.user_header.cast(),
                                core::slice::from_raw_parts(
                                    chunk.payload.cast::<CustomPayloadMarker>(),
                                    number_of_bytes as _,
                                ),
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
