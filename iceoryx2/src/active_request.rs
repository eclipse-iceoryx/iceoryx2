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
//! ```
//! use iceoryx2::prelude::*;
//!
//! # fn main() -> Result<(), Box<dyn core::error::Error>> {
//! # let node = NodeBuilder::new().create::<ipc::Service>()?;
//! # let service = node
//! #     .service_builder(&"My/Funk/ServiceName".try_into()?)
//! #     .request_response::<u64, u64>()
//! #     .open_or_create()?;
//! # let client = service.client_builder().create()?;
//! # let server = service.server_builder().create()?;
//! #
//! # client.send_copy(123)?;
//!
//! let active_request = server.receive()?.unwrap();
//!
//! // send a stream of responses until the corresponding client
//! // lets the pending response go out-of-scope and signaling that there is no more interest
//! // in further responses
//! while active_request.is_connected() {
//!     let response = active_request.loan_uninit()?;
//!     response.write_payload(456).send()?;
//! }
//!
//! # Ok(())
//! # }
//! ```

use alloc::sync::Arc;
use core::{
    any::TypeId, fmt::Debug, marker::PhantomData, mem::MaybeUninit, ops::Deref,
    sync::atomic::Ordering,
};

use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use iceoryx2_bb_log::fail;
use iceoryx2_bb_posix::unique_system_id::UniqueSystemId;
use iceoryx2_cal::{
    arc_sync_policy::ArcSyncPolicy, shm_allocator::AllocationStrategy,
    zero_copy_connection::ChannelId,
};
use iceoryx2_pal_concurrency_sync::iox_atomic::IoxAtomicUsize;

use crate::{
    port::{
        details::chunk_details::ChunkDetails,
        port_identifiers::{UniqueClientId, UniqueServerId},
        server::{SharedServerState, INVALID_CONNECTION_ID},
        LoanError, SendError,
    },
    raw_sample::{RawSample, RawSampleMut},
    response_mut::ResponseMut,
    response_mut_uninit::ResponseMutUninit,
    service::{
        self,
        builder::{CustomHeaderMarker, CustomPayloadMarker},
        static_config::message_type_details::TypeVariant,
    },
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
    RequestPayload: Debug + ZeroCopySend + ?Sized,
    RequestHeader: Debug + ZeroCopySend,
    ResponsePayload: Debug + ZeroCopySend + ?Sized,
    ResponseHeader: Debug + ZeroCopySend,
> {
    pub(crate) ptr: RawSample<
        crate::service::header::request_response::RequestHeader,
        RequestHeader,
        RequestPayload,
    >,
    pub(crate) shared_state: Service::ArcThreadSafetyPolicy<SharedServerState<Service>>,
    pub(crate) shared_loan_counter: Arc<IoxAtomicUsize>,
    pub(crate) max_loan_count: usize,
    pub(crate) details: ChunkDetails,
    pub(crate) request_id: u64,
    pub(crate) channel_id: ChannelId,
    pub(crate) connection_id: usize,
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
    for ActiveRequest<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>
where
    Service::ArcThreadSafetyPolicy<SharedServerState<Service>>: Send + Sync,
{
}

impl<
        Service: crate::service::Service,
        RequestPayload: Debug + ZeroCopySend + ?Sized,
        RequestHeader: Debug + ZeroCopySend,
        ResponsePayload: Debug + ZeroCopySend + ?Sized,
        ResponseHeader: Debug + ZeroCopySend,
    > Debug
    for ActiveRequest<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "ActiveRequest<{}, {}, {}, {}, {}> {{ details: {:?}, request_id: {}, channel_id: {} }}",
            core::any::type_name::<Service>(),
            core::any::type_name::<RequestPayload>(),
            core::any::type_name::<RequestHeader>(),
            core::any::type_name::<ResponsePayload>(),
            core::any::type_name::<ResponseHeader>(),
            self.details,
            self.request_id,
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
    for ActiveRequest<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>
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
    > Drop
    for ActiveRequest<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>
{
    fn drop(&mut self) {
        self.shared_state
            .lock()
            .request_receiver
            .release_offset(&self.details, ChannelId::new(0));
        self.finish();
    }
}

impl<
        Service: crate::service::Service,
        RequestPayload: Debug + ZeroCopySend + ?Sized,
        RequestHeader: Debug + ZeroCopySend,
        ResponsePayload: Debug + ZeroCopySend + ?Sized,
        ResponseHeader: Debug + ZeroCopySend,
    > ActiveRequest<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>
{
    fn finish(&self) {
        if self.connection_id != INVALID_CONNECTION_ID {
            self.shared_state
                .lock()
                .response_sender
                .invalidate_channel_state(self.channel_id, self.connection_id, self.request_id);
        }
    }

    /// Returns [`true`] if the [`Client`](crate::port::client::Client) wants to gracefully disconnect.
    /// This allows the [`Server`](crate::port::server::Server) to send its last response and then
    /// drop the [`ActiveRequest`] to signal the [`Client`](crate::port::client::Client) that no more
    /// [`ResponseMut`] will be sent.
    pub fn has_disconnect_hint(&self) -> bool {
        if self.connection_id != INVALID_CONNECTION_ID {
            self.shared_state
                .lock()
                .response_sender
                .has_disconnect_hint(self.channel_id, self.connection_id, self.request_id)
        } else {
            false
        }
    }

    /// Returns [`true`] until the [`PendingResponse`](crate::pending_response::PendingResponse)
    /// goes out of scope on the [`Client`](crate::port::client::Client)s side indicating that the
    /// [`Client`](crate::port::client::Client) no longer receives the [`ResponseMut`].
    pub fn is_connected(&self) -> bool {
        if self.connection_id != INVALID_CONNECTION_ID {
            self.shared_state.lock().response_sender.has_channel_state(
                self.channel_id,
                self.connection_id,
                self.request_id,
            )
        } else {
            false
        }
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

    fn increment_loan_counter(&self) -> Result<(), LoanError> {
        let mut current_loan_count = self.shared_loan_counter.load(Ordering::Relaxed);
        loop {
            if self.max_loan_count <= current_loan_count {
                fail!(from self,
                with LoanError::ExceedsMaxLoans,
                "Unable to loan memory for Response since it would exceed the maximum number of loans of {}.",
                self.max_loan_count);
            }

            match self.shared_loan_counter.compare_exchange(
                current_loan_count,
                current_loan_count + 1,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => return Ok(()),
                Err(v) => current_loan_count = v,
            }
        }
    }
}

////////////////////////
// BEGIN: typed API
////////////////////////
impl<
        Service: crate::service::Service,
        RequestPayload: Debug + ZeroCopySend + ?Sized,
        RequestHeader: Debug + ZeroCopySend,
        ResponsePayload: Debug + ZeroCopySend + Sized,
        ResponseHeader: Default + Debug + ZeroCopySend,
    > ActiveRequest<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>
{
    /// Loans uninitialized memory for a [`ResponseMut`] where the user can write its payload to.
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    ///
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// # let service = node
    /// #     .service_builder(&"My/Funk/ServiceName".try_into()?)
    /// #     .request_response::<u64, u64>()
    /// #     .open_or_create()?;
    /// # let client = service.client_builder().create()?;
    /// # let server = service.server_builder().create()?;
    /// #
    /// # let pending_response = client.send_copy(123)?;
    ///
    /// let active_request = server.receive()?.unwrap();
    /// let response = active_request.loan_uninit()?;
    /// response.write_payload(456).send()?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn loan_uninit(
        &self,
    ) -> Result<ResponseMutUninit<Service, MaybeUninit<ResponsePayload>, ResponseHeader>, LoanError>
    {
        self.increment_loan_counter()?;
        let shared_state = self.shared_state.lock();

        let chunk = shared_state
            .response_sender
            .allocate(shared_state.response_sender.sample_layout(1))?;

        let header_ptr: *mut service::header::request_response::ResponseHeader =
            chunk.header.cast();
        let user_header_ptr: *mut ResponseHeader = chunk.user_header.cast();
        unsafe {
            header_ptr.write(service::header::request_response::ResponseHeader {
                server_id: UniqueServerId(UniqueSystemId::from(
                    shared_state.response_sender.sender_port_id,
                )),
                request_id: self.request_id,
                number_of_elements: 1,
            })
        };
        unsafe { user_header_ptr.write(ResponseHeader::default()) };

        let ptr = unsafe {
            RawSampleMut::<
                service::header::request_response::ResponseHeader,
                ResponseHeader,
                MaybeUninit<ResponsePayload>,
            >::new_unchecked(header_ptr, user_header_ptr, chunk.payload.cast())
        };

        Ok(ResponseMutUninit {
            response: ResponseMut {
                ptr,
                shared_loan_counter: self.shared_loan_counter.clone(),
                shared_state: self.shared_state.clone(),
                offset_to_chunk: chunk.offset,
                channel_id: self.channel_id,
                connection_id: self.connection_id,
                sample_size: chunk.size,
                _response_payload: PhantomData,
                _response_header: PhantomData,
            },
        })
    }

    /// Sends a copy of the provided data to the
    /// [`PendingResponse`](crate::pending_response::PendingResponse) of the corresponding
    /// [`Client`](crate::port::client::Client).
    /// This is not a zero-copy API. Use [`ActiveRequest::loan_uninit()`] instead.
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    ///
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// # let service = node
    /// #     .service_builder(&"My/Funk/ServiceName".try_into()?)
    /// #     .request_response::<u64, u64>()
    /// #     .open_or_create()?;
    /// # let client = service.client_builder().create()?;
    /// # let server = service.server_builder().create()?;
    /// #
    /// # let pending_response = client.send_copy(123)?;
    ///
    /// let active_request = server.receive()?.unwrap();
    /// active_request.send_copy(456)?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn send_copy(&self, value: ResponsePayload) -> Result<(), SendError> {
        let msg = "Unable to send copy of response";
        let response = fail!(from self,
                            when self.loan_uninit(),
                            "{} since the loan of the response failed.", msg);

        response.write_payload(value).send()
    }
}

impl<
        Service: crate::service::Service,
        RequestPayload: Debug + ZeroCopySend + ?Sized,
        RequestHeader: Debug + ZeroCopySend,
        ResponsePayload: Debug + Default + ZeroCopySend + Sized,
        ResponseHeader: Default + Debug + ZeroCopySend,
    > ActiveRequest<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>
{
    /// Loans default initialized memory for a [`ResponseMut`] where the user can write its
    /// payload to.
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    ///
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// # let service = node
    /// #     .service_builder(&"My/Funk/ServiceName".try_into()?)
    /// #     .request_response::<u64, u64>()
    /// #     .open_or_create()?;
    /// # let client = service.client_builder().create()?;
    /// # let server = service.server_builder().create()?;
    /// #
    /// # let pending_response = client.send_copy(123)?;
    ///
    /// let active_request = server.receive()?.unwrap();
    /// let mut response = active_request.loan()?;
    /// *response = 789;
    /// response.send()?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn loan(&self) -> Result<ResponseMut<Service, ResponsePayload, ResponseHeader>, LoanError> {
        Ok(self
            .loan_uninit()?
            .write_payload(ResponsePayload::default()))
    }
}
////////////////////////
// END: typed API
////////////////////////

////////////////////////
// BEGIN: sliced API
////////////////////////
impl<
        Service: crate::service::Service,
        RequestPayload: Debug + ZeroCopySend + ?Sized,
        RequestHeader: Debug + ZeroCopySend,
        ResponsePayload: Debug + Default + ZeroCopySend + 'static,
        ResponseHeader: Default + Debug + ZeroCopySend,
    > ActiveRequest<Service, RequestPayload, RequestHeader, [ResponsePayload], ResponseHeader>
{
    /// Loans/allocates a [`ResponseMut`] from the underlying data segment of the
    /// [`Server`](crate::port::server::Server)
    /// and initializes all slice elements with the default value. This can be a performance hit
    /// and [`ActiveRequest::loan_slice_uninit()`] can be used to loan a slice of
    /// [`core::mem::MaybeUninit<Payload>`].
    ///
    /// On failure it returns [`LoanError`] describing the failure.
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// # let service = node.service_builder(&"Whatever6".try_into()?)
    /// #     .request_response::<u64, [usize]>()
    /// #     .open_or_create()?;
    /// #
    /// # let client = service.client_builder().create()?;
    /// let server = service.server_builder()
    ///                     .initial_max_slice_len(32)
    ///                     .create()?;
    /// # let pending_response = client.send_copy(0)?;
    /// let active_request = server.receive()?.unwrap();
    ///
    /// let slice_length = 13;
    /// let mut response = active_request.loan_slice(slice_length)?;
    /// response.send()?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn loan_slice(
        &self,
        slice_len: usize,
    ) -> Result<ResponseMut<Service, [ResponsePayload], ResponseHeader>, LoanError> {
        let response = self.loan_slice_uninit(slice_len)?;
        Ok(response.write_from_fn(|_| ResponsePayload::default()))
    }
}

impl<
        Service: crate::service::Service,
        RequestPayload: Debug + ZeroCopySend + ?Sized,
        RequestHeader: Debug + ZeroCopySend,
        ResponsePayload: Debug + ZeroCopySend + 'static,
        ResponseHeader: Default + Debug + ZeroCopySend,
    > ActiveRequest<Service, RequestPayload, RequestHeader, [ResponsePayload], ResponseHeader>
{
    /// Loans/allocates a [`ResponseMutUninit`] from the underlying data segment of the
    /// [`Server`](crate::port::server::Server).
    /// The user has to initialize the payload before it can be sent.
    ///
    /// On failure it returns [`LoanError`] describing the failure.
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// # let service = node.service_builder(&"Whatever6".try_into()?)
    /// #     .request_response::<u64, [usize]>()
    /// #     .open_or_create()?;
    /// #
    /// # let client = service.client_builder().create()?;
    /// let server = service.server_builder()
    ///                     .initial_max_slice_len(32)
    ///                     .create()?;
    /// # let pending_response = client.send_copy(0)?;
    /// let active_request = server.receive()?.unwrap();
    ///
    /// let slice_length = 13;
    /// let mut response = active_request.loan_slice_uninit(slice_length)?;
    /// for element in response.payload_mut() {
    ///     element.write(1234);
    /// }
    /// let response = unsafe { response.assume_init() };
    /// response.send()?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn loan_slice_uninit(
        &self,
        slice_len: usize,
    ) -> Result<ResponseMutUninit<Service, [MaybeUninit<ResponsePayload>], ResponseHeader>, LoanError>
    {
        debug_assert!(TypeId::of::<ResponsePayload>() != TypeId::of::<CustomPayloadMarker>());
        unsafe { self.loan_slice_uninit_impl(slice_len, slice_len) }
    }

    unsafe fn loan_slice_uninit_impl(
        &self,
        slice_len: usize,
        underlying_number_of_slice_elements: usize,
    ) -> Result<ResponseMutUninit<Service, [MaybeUninit<ResponsePayload>], ResponseHeader>, LoanError>
    {
        let shared_state = self.shared_state.lock();
        let max_slice_len = shared_state.config.initial_max_slice_len;

        if shared_state.config.allocation_strategy == AllocationStrategy::Static
            && max_slice_len < slice_len
        {
            fail!(from self, with LoanError::ExceedsMaxLoanSize,
                "Unable to loan slice with {} elements since it would exceed the max supported slice length of {}.",
                slice_len, max_slice_len);
        }

        self.increment_loan_counter()?;

        let response_layout = shared_state.response_sender.sample_layout(slice_len);
        let chunk = shared_state.response_sender.allocate(response_layout)?;

        let header_ptr: *mut service::header::request_response::ResponseHeader =
            chunk.header.cast();
        let user_header_ptr: *mut ResponseHeader = chunk.user_header.cast();
        unsafe {
            header_ptr.write(service::header::request_response::ResponseHeader {
                server_id: UniqueServerId(UniqueSystemId::from(
                    shared_state.response_sender.sender_port_id,
                )),
                request_id: self.request_id,
                number_of_elements: slice_len as _,
            })
        };
        unsafe { user_header_ptr.write(ResponseHeader::default()) };

        let ptr = unsafe {
            RawSampleMut::<
                service::header::request_response::ResponseHeader,
                ResponseHeader,
                [MaybeUninit<ResponsePayload>],
            >::new_unchecked(
                header_ptr,
                user_header_ptr,
                core::slice::from_raw_parts_mut(
                    chunk.payload.cast(),
                    underlying_number_of_slice_elements,
                ),
            )
        };

        Ok(ResponseMutUninit {
            response: ResponseMut {
                ptr,
                shared_loan_counter: self.shared_loan_counter.clone(),
                shared_state: self.shared_state.clone(),
                offset_to_chunk: chunk.offset,
                channel_id: self.channel_id,
                connection_id: self.connection_id,
                sample_size: chunk.size,
                _response_payload: PhantomData,
                _response_header: PhantomData,
            },
        })
    }
}

impl<Service: crate::service::Service>
    ActiveRequest<
        Service,
        [CustomPayloadMarker],
        CustomHeaderMarker,
        [CustomPayloadMarker],
        CustomHeaderMarker,
    >
{
    #[doc(hidden)]
    pub unsafe fn loan_custom_payload(
        &self,
        slice_len: usize,
    ) -> Result<
        ResponseMutUninit<Service, [MaybeUninit<CustomPayloadMarker>], CustomHeaderMarker>,
        LoanError,
    > {
        let shared_state = self.shared_state.lock();
        // TypeVariant::Dynamic == slice and only here it makes sense to loan more than one element
        debug_assert!(
            slice_len == 1
                || shared_state.response_sender.payload_type_variant() == TypeVariant::Dynamic
        );

        self.loan_slice_uninit_impl(
            slice_len,
            shared_state.response_sender.payload_size() * slice_len,
        )
    }
}
////////////////////////
// END: sliced API
////////////////////////
