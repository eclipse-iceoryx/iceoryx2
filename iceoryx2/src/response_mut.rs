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
//! # let client = service.client_builder().create()?;
//! # let server = service.server_builder().create()?;
//! # let pending_response = client.send_copy(0)?;
//! # let active_request = server.receive()?.unwrap();
//!
//! let response = active_request.loan()?;
//! // write 456 because its fun
//! *response.payload_mut() = 456;
//!
//! println!("server port id: {:?}", response.header().server_id());
//! response.send()?;
//!
//! # Ok(())
//! # }
//! ```

use core::{
    fmt::Debug,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};
use std::sync::Arc;

use iceoryx2_bb_log::fail;
use iceoryx2_cal::{shm_allocator::PointerOffset, zero_copy_connection::ChannelId};

use crate::{
    port::{server::SharedServerState, SendError},
    raw_sample::RawSampleMut,
    service,
};

/// Acquired by a [`ActiveRequest`](crate::active_request::ActiveRequest) with
///  * [`ActiveRequest::loan_uninit()`](crate::active_request::ActiveRequest::loan_uninit())
///
/// It stores the payload of the response that will be sent to the corresponding
/// [`PendingResponse`](crate::pending_response::PendingResponse) of the
/// [`Client`](crate::port::client::Client).
///
/// If the [`ResponseMut`] is not sent it will reelase the loaned memory when going out of
/// scope.
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
    pub(crate) connection_id: usize,
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
    /// Returns a reference to the header of the response.
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// # let service = node.service_builder(&"My/Funk/ServiceName".try_into()?)
    /// #     .request_response::<u64, u64>()
    /// #     .open_or_create()?;
    /// #
    /// # let client = service.client_builder().create()?;
    /// # let server = service.server_builder().create()?;
    /// # let pending_response = client.send_copy(0)?;
    /// # let active_request = server.receive()?.unwrap();
    ///
    /// let response = active_request.loan()?;
    ///
    /// println!("server port id: {:?}", response.header().server_id());
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn header(&self) -> &service::header::request_response::ResponseHeader {
        self.ptr.as_header_ref()
    }

    /// Returns a reference to the user header of the response.
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// # let service = node.service_builder(&"My/Funk/ServiceName".try_into()?)
    /// #     .request_response::<u64, u64>()
    /// #     .response_header::<u64>()
    /// #     .open_or_create()?;
    /// #
    /// # let client = service.client_builder().create()?;
    /// # let server = service.server_builder().create()?;
    /// # let pending_response = client.send_copy(0)?;
    /// # let active_request = server.receive()?.unwrap();
    ///
    /// // initializes the user header with default, therefore it is okay to access
    /// // it without assigning something first
    /// let mut response = active_request.loan()?;
    /// println!("user header {}", response.user_header());
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn user_header(&self) -> &ResponseHeader {
        self.ptr.as_user_header_ref()
    }

    /// Returns a mutable reference to the user header of the response.
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// # let service = node.service_builder(&"My/Funk/ServiceName".try_into()?)
    /// #     .request_response::<u64, u64>()
    /// #     .response_header::<u64>()
    /// #     .open_or_create()?;
    /// #
    /// # let client = service.client_builder().create()?;
    /// # let server = service.server_builder().create()?;
    /// # let pending_response = client.send_copy(0)?;
    /// # let active_request = server.receive()?.unwrap();
    ///
    /// let mut response = active_request.loan()?;
    /// *response.user_header_mut() = 123;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn user_header_mut(&mut self) -> &mut ResponseHeader {
        self.ptr.as_user_header_mut()
    }

    /// Returns a reference to the payload of the response.
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// # let service = node.service_builder(&"My/Funk/ServiceName".try_into()?)
    /// #     .request_response::<u64, u64>()
    /// #     .open_or_create()?;
    /// #
    /// # let client = service.client_builder().create()?;
    /// # let server = service.server_builder().create()?;
    /// # let pending_response = client.send_copy(0)?;
    /// # let active_request = server.receive()?.unwrap();
    ///
    /// // initializes the payload with default, therefore it is okay to access
    /// // it without assigning something first
    /// let mut response = active_request.loan()?;
    /// println!("default payload {}", *response.payload());
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn payload(&self) -> &ResponsePayload {
        self.ptr.as_payload_ref()
    }

    /// Returns a reference to the payload of the response.
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// # let service = node.service_builder(&"My/Funk/ServiceName".try_into()?)
    /// #     .request_response::<u64, u64>()
    /// #     .open_or_create()?;
    /// #
    /// # let client = service.client_builder().create()?;
    /// # let server = service.server_builder().create()?;
    /// # let pending_response = client.send_copy(0)?;
    /// # let active_request = server.receive()?.unwrap();
    ///
    /// let mut response = active_request.loan()?;
    /// *response.payload_mut() = 123;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn payload_mut(&mut self) -> &mut ResponsePayload {
        self.ptr.as_payload_mut()
    }

    /// Sends a [`ResponseMut`] to the corresponding
    /// [`PendingResponse`](crate::pending_response::PendingResponse) of the
    /// [`Client`](crate::port::client::Client).
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// # let service = node.service_builder(&"My/Funk/ServiceName".try_into()?)
    /// #     .request_response::<u64, u64>()
    /// #     .open_or_create()?;
    /// #
    /// # let client = service.client_builder().create()?;
    /// # let server = service.server_builder().create()?;
    /// # let pending_response = client.send_copy(0)?;
    /// # let active_request = server.receive()?.unwrap();
    ///
    /// let mut response = active_request.loan()?;
    /// *response.payload_mut() = 456;
    /// response.send()?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn send(self) -> Result<(), SendError> {
        let msg = "Unable to send response";

        fail!(from self, when self.shared_state.update_connections(),
            "{} since the connections could not be updated.", msg);

        self.shared_state
            .response_sender
            .deliver_offset_to_connection(
                self.offset_to_chunk,
                self.sample_size,
                self.channel_id,
                self.connection_id,
            )?;

        Ok(())
    }
}
