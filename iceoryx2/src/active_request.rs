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

use core::{fmt::Debug, marker::PhantomData, mem::MaybeUninit, ops::Deref};
use std::sync::Arc;

use iceoryx2_bb_log::{error, fail};
use iceoryx2_bb_posix::unique_system_id::UniqueSystemId;
use iceoryx2_cal::zero_copy_connection::{ChannelId, ZeroCopyReceiver, ZeroCopyReleaseError};

use crate::{
    port::{
        details::chunk_details::ChunkDetails,
        port_identifiers::{UniqueClientId, UniqueServerId},
        server::SharedServerState,
        update_connections::ConnectionFailure,
        LoanError, SendError,
    },
    raw_sample::{RawSample, RawSampleMut},
    response_mut::ResponseMut,
    response_mut_uninit::ResponseMutUninit,
    service,
};

/// Represents a one-to-one connection to a [`Client`](crate::port::client::Client)
/// holding the corresponding
/// [`PendingResponse`](crate::pending_response::PendingResponse) that is coupled
/// with the [`RequestMut`](crate::request_mut::RequestMut) the
/// [`Client`](crate::port::client::Client) sent to the
/// [`Server`](crate::port::server::Server).
/// The [`Server`](crate::port::server::Server) will use it to send arbitrary many
/// [`Response`](crate::response::Response)s.
pub struct ActiveRequest<
    Service: crate::service::Service,
    RequestPayload: Debug,
    RequestHeader: Debug,
    ResponsePayload: Debug,
    ResponseHeader: Debug,
> {
    pub(crate) ptr: RawSample<
        crate::service::header::request_response::RequestHeader,
        RequestHeader,
        RequestPayload,
    >,
    pub(crate) shared_state: Arc<SharedServerState<Service>>,
    pub(crate) details: ChunkDetails<Service>,
    pub(crate) request_id: u64,
    pub(crate) channel_id: ChannelId,
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
    for ActiveRequest<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "ActiveRequest<{}, {}, {}, {}, {}> {{ details: {:?} }}",
            core::any::type_name::<Service>(),
            core::any::type_name::<RequestPayload>(),
            core::any::type_name::<RequestHeader>(),
            core::any::type_name::<ResponsePayload>(),
            core::any::type_name::<ResponseHeader>(),
            self.details
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
    for ActiveRequest<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>
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
    > Drop
    for ActiveRequest<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>
{
    fn drop(&mut self) {
        unsafe {
            self.details
                .connection
                .data_segment
                .unregister_offset(self.details.offset)
        };

        match self
            .details
            .connection
            .receiver
            .release(self.details.offset, ChannelId::new(0))
        {
            Ok(()) => (),
            Err(ZeroCopyReleaseError::RetrieveBufferFull) => {
                error!(from self, "This should never happen! The clients retrieve channel is full and the request cannot be returned.");
            }
        }
    }
}

impl<
        Service: crate::service::Service,
        RequestPayload: Debug,
        RequestHeader: Debug,
        ResponsePayload: Debug,
        ResponseHeader: Debug,
    > ActiveRequest<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>
{
    pub fn loan_uninit(
        &self,
    ) -> Result<ResponseMutUninit<Service, MaybeUninit<ResponsePayload>, ResponseHeader>, LoanError>
    {
        let chunk = self
            .shared_state
            .response_sender
            .allocate(self.shared_state.response_sender.sample_layout(1))?;

        unsafe {
            (chunk.header as *mut service::header::request_response::ResponseHeader).write(
                service::header::request_response::ResponseHeader {
                    server_port_id: UniqueServerId(UniqueSystemId::from(
                        self.shared_state.response_sender.sender_port_id,
                    )),
                    request_id: self.request_id,
                },
            )
        };

        let ptr = unsafe {
            RawSampleMut::<
                service::header::request_response::ResponseHeader,
                ResponseHeader,
                MaybeUninit<ResponsePayload>,
            >::new_unchecked(
                chunk.header.cast(),
                chunk.user_header.cast(),
                chunk.payload.cast(),
            )
        };

        Ok(ResponseMutUninit {
            response: ResponseMut {
                ptr,
                shared_state: self.shared_state.clone(),
                offset_to_chunk: chunk.offset,
                channel_id: self.channel_id,
                sample_size: chunk.size,
                _service: PhantomData,
                _response_payload: PhantomData,
                _response_header: PhantomData,
            },
        })
    }

    pub fn send_copy(&self, value: ResponsePayload) -> Result<(), SendError> {
        let msg = "Unable to send copy of response";
        let response = fail!(from self,
                            when self.loan_uninit(),
                            "{} since the loan of the response failed.", msg);

        response.write_payload(value).send()
    }

    /// Returns a reference to the payload of the received
    /// [`RequestMut`](crate::request_mut::RequestMut)
    pub fn payload(&self) -> &RequestPayload {
        self.ptr.as_payload_ref()
    }

    /// Returns a reference to the user_header of the received
    /// [`RequestMut`](crate::request_mut::RequestMut)
    pub fn user_header(&self) -> &RequestHeader {
        self.ptr.as_user_header_ref()
    }

    /// Returns a reference to the
    /// [`crate::service::header::request_response::RequestHeader`] of the received
    /// [`RequestMut`](crate::request_mut::RequestMut)
    pub fn header(&self) -> &crate::service::header::request_response::RequestHeader {
        self.ptr.as_header_ref()
    }

    /// Returns the [`UniqueClientId`] of the [`Client`](crate::port::client::Client)
    pub fn origin(&self) -> UniqueClientId {
        UniqueClientId(UniqueSystemId::from(self.details.origin))
    }
}

impl<
        Service: crate::service::Service,
        RequestPayload: Debug,
        RequestHeader: Debug,
        ResponsePayload: Debug + Default,
        ResponseHeader: Debug,
    > ActiveRequest<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>
{
    pub fn loan(&self) -> Result<ResponseMut<Service, ResponsePayload, ResponseHeader>, LoanError> {
        Ok(self
            .loan_uninit()?
            .write_payload(ResponsePayload::default()))
    }
}
