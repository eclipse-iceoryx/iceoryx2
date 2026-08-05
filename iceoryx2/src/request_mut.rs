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

use iceoryx2_bb_concurrency::atomic::AtomicBool;
use iceoryx2_bb_concurrency::atomic::Ordering;
use iceoryx2_bb_elementary::static_assert_size_of;
use iceoryx2_bb_elementary_traits::iceoryx_send::IceoryxSend;
use iceoryx2_bb_elementary_traits::testing::abandonable::Abandonable;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use iceoryx2_cal::arc_sync_policy::ArcSyncPolicy;
use iceoryx2_cal::zero_copy_connection::ChannelId;
use iceoryx2_log::fatal_panic;

use crate::port::details::chunk::ChunkMut;
use crate::service::marker::CustomPayloadMarker;
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
    pub(crate) client_shared_state: Service::ArcThreadSafetyPolicy<ClientSharedState<Service>>,
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
        unsafe { core::ptr::drop_in_place(&mut this.client_shared_state) };
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
        let client_shared_state = self.client_shared_state.lock();
        if !unsafe { &mut *client_shared_state.available_channel_ids.get() }
            .push(self.header().channel_id)
        {
            fatal_panic!(from self,
                    "This should never happen! The channel id could not be returned.");
        }

        client_shared_state
            .request_sender
            .release_sample(self.chunk.offset());
        if !self.was_sample_sent.load(Ordering::Relaxed) {
            client_shared_state
                .request_sender
                .loan_counter
                .fetch_sub(1, Ordering::Relaxed);
        }
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
        unsafe {
            &*core::ptr::slice_from_raw_parts(
                self.chunk.payload_ptr().cast(),
                self.number_of_elements(),
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
        unsafe {
            &mut *core::ptr::slice_from_raw_parts_mut(
                self.chunk.payload_mut_ptr().cast(),
                self.number_of_elements(),
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
        let client_shared_state = self.client_shared_state.lock();
        match client_shared_state.send_request(
            &self.chunk,
            self.channel_id,
            self.header().request_id,
        ) {
            Ok(number_of_server_connections) => {
                self.was_sample_sent.store(true, Ordering::Relaxed);
                client_shared_state
                    .request_sender
                    .loan_counter
                    .fetch_sub(1, Ordering::Relaxed);
                drop(client_shared_state);
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
    fn number_of_elements(&self) -> usize {
        static_assert_size_of!(CustomPayloadMarker, 1);
        // We need to handle the custom payload marker her, that has always a size of 1
        // and the ability to set custom payload type size/alignment. Therefore, we need
        // to calculate number of elements * payload_size divided again by the payload size.
        // If the generic argument and payload size is equal it will return the actual
        // number of elements.
        //
        // But in the special case of the CustomPayloadMarker, it will divide by 1 and
        // return a slice of bytes with the correct size.
        self.header().number_of_elements() as usize
            * self
                .client_shared_state
                .lock()
                .request_sender
                .payload_size()
            / core::mem::size_of::<RequestPayload>()
    }

    /// Returns a reference to the user defined request payload.
    pub fn payload(&self) -> &[RequestPayload] {
        self.deref()
    }

    /// Returns a mutable reference to the user defined request payload.
    pub fn payload_mut(&mut self) -> &mut [RequestPayload] {
        &mut *self
    }
}
