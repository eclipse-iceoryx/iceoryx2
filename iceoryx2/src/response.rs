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
//! ## Typed API
//!
//! ```
//! use iceoryx2::prelude::*;
//! # fn main() -> Result<(), Box<dyn core::error::Error>> {
//! # let node = NodeBuilder::new().create::<ipc::Service>()?;
//! #
//! # let service = node.service_builder(&"My/Funk/ServiceName".try_into()?)
//! #     .request_response::<u64, u64>()
//! #     .open_or_create()?;
//! #
//! let client = service.client_builder().create()?;
//! # let server = service.server_builder().create()?;
//! let pending_response = client.send_copy(0)?;
//! # let active_request = server.receive()?.unwrap();
//! # active_request.send_copy(0)?;
//!
//! if let Some(response) = pending_response.receive()? {
//!     println!("received response: {} from: {:?}",
//!         response.payload(), response.header().server_id());
//! }
//!
//! # Ok(())
//! # }
//! ```

use core::fmt::Debug;
use core::ops::Deref;

use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use iceoryx2_bb_posix::unique_system_id::UniqueSystemId;
use iceoryx2_cal::arc_sync_policy::ArcSyncPolicy;
use iceoryx2_cal::zero_copy_connection::ChannelId;

use crate::port::client::ClientSharedState;
use crate::port::details::chunk_details::ChunkDetails;
use crate::port::port_identifiers::UniqueServerId;
use crate::raw_sample::RawSample;
use crate::service;

/// It stores the payload and can be received by the
/// [`PendingResponse`](crate::pending_response::PendingResponse) after a
/// [`RequestMut`](crate::request_mut::RequestMut) was sent to a
/// [`Server`](crate::port::server::Server) via the [`Client`](crate::port::client::Client).
pub struct Response<
    Service: crate::service::Service,
    ResponsePayload: Debug + ZeroCopySend + ?Sized,
    ResponseHeader: Debug + ZeroCopySend,
> {
    pub(crate) ptr: RawSample<
        crate::service::header::request_response::ResponseHeader,
        ResponseHeader,
        ResponsePayload,
    >,
    pub(crate) client_shared_state: Service::ArcThreadSafetyPolicy<ClientSharedState<Service>>,
    pub(crate) details: ChunkDetails,
    pub(crate) channel_id: ChannelId,
}

unsafe impl<
        Service: crate::service::Service,
        ResponsePayload: Debug + ZeroCopySend + ?Sized,
        ResponseHeader: Debug + ZeroCopySend,
    > Send for Response<Service, ResponsePayload, ResponseHeader>
where
    Service::ArcThreadSafetyPolicy<ClientSharedState<Service>>: Send + Sync,
{
}

impl<
        Service: crate::service::Service,
        ResponsePayload: Debug + ZeroCopySend + ?Sized,
        ResponseHeader: Debug + ZeroCopySend,
    > Drop for Response<Service, ResponsePayload, ResponseHeader>
{
    fn drop(&mut self) {
        self.client_shared_state
            .lock()
            .response_receiver
            .release_offset(&self.details, self.channel_id);
    }
}

impl<
        Service: crate::service::Service,
        ResponsePayload: Debug + ZeroCopySend + ?Sized,
        ResponseHeader: Debug + ZeroCopySend,
    > Debug for Response<Service, ResponsePayload, ResponseHeader>
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

impl<
        Service: crate::service::Service,
        ResponsePayload: Debug + ZeroCopySend + ?Sized,
        ResponseHeader: Debug + ZeroCopySend,
    > Deref for Response<Service, ResponsePayload, ResponseHeader>
{
    type Target = ResponsePayload;
    fn deref(&self) -> &Self::Target {
        self.ptr.as_payload_ref()
    }
}

impl<
        Service: crate::service::Service,
        ResponsePayload: Debug + ZeroCopySend + ?Sized,
        ResponseHeader: Debug + ZeroCopySend,
    > Response<Service, ResponsePayload, ResponseHeader>
{
    /// Returns a reference to the
    /// [`ResponseHeader`](service::header::request_response::ResponseHeader).
    pub fn header(&self) -> &service::header::request_response::ResponseHeader {
        self.ptr.as_header_ref()
    }

    /// Returns a reference to the user header of the response.
    pub fn user_header(&self) -> &ResponseHeader {
        self.ptr.as_user_header_ref()
    }

    /// Returns a reference to the payload of the response.
    pub fn payload(&self) -> &ResponsePayload {
        self.ptr.as_payload_ref()
    }

    /// Returns the [`UniqueServerId`] of the [`Server`](crate::port::server::Server) which sent
    /// the [`Response`].
    pub fn origin(&self) -> UniqueServerId {
        UniqueServerId(UniqueSystemId::from(self.details.origin))
    }
}
