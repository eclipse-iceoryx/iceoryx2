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
//! # let service = node.service_builder(&"ResponseMutExample1".try_into()?)
//! #     .request_response::<u64, u64>()
//! #     .open_or_create()?;
//! #
//! # let client = service.client_builder().create()?;
//! # let server = service.server_builder().create()?;
//! # let pending_response = client.send_copy(0)?;
//! # let active_request = server.receive()?.unwrap();
//!
//! let mut response = active_request.loan()?;
//! // write 456 because its fun
//! *response.payload_mut() = 456;
//!
//! println!("server id: {:?}", response.header().server_id());
//! response.send()?;
//!
//! # Ok(())
//! # }
//! ```

use alloc::sync::Arc;
use core::{
    fmt::Debug,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use iceoryx2_bb_concurrency::atomic::AtomicUsize;
use iceoryx2_bb_concurrency::atomic::Ordering;
use iceoryx2_bb_elementary_traits::{iceoryx_send::IceoryxSend, zero_copy_send::ZeroCopySend};
use iceoryx2_cal::zero_copy_connection::ChannelId;
use iceoryx2_log::fail;

use crate::{
    payload::number_of_elements,
    port::{
        SendError,
        details::{chunk::ChunkMut, chunk_mut_shared_state::ChunkMutSharedState},
        server::{INVALID_CONNECTION_ID, SharedServerState},
    },
    service,
};

/// Acquired by a [`ActiveRequest`](crate::active_request::ActiveRequest) with
///  * [`ActiveRequest::loan()`](crate::active_request::ActiveRequest::loan())
///
/// It stores the payload of the response that will be sent to the corresponding
/// [`PendingResponse`](crate::pending_response::PendingResponse) of the
/// [`Client`](crate::port::client::Client).
///
/// If the [`ResponseMut`] is not sent it will reelase the loaned memory when going out of
/// scope.
pub struct ResponseMut<
    Service: service::Service,
    ResponsePayload: Debug + IceoryxSend + ?Sized,
    ResponseHeader: Debug + ZeroCopySend,
> {
    pub(crate) shared_state: ChunkMutSharedState<Service, SharedServerState<Service>>,
    pub(crate) shared_loan_counter: Arc<AtomicUsize>,
    pub(crate) chunk: ChunkMut,
    pub(crate) channel_id: ChannelId,
    pub(crate) connection_id: usize,
    pub(crate) _response_payload: PhantomData<ResponsePayload>,
    pub(crate) _response_header: PhantomData<ResponseHeader>,
}

unsafe impl<
    Service: crate::service::Service,
    ResponsePayload: Debug + IceoryxSend + ?Sized,
    ResponseHeader: Debug + ZeroCopySend,
> Send for ResponseMut<Service, ResponsePayload, ResponseHeader>
where
    Service::ArcThreadSafetyPolicy<SharedServerState<Service>>: Send + Sync,
{
}

impl<
    Service: crate::service::Service,
    ResponsePayload: Debug + IceoryxSend + ?Sized,
    ResponseHeader: Debug + ZeroCopySend,
> Debug for ResponseMut<Service, ResponsePayload, ResponseHeader>
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "ResponseMut<{}, {}, {}> {{ chunk: {:?}, channel_id: {} }}",
            core::any::type_name::<Service>(),
            core::any::type_name::<ResponsePayload>(),
            core::any::type_name::<ResponseHeader>(),
            self.chunk,
            self.channel_id.value()
        )
    }
}

impl<
    Service: crate::service::Service,
    ResponsePayload: Debug + IceoryxSend + ?Sized,
    ResponseHeader: Debug + ZeroCopySend,
> Drop for ResponseMut<Service, ResponsePayload, ResponseHeader>
{
    fn drop(&mut self) {
        self.shared_loan_counter.fetch_sub(1, Ordering::Relaxed);
    }
}

impl<
    Service: crate::service::Service,
    ResponsePayload: Debug + ZeroCopySend,
    ResponseHeader: Debug + ZeroCopySend,
> Deref for ResponseMut<Service, ResponsePayload, ResponseHeader>
{
    type Target = ResponsePayload;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.chunk.payload_ptr().cast() }
    }
}

impl<
    Service: crate::service::Service,
    ResponsePayload: IceoryxSend + Debug + ZeroCopySend,
    ResponseHeader: Debug + ZeroCopySend,
> Deref for ResponseMut<Service, [ResponsePayload], ResponseHeader>
{
    type Target = [ResponsePayload];
    fn deref(&self) -> &Self::Target {
        let payload_size = self.shared_state.payload_size();
        unsafe {
            &*core::ptr::slice_from_raw_parts(
                self.chunk.payload_ptr().cast(),
                number_of_elements::<ResponsePayload, _>(self.header(), payload_size),
            )
        }
    }
}

impl<
    Service: crate::service::Service,
    ResponsePayload: IceoryxSend + Debug + ZeroCopySend,
    ResponseHeader: Debug + ZeroCopySend,
> DerefMut for ResponseMut<Service, ResponsePayload, ResponseHeader>
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.chunk.payload_mut_ptr().cast() }
    }
}

impl<
    Service: crate::service::Service,
    ResponsePayload: IceoryxSend + Debug + ZeroCopySend,
    ResponseHeader: Debug + ZeroCopySend,
> DerefMut for ResponseMut<Service, [ResponsePayload], ResponseHeader>
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        let payload_size = self.shared_state.payload_size();
        unsafe {
            &mut *core::ptr::slice_from_raw_parts_mut(
                self.chunk.payload_mut_ptr().cast(),
                number_of_elements::<ResponsePayload, _>(self.header(), payload_size),
            )
        }
    }
}

impl<
    Service: crate::service::Service,
    ResponsePayload: Debug + IceoryxSend + ?Sized,
    ResponseHeader: Debug + ZeroCopySend,
> ResponseMut<Service, ResponsePayload, ResponseHeader>
{
    /// Returns a reference to the
    /// [`ResponseHeader`](service::header::request_response::ResponseHeader).
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// # let service = node.service_builder(&"ResponseMutExample2".try_into()?)
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
    /// println!("server id: {:?}", response.header().server_id());
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn header(&self) -> &service::header::request_response::ResponseHeader {
        unsafe { &*self.chunk.header_ptr().cast() }
    }

    /// Returns a reference to the user header of the response.
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// # let service = node.service_builder(&"ResponseMutExample3".try_into()?)
    /// #     .request_response::<u64, u64>()
    /// #     .response_user_header::<u64>()
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
        unsafe { &*self.chunk.user_header_ptr().cast() }
    }

    /// Returns a mutable reference to the user header of the response.
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// # let service = node.service_builder(&"ResponseMutExample4".try_into()?)
    /// #     .request_response::<u64, u64>()
    /// #     .response_user_header::<u64>()
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
        unsafe { &mut *self.chunk.user_header_mut_ptr().cast() }
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
    /// # let service = node.service_builder(&"ResponseMutExample6".try_into()?)
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
        self.shared_state.call(|s| {
            let msg = "Unable to send response";
            fail!(from self, when s.update_connections(),
                "{} since the connections could not be updated.", msg);

            if self.connection_id != INVALID_CONNECTION_ID {
                s.response_sender.deliver_offset_to_connection(
                    &self.chunk,
                    self.channel_id,
                    self.connection_id,
                )?;
            }

            Ok(())
        })
    }
}

impl<
    Service: crate::service::Service,
    ResponsePayload: Debug + ZeroCopySend,
    ResponseHeader: Debug + ZeroCopySend,
> ResponseMut<Service, ResponsePayload, ResponseHeader>
{
    /// Returns a reference to the payload of the response.
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// # let service = node.service_builder(&"ResponseMutExample4".try_into()?)
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
        self.deref()
    }

    /// Returns a mutable reference to the payload of the response.
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// # let service = node.service_builder(&"ResponseMutExample5".try_into()?)
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
        &mut *self
    }
}

impl<
    Service: crate::service::Service,
    ResponsePayload: Debug + ZeroCopySend,
    ResponseHeader: Debug + ZeroCopySend,
> ResponseMut<Service, [ResponsePayload], ResponseHeader>
{
    /// Returns a reference to the payload of the response.
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// # let service = node.service_builder(&"ResponseMutExample4".try_into()?)
    /// #     .request_response::<u64, [u64]>()
    /// #     .open_or_create()?;
    /// #
    /// # let client = service.client_builder().create()?;
    /// # let server = service.server_builder().create()?;
    /// # let pending_response = client.send_copy(0)?;
    /// # let active_request = server.receive()?.unwrap();
    ///
    /// // initializes the payload with default, therefore it is okay to access
    /// // it without assigning something first
    /// let mut response = active_request.loan_slice(12)?;
    /// println!("default payload {}", *response.payload());
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn payload(&self) -> &[ResponsePayload] {
        self.deref()
    }

    /// Returns a mutable reference to the payload of the response.
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// # let service = node.service_builder(&"ResponseMutExample5".try_into()?)
    /// #     .request_response::<u64, [u64]>()
    /// #     .open_or_create()?;
    /// #
    /// # let client = service.client_builder().create()?;
    /// # let server = service.server_builder().create()?;
    /// # let pending_response = client.send_copy(0)?;
    /// # let active_request = server.receive()?.unwrap();
    ///
    /// let mut response = active_request.loan_slice(12)?;
    /// response.payload_mut()[1] = 123;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn payload_mut(&mut self) -> &mut [ResponsePayload] {
        &mut *self
    }
}
