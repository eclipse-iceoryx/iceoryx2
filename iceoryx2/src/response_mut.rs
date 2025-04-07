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

use core::{
    fmt::Debug,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    sync::atomic::Ordering,
};
use std::sync::Arc;

use iceoryx2_bb_log::fail;
use iceoryx2_cal::{shm_allocator::PointerOffset, zero_copy_connection::ChannelId};

use crate::{
    port::{server::SharedServerState, SendError},
    raw_sample::RawSampleMut,
    service,
};

pub struct ResponseMut<Service: service::Service, ResponsePayload: Debug, ResponseHeader: Debug> {
    pub(crate) ptr: RawSampleMut<
        service::header::request_response::ResponseHeader,
        ResponseHeader,
        ResponsePayload,
    >,
    pub(crate) shared_state: Arc<SharedServerState<Service>>,
    pub(crate) offset_to_chunk: PointerOffset,
    pub(crate) sample_size: usize,
    pub(crate) channel_id: ChannelId,
    pub(crate) _service: PhantomData<Service>,
    pub(crate) _response_payload: PhantomData<ResponsePayload>,
    pub(crate) _response_header: PhantomData<ResponseHeader>,
}

impl<Service: crate::service::Service, ResponsePayload: Debug, ResponseHeader: Debug> Debug
    for ResponseMut<Service, ResponsePayload, ResponseHeader>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ResponseMut<{}, {}, {}> {{ ptr: {:?}, offset_to_chunk: {:?}, sample_size: {}, channel_id: {} }}",
            core::any::type_name::<Service>(),
            core::any::type_name::<ResponsePayload>(),
            core::any::type_name::<ResponseHeader>(),
            self.ptr,
            self.offset_to_chunk,
            self.sample_size,
            self.channel_id.value()
        )
    }
}

impl<Service: crate::service::Service, ResponsePayload: Debug, ResponseHeader: Debug> Drop
    for ResponseMut<Service, ResponsePayload, ResponseHeader>
{
    fn drop(&mut self) {
        self.shared_state
            .response_sender
            .return_loaned_sample(self.offset_to_chunk);
    }
}

impl<Service: crate::service::Service, ResponsePayload: Debug, ResponseHeader: Debug> Deref
    for ResponseMut<Service, ResponsePayload, ResponseHeader>
{
    type Target = ResponsePayload;
    fn deref(&self) -> &Self::Target {
        self.ptr.as_payload_ref()
    }
}

impl<Service: crate::service::Service, ResponsePayload: Debug, ResponseHeader: Debug> DerefMut
    for ResponseMut<Service, ResponsePayload, ResponseHeader>
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.ptr.as_payload_mut()
    }
}

impl<Service: crate::service::Service, ResponsePayload: Debug, ResponseHeader: Debug>
    ResponseMut<Service, ResponsePayload, ResponseHeader>
{
    pub fn header(&self) -> &service::header::request_response::ResponseHeader {
        self.ptr.as_header_ref()
    }

    pub fn user_header(&self) -> &ResponseHeader {
        self.ptr.as_user_header_ref()
    }

    pub fn user_header_mut(&mut self) -> &mut ResponseHeader {
        self.ptr.as_user_header_mut()
    }

    pub fn payload(&self) -> &ResponsePayload {
        self.ptr.as_payload_ref()
    }

    pub fn payload_mut(&mut self) -> &mut ResponsePayload {
        self.ptr.as_payload_mut()
    }

    pub fn send(self) -> Result<(), SendError> {
        let msg = "Unable to send response";

        if !self.shared_state.is_active.load(Ordering::Relaxed) {
            fail!(from self, with SendError::ConnectionBrokenSinceSenderNoLongerExists,
                "{} since the corresponding server is already disconnected.", msg);
        }

        self.shared_state.response_sender.deliver_offset(
            self.offset_to_chunk,
            self.sample_size,
            self.channel_id,
        )?;

        Ok(())
    }
}
