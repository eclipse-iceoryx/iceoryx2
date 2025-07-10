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

use core::{fmt::Debug, marker::PhantomData};
use core::{
    ops::{Deref, DerefMut},
    sync::atomic::Ordering,
};
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use iceoryx2_bb_log::fatal_panic;
use iceoryx2_cal::arc_sync_policy::ArcSyncPolicy;
use iceoryx2_cal::zero_copy_connection::ChannelId;

use iceoryx2_cal::shm_allocator::PointerOffset;
use iceoryx2_pal_concurrency_sync::iox_atomic::IoxAtomicBool;

use crate::{
    pending_response::PendingResponse,
    port::client::{ClientSharedState, RequestSendError},
    raw_sample::RawSampleMut,
    service,
};

/// The [`RequestMut`] represents the object that contains the payload that the
/// [`Client`](crate::port::client::Client) sends to the
/// [`Server`](crate::port::server::Server).
pub struct RequestMut<
    Service: crate::service::Service,
    RequestPayload: Debug + ZeroCopySend + ?Sized,
    RequestHeader: Debug + ZeroCopySend,
    ResponsePayload: Debug + ZeroCopySend + ?Sized,
    ResponseHeader: Debug + ZeroCopySend,
> {
    pub(crate) ptr: RawSampleMut<
        service::header::request_response::RequestHeader,
        RequestHeader,
        RequestPayload,
    >,
    pub(crate) sample_size: usize,
    pub(crate) offset_to_chunk: PointerOffset,
    pub(crate) client_shared_state: Service::ArcThreadSafetyPolicy<ClientSharedState<Service>>,
    pub(crate) was_sample_sent: IoxAtomicBool,
    pub(crate) channel_id: ChannelId,
    pub(crate) _response_payload: PhantomData<ResponsePayload>,
    pub(crate) _response_header: PhantomData<ResponseHeader>,
}

unsafe impl<
        Service: crate::service::Service,
        RequestPayload: Debug + ZeroCopySend + ?Sized,
        RequestHeader: Debug + ZeroCopySend,
        ResponsePayload: Debug + ZeroCopySend + ?Sized,
        ResponseHeader: Debug + ZeroCopySend,
    > Send for RequestMut<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>
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
            .release_sample(self.offset_to_chunk);
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
        RequestPayload: Debug + ZeroCopySend + ?Sized,
        RequestHeader: Debug + ZeroCopySend,
        ResponsePayload: Debug + ZeroCopySend + ?Sized,
        ResponseHeader: Debug + ZeroCopySend,
    > Debug
    for RequestMut<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "RequestMut<{}, {}, {}, {}, {}> {{ ptr: {:?}, sample_size: {}, offset_to_chunk: {:?}, was_sample_sent: {}, channel_id: {} }}",
            core::any::type_name::<Service>(),
            core::any::type_name::<RequestPayload>(),
            core::any::type_name::<RequestHeader>(),
            core::any::type_name::<ResponsePayload>(),
            core::any::type_name::<ResponseHeader>(),
            self.ptr,
            self.sample_size,
            self.offset_to_chunk,
            self.was_sample_sent.load(Ordering::Relaxed),
            self.channel_id.value()
        )
    }
}

impl<
        Service: crate::service::Service,
        RequestPayload: Debug + ZeroCopySend + ?Sized,
        RequestHeader: Debug + ZeroCopySend,
        ResponsePayload: Debug + ZeroCopySend + ?Sized,
        ResponseHeader: Debug + ZeroCopySend,
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
        RequestPayload: Debug + ZeroCopySend + ?Sized,
        RequestHeader: Debug + ZeroCopySend,
        ResponsePayload: Debug + ZeroCopySend + ?Sized,
        ResponseHeader: Debug + ZeroCopySend,
    > DerefMut
    for RequestMut<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.ptr.as_payload_mut()
    }
}

impl<
        Service: crate::service::Service,
        RequestPayload: Debug + ZeroCopySend + ?Sized,
        RequestHeader: Debug + ZeroCopySend,
        ResponsePayload: Debug + ZeroCopySend + ?Sized,
        ResponseHeader: Debug + ZeroCopySend,
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
        let client_shared_state = self.client_shared_state.lock();
        match client_shared_state.send_request(
            self.offset_to_chunk,
            self.sample_size,
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
