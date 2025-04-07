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

use core::fmt::Debug;
use core::ops::Deref;

use iceoryx2_bb_log::error;
use iceoryx2_bb_posix::unique_system_id::UniqueSystemId;
use iceoryx2_cal::zero_copy_connection::{ChannelId, ZeroCopyReceiver, ZeroCopyReleaseError};

use crate::port::details::chunk_details::ChunkDetails;
use crate::port::port_identifiers::UniqueServerId;
use crate::raw_sample::RawSample;
use crate::service;

pub struct Response<Service: crate::service::Service, ResponsePayload: Debug, ResponseHeader: Debug>
{
    pub(crate) ptr: RawSample<
        crate::service::header::request_response::ResponseHeader,
        ResponseHeader,
        ResponsePayload,
    >,
    pub(crate) details: ChunkDetails<Service>,
    pub(crate) channel_id: ChannelId,
}

impl<Service: crate::service::Service, ResponsePayload: Debug, ResponseHeader: Debug> Drop
    for Response<Service, ResponsePayload, ResponseHeader>
{
    fn drop(&mut self) {
        unsafe {
            self.details
                .connection
                .data_segment
                .unregister_offset(self.details.offset);
        }

        match self
            .details
            .connection
            .receiver
            .release(self.details.offset, self.channel_id)
        {
            Ok(()) => (),
            Err(ZeroCopyReleaseError::RetrieveBufferFull) => {
                error!(from self, "This should never happen! The servers retrieve channel is full and the response cannot be returned.");
            }
        }
    }
}

impl<Service: crate::service::Service, ResponsePayload: Debug, ResponseHeader: Debug> Debug
    for Response<Service, ResponsePayload, ResponseHeader>
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Response<{}, {}, {}> {{ ptr: {:?} }}",
            core::any::type_name::<Service>(),
            core::any::type_name::<ResponsePayload>(),
            core::any::type_name::<ResponseHeader>(),
            self.ptr
        )
    }
}

impl<Service: crate::service::Service, ResponsePayload: Debug, ResponseHeader: Debug> Deref
    for Response<Service, ResponsePayload, ResponseHeader>
{
    type Target = ResponsePayload;
    fn deref(&self) -> &Self::Target {
        self.ptr.as_payload_ref()
    }
}

impl<Service: crate::service::Service, ResponsePayload: Debug, ResponseHeader: Debug>
    Response<Service, ResponsePayload, ResponseHeader>
{
    pub fn header(&self) -> &service::header::request_response::ResponseHeader {
        self.ptr.as_header_ref()
    }

    pub fn user_header(&self) -> &ResponseHeader {
        self.ptr.as_user_header_ref()
    }

    pub fn payload(&self) -> &ResponsePayload {
        self.ptr.as_payload_ref()
    }

    pub fn origin(&self) -> UniqueServerId {
        UniqueServerId(UniqueSystemId::from(self.details.origin))
    }
}
