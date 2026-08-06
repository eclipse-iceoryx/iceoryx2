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
//! let request = client.loan_uninit()?;
//! let request = request.write_payload(9219);
//!
//! println!("client id: {:?}", request.header().client_id());
//! let pending_response = request.send()?;
//!
//! # Ok(())
//! # }
//! ```

use core::ops::{Deref, DerefMut};
use core::{fmt::Debug, marker::PhantomData};

use iceoryx2_bb_concurrency::atomic::{AtomicBool, Ordering};
use iceoryx2_bb_elementary_traits::iceoryx_send::IceoryxSend;
use iceoryx2_bb_elementary_traits::testing::abandonable::Abandonable;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use iceoryx2_cal::zero_copy_connection::ChannelId;

use crate::payload::number_of_elements;
use crate::port::details::chunk::ChunkMut;
use crate::port::details::chunk_mut_shared_state::ChunkMutSharedState;
use crate::{
    pending_response::PendingResponse,
    port::client::{ClientSharedState, RequestSendError},
    service,
};

/// The [`RequestMut`] represents the object that contains the payload that the
/// [`Client`](crate::port::client::Client) sends to the
/// [`Server`](crate::port::server::Server).
pub struct RequestMut<
    Service: crate::service::Service,
    RequestPayload: Debug + IceoryxSend + ?Sized,
    RequestHeader: Debug + ZeroCopySend,
    ResponsePayload: Debug + IceoryxSend + ?Sized,
    ResponseHeader: Debug + ZeroCopySend,
> {
    pub(crate) chunk: ChunkMut,
    pub(crate) shared_state: ChunkMutSharedState<Service, ClientSharedState<Service>>,
    pub(crate) was_sample_sent: AtomicBool,
    pub(crate) channel_id: ChannelId,
    pub(crate) _request_payload: PhantomData<RequestPayload>,
    pub(crate) _request_header: PhantomData<RequestHeader>,
    pub(crate) _response_payload: PhantomData<ResponsePayload>,
    pub(crate) _response_header: PhantomData<ResponseHeader>,
}

impl<
    Service: crate::service::Service,
    RequestPayload: Debug + IceoryxSend + ?Sized,
    RequestHeader: Debug + ZeroCopySend,
    ResponsePayload: Debug + IceoryxSend + ?Sized,
    ResponseHeader: Debug + ZeroCopySend,
> Abandonable
    for RequestMut<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>
{
    unsafe fn abandon_in_place(mut this: core::ptr::NonNull<Self>) {
        let this = unsafe { this.as_mut() };
        unsafe { core::ptr::drop_in_place(&mut this.shared_state) };
    }
}

unsafe impl<
    Service: crate::service::Service,
    RequestPayload: Debug + IceoryxSend + ?Sized,
    RequestHeader: Debug + ZeroCopySend,
    ResponsePayload: Debug + IceoryxSend + ?Sized,
    ResponseHeader: Debug + ZeroCopySend,
> Send for RequestMut<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>
where
    Service::ArcThreadSafetyPolicy<ClientSharedState<Service>>: Send + Sync,
{
}

impl<
    Service: crate::service::Service,
    RequestPayload: Debug + IceoryxSend + ?Sized,
    RequestHeader: Debug + ZeroCopySend,
    ResponsePayload: Debug + IceoryxSend + ?Sized,
    ResponseHeader: Debug + ZeroCopySend,
> Drop for RequestMut<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>
{
    fn drop(&mut self) {
        let _ = self.shared_state.call(|s| -> Result<(), ()> {
            s.release_request(self.was_sample_sent.load(Ordering::Relaxed), self.header());
            Ok(())
        });
    }
}

impl<
    Service: crate::service::Service,
    RequestPayload: Debug + IceoryxSend + ?Sized,
    RequestHeader: Debug + ZeroCopySend,
    ResponsePayload: Debug + IceoryxSend + ?Sized,
    ResponseHeader: Debug + ZeroCopySend,
> Debug for RequestMut<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "RequestMut<{}, {}, {}, {}, {}> {{ chunk: {:?}, was_sample_sent: {}, channel_id: {} }}",
            core::any::type_name::<Service>(),
            core::any::type_name::<RequestPayload>(),
            core::any::type_name::<RequestHeader>(),
            core::any::type_name::<ResponsePayload>(),
            core::any::type_name::<ResponseHeader>(),
            self.chunk,
            self.was_sample_sent.load(Ordering::Relaxed),
            self.channel_id.value()
        )
    }
}

impl<
    Service: crate::service::Service,
    RequestPayload: Debug + IceoryxSend + ZeroCopySend,
    RequestHeader: Debug + ZeroCopySend,
    ResponsePayload: Debug + IceoryxSend + ?Sized,
    ResponseHeader: Debug + ZeroCopySend,
> Deref for RequestMut<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>
{
    type Target = RequestPayload;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.chunk.payload_ptr().cast() }
    }
}

impl<
    Service: crate::service::Service,
    RequestPayload: Debug + IceoryxSend + ZeroCopySend,
    RequestHeader: Debug + ZeroCopySend,
    ResponsePayload: Debug + IceoryxSend + ?Sized,
    ResponseHeader: Debug + ZeroCopySend,
> Deref for RequestMut<Service, [RequestPayload], RequestHeader, ResponsePayload, ResponseHeader>
{
    type Target = [RequestPayload];
    fn deref(&self) -> &Self::Target {
        let payload_size = self.shared_state.payload_size();

        unsafe {
            &*core::ptr::slice_from_raw_parts(
                self.chunk.payload_ptr().cast(),
                number_of_elements::<RequestPayload, _>(self.header(), payload_size),
            )
        }
    }
}

impl<
    Service: crate::service::Service,
    RequestPayload: Debug + IceoryxSend + ZeroCopySend,
    RequestHeader: Debug + ZeroCopySend,
    ResponsePayload: Debug + IceoryxSend + ?Sized,
    ResponseHeader: Debug + ZeroCopySend,
> DerefMut for RequestMut<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.chunk.payload_mut_ptr().cast() }
    }
}

impl<
    Service: crate::service::Service,
    RequestPayload: Debug + IceoryxSend + ZeroCopySend,
    RequestHeader: Debug + ZeroCopySend,
    ResponsePayload: Debug + IceoryxSend + ?Sized,
    ResponseHeader: Debug + ZeroCopySend,
> DerefMut
    for RequestMut<Service, [RequestPayload], RequestHeader, ResponsePayload, ResponseHeader>
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        let payload_size = self.shared_state.payload_size();
        unsafe {
            &mut *core::ptr::slice_from_raw_parts_mut(
                self.chunk.payload_mut_ptr().cast(),
                number_of_elements::<RequestPayload, _>(self.header(), payload_size),
            )
        }
    }
}

impl<
    Service: crate::service::Service,
    RequestPayload: Debug + IceoryxSend + ?Sized,
    RequestHeader: Debug + ZeroCopySend,
    ResponsePayload: Debug + IceoryxSend + ?Sized,
    ResponseHeader: Debug + ZeroCopySend,
> RequestMut<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>
{
    /// Returns a reference to the iceoryx2 internal
    /// [`service::header::request_response::RequestHeader`]
    pub fn header(&self) -> &service::header::request_response::RequestHeader {
        unsafe { &*self.chunk.header_ptr().cast() }
    }

    /// Returns a reference to the user defined request header.
    pub fn user_header(&self) -> &RequestHeader {
        unsafe { &*self.chunk.user_header_ptr().cast() }
    }

    /// Returns a mutable reference to the user defined request header.
    pub fn user_header_mut(&mut self) -> &mut RequestHeader {
        unsafe { &mut *self.chunk.user_header_mut_ptr().cast() }
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
        let shared_state = self.shared_state.clone();
        shared_state.call(|s| {
            match s.send_request(&self.chunk, self.channel_id, self.header().request_id) {
                Ok(number_of_server_connections) => {
                    self.was_sample_sent.store(true, Ordering::Relaxed);
                    s.loan_counter.fetch_sub(1, Ordering::Relaxed);
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
        })
    }
}

impl<
    Service: crate::service::Service,
    RequestPayload: Debug + IceoryxSend + ZeroCopySend,
    RequestHeader: Debug + ZeroCopySend,
    ResponsePayload: Debug + IceoryxSend + ?Sized,
    ResponseHeader: Debug + ZeroCopySend,
> RequestMut<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>
{
    /// Returns a reference to the user defined request payload.
    pub fn payload(&self) -> &RequestPayload {
        self.deref()
    }

    /// Returns a mutable reference to the user defined request payload.
    pub fn payload_mut(&mut self) -> &mut RequestPayload {
        &mut *self
    }
}

impl<
    Service: crate::service::Service,
    RequestPayload: Debug + IceoryxSend + ZeroCopySend,
    RequestHeader: Debug + ZeroCopySend,
    ResponsePayload: Debug + IceoryxSend + ?Sized,
    ResponseHeader: Debug + ZeroCopySend,
> RequestMut<Service, [RequestPayload], RequestHeader, ResponsePayload, ResponseHeader>
{
    /// Returns a reference to the user defined request payload.
    pub fn payload(&self) -> &[RequestPayload] {
        self.deref()
    }

    /// Returns a mutable reference to the user defined request payload.
    pub fn payload_mut(&mut self) -> &mut [RequestPayload] {
        &mut *self
    }
}
