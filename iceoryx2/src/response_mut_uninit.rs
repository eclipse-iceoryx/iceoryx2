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
//! let mut response = active_request.loan_uninit()?;
//! // write 1234 into sample
//! response.payload_mut().write(1234);
//! // overwrite contents with 456 because its fun
//! let response = response.write_payload(456);
//!
//! println!("server id: {:?}", response.header().server_id());
//! response.send()?;
//!
//! # Ok(())
//! # }
//! ```

extern crate alloc;

use flatbuffers::{FlatBufferBuilder, WIPOffset};
use iceoryx2_bb_concurrency::atomic::{AtomicUsize, Ordering};
use iceoryx2_bb_elementary_traits::{iceoryx_send::IceoryxSend, zero_copy_send::ZeroCopySend};
use iceoryx2_bb_flatbuffers::ResizableMemory;
use iceoryx2_cal::{shared_memory::ShmPointer, zero_copy_connection::ChannelId};

use crate::{
    payload::number_of_elements,
    port::{
        details::{chunk::ChunkMut, chunk_mut_shared_state::ChunkMutSharedState},
        server::SharedServerState,
    },
    response_mut::ResponseMut,
    service::{self, marker::Flatbuffer},
};
use alloc::sync::Arc;
use core::marker::PhantomData;
use core::{fmt::Debug, mem::MaybeUninit};

/// The memory used inside the [`FlatBufferBuilder`].
pub type FlatbufferMemory<Service> =
    ResizableMemory<ShmPointer, ChunkMutSharedState<Service, SharedServerState<Service>>>;

/// Acquired by a [`ActiveRequest`](crate::active_request::ActiveRequest) with
///  * [`ActiveRequest::loan_uninit()`](crate::active_request::ActiveRequest::loan_uninit())
///
/// It stores the payload of the response that will be sent to the corresponding
/// [`PendingResponse`](crate::pending_response::PendingResponse) of the
/// [`Client`](crate::port::client::Client).
///
/// If the [`ResponseMutUninit`] is not sent it will reelase the loaned memory when going out of
/// scope.
///
/// The generic parameter `Payload` is actually [`core::mem::MaybeUninit<Payload>`].
pub struct ResponseMutUninit<
    Service: service::Service,
    ResponsePayload: Debug + IceoryxSend + ?Sized,
    ResponseHeader: Debug + ZeroCopySend,
> {
    shared_state: ChunkMutSharedState<Service, SharedServerState<Service>>,
    shared_loan_counter: Arc<AtomicUsize>,
    chunk: ChunkMut,
    channel_id: ChannelId,
    connection_id: usize,
    flatbuffer_builder: Option<FlatBufferBuilder<'static, FlatbufferMemory<Service>>>,
    assume_init_was_called: bool,
    _response_payload: PhantomData<ResponsePayload>,
    _response_header: PhantomData<ResponseHeader>,
}

unsafe impl<
    Service: crate::service::Service,
    ResponsePayload: Debug + IceoryxSend + ?Sized,
    ResponseHeader: Debug + ZeroCopySend,
> Send for ResponseMutUninit<Service, ResponsePayload, ResponseHeader>
where
    Service::ArcThreadSafetyPolicy<SharedServerState<Service>>: Send + Sync,
{
}

impl<
    Service: crate::service::Service,
    ResponsePayload: Debug + IceoryxSend + ?Sized,
    ResponseHeader: Debug + ZeroCopySend,
> Drop for ResponseMutUninit<Service, ResponsePayload, ResponseHeader>
{
    fn drop(&mut self) {
        if !self.assume_init_was_called {
            self.shared_loan_counter.fetch_sub(1, Ordering::Relaxed);
        }
    }
}

impl<
    Service: crate::service::Service,
    ResponsePayload: Debug + IceoryxSend + ?Sized,
    ResponseHeader: Debug + ZeroCopySend,
> Debug for ResponseMutUninit<Service, ResponsePayload, ResponseHeader>
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "ResponseMut {{ shared_loan_counter: {:?}, chunk: {:?}, channel_id: {:?}, connection_id: {} }}",
            self.shared_loan_counter.load(Ordering::Relaxed),
            self.chunk,
            self.channel_id,
            self.connection_id
        )
    }
}

impl<Service: crate::service::Service, ResponsePayload, ResponseHeader: Debug + ZeroCopySend>
    ResponseMutUninit<Service, Flatbuffer<ResponsePayload>, ResponseHeader>
{
    pub(crate) fn new_flatbuffer(
        shared_state: &Service::ArcThreadSafetyPolicy<SharedServerState<Service>>,
        chunk: ChunkMut,
        shared_loan_counter: &Arc<AtomicUsize>,
        channel_id: ChannelId,
        connection_id: usize,
    ) -> Self {
        let mut new_self = Self {
            flatbuffer_builder: None,
            shared_state: ChunkMutSharedState::new(shared_state, &chunk).unwrap(),
            chunk,
            channel_id,
            assume_init_was_called: false,
            shared_loan_counter: shared_loan_counter.clone(),
            connection_id,
            _response_header: PhantomData,
            _response_payload: PhantomData,
        };

        new_self.flatbuffer_builder = Some(FlatBufferBuilder::new_in(
            new_self.__internal_create_resizable_memory(),
        ));

        new_self
    }

    /// Returns the internal [`FlatBufferBuilder`] that was constructed with the internal iceoryx2
    /// allocator to enable true zero-copy data transfer.
    pub fn flatbuffer_builder(
        &mut self,
    ) -> &mut FlatBufferBuilder<'static, FlatbufferMemory<Service>> {
        self.flatbuffer_builder.as_mut().unwrap()
    }

    /// Finalize the Flatbuffer and initialize the sample. After that call the content can no longer be
    /// modified.
    pub fn assume_init(
        mut self,
        root: WIPOffset<ResponsePayload>,
    ) -> ResponseMut<Service, Flatbuffer<ResponsePayload>, ResponseHeader> {
        self.flatbuffer_builder().finish(root, None);
        let payload_ptr = self.flatbuffer_builder().finished_data().as_ptr();
        self.__internal_finish_serialized(payload_ptr);
        self.assume_init_was_called = true;

        ResponseMut {
            shared_loan_counter: self.shared_loan_counter.clone(),
            shared_state: self.shared_state.clone(),
            connection_id: self.connection_id,
            chunk: self.chunk.clone(),
            channel_id: self.channel_id,
            _response_header: PhantomData,
            _response_payload: PhantomData,
        }
    }
}

impl<
    Service: crate::service::Service,
    ResponsePayload: Debug + IceoryxSend + ?Sized,
    ResponseHeader: Debug + ZeroCopySend,
> ResponseMutUninit<Service, ResponsePayload, ResponseHeader>
{
    #[doc(hidden)]
    pub fn __internal_create_resizable_memory(&self) -> FlatbufferMemory<Service> {
        self.shared_state.create_resizable_memory(&self.chunk)
    }

    #[doc(hidden)]
    pub fn __internal_finish_serialized(&mut self, payload_ptr: *const u8) {
        let memory_structure = self.shared_state.memory_structure(payload_ptr);
        self.chunk = memory_structure.chunk;

        let header = unsafe {
            &mut *self
                .chunk
                .header_mut_ptr()
                .cast::<crate::service::header::request_response::ResponseHeader>()
        };
        header.number_of_elements = self.chunk.size() as _;
        header.payload_offset = memory_structure.payload_offset;
    }
}

impl<
    Service: crate::service::Service,
    ResponsePayload: Debug + IceoryxSend + ZeroCopySend + ?Sized,
    ResponseHeader: Debug + ZeroCopySend,
> ResponseMutUninit<Service, ResponsePayload, ResponseHeader>
{
    pub(crate) fn new(
        shared_state: &Service::ArcThreadSafetyPolicy<SharedServerState<Service>>,
        chunk: &ChunkMut,
        shared_loan_counter: &Arc<AtomicUsize>,
        channel_id: ChannelId,
        connection_id: usize,
    ) -> Self {
        Self {
            shared_state: ChunkMutSharedState::new(shared_state, chunk).unwrap(),
            shared_loan_counter: shared_loan_counter.clone(),
            chunk: chunk.clone(),
            channel_id,
            connection_id,
            assume_init_was_called: false,
            flatbuffer_builder: None,
            _response_payload: PhantomData,
            _response_header: PhantomData,
        }
    }

    /// Returns a reference to the
    /// [`ResponseHeader`](service::header::request_response::ResponseHeader).
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
    /// let response = active_request.loan_uninit()?;
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
    /// # let service = node.service_builder(&"Whatever2".try_into()?)
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
    /// let mut response = active_request.loan_uninit()?;
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
    /// # let service = node.service_builder(&"Whatever".try_into()?)
    /// #     .request_response::<u64, u64>()
    /// #     .response_user_header::<u64>()
    /// #     .open_or_create()?;
    /// #
    /// # let client = service.client_builder().create()?;
    /// # let server = service.server_builder().create()?;
    /// # let pending_response = client.send_copy(0)?;
    /// # let active_request = server.receive()?.unwrap();
    ///
    /// let mut response = active_request.loan_uninit()?;
    /// *response.user_header_mut() = 123;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn user_header_mut(&mut self) -> &mut ResponseHeader {
        unsafe { &mut *self.chunk.user_header_mut_ptr().cast() }
    }
}

impl<
    Service: crate::service::Service,
    ResponsePayload: Debug + ZeroCopySend,
    ResponseHeader: Debug + ZeroCopySend,
> ResponseMutUninit<Service, ResponsePayload, ResponseHeader>
{
    /// Returns a reference to the payload of the response.
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// # let service = node.service_builder(&"Whatever3".try_into()?)
    /// #     .request_response::<u64, u64>()
    /// #     .open_or_create()?;
    /// #
    /// # let client = service.client_builder().create()?;
    /// # let server = service.server_builder().create()?;
    /// # let pending_response = client.send_copy(0)?;
    /// # let active_request = server.receive()?.unwrap();
    ///
    /// let mut response = active_request.loan_uninit()?;
    /// response.payload_mut().write(123);
    /// println!("payload: {:?}", *response.payload());
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn payload(&self) -> &ResponsePayload {
        unsafe { &*self.chunk.payload_ptr().cast() }
    }

    /// Returns a mutable reference to the payload of the response.
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// # let service = node.service_builder(&"Whatever4".try_into()?)
    /// #     .request_response::<u64, u64>()
    /// #     .open_or_create()?;
    /// #
    /// # let client = service.client_builder().create()?;
    /// # let server = service.server_builder().create()?;
    /// # let pending_response = client.send_copy(0)?;
    /// # let active_request = server.receive()?.unwrap();
    ///
    /// let mut response = active_request.loan_uninit()?;
    /// response.payload_mut().write(123);
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn payload_mut(&mut self) -> &mut ResponsePayload {
        unsafe { &mut *self.chunk.payload_mut_ptr().cast() }
    }
}

impl<
    Service: crate::service::Service,
    ResponsePayload: Debug + ZeroCopySend,
    ResponseHeader: Debug + ZeroCopySend,
> ResponseMutUninit<Service, [ResponsePayload], ResponseHeader>
{
    /// Returns a reference to the payload of the response.
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// # let service = node.service_builder(&"Whatever3".try_into()?)
    /// #     .request_response::<u64, [u64]>()
    /// #     .open_or_create()?;
    /// #
    /// # let client = service.client_builder().create()?;
    /// # let server = service.server_builder().create()?;
    /// # let pending_response = client.send_copy(0)?;
    /// # let active_request = server.receive()?.unwrap();
    ///
    /// let mut response = active_request.loan_slice_uninit(1)?;
    /// response.payload_mut()[0].write(123);
    /// println!("payload: {:?}", *response.payload());
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn payload(&self) -> &[ResponsePayload] {
        let payload_size = self.shared_state.payload_size();
        unsafe {
            &*core::ptr::slice_from_raw_parts(
                self.chunk.payload_ptr().cast(),
                number_of_elements::<ResponsePayload, _>(self.header(), payload_size),
            )
        }
    }

    /// Returns a mutable reference to the payload of the response.
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// # let service = node.service_builder(&"Whatever4".try_into()?)
    /// #     .request_response::<u64, [u64]>()
    /// #     .open_or_create()?;
    /// #
    /// # let client = service.client_builder().create()?;
    /// # let server = service.server_builder().create()?;
    /// # let pending_response = client.send_copy(0)?;
    /// # let active_request = server.receive()?.unwrap();
    ///
    /// let mut response = active_request.loan_slice_uninit(12)?;
    /// response.payload_mut()[4].write(123);
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn payload_mut(&mut self) -> &mut [ResponsePayload] {
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
    ResponsePayload: Debug + ZeroCopySend,
    ResponseHeader: Debug + ZeroCopySend,
> ResponseMutUninit<Service, MaybeUninit<ResponsePayload>, ResponseHeader>
{
    /// Writes the provided payload into the [`ResponseMutUninit`] and returns an initialized
    /// [`ResponseMut`] that is ready to be sent.
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// # let service = node.service_builder(&"Whatever5".try_into()?)
    /// #     .request_response::<u64, u64>()
    /// #     .open_or_create()?;
    /// #
    /// # let client = service.client_builder().create()?;
    /// # let server = service.server_builder().create()?;
    /// # let pending_response = client.send_copy(0)?;
    /// # let active_request = server.receive()?.unwrap();
    ///
    /// let mut response = active_request.loan_uninit()?;
    /// let response = response.write_payload(123);
    /// response.send()?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn write_payload(
        mut self,
        value: ResponsePayload,
    ) -> ResponseMut<Service, ResponsePayload, ResponseHeader> {
        self.payload_mut().write(value);
        unsafe { self.assume_init() }
    }

    /// Converts the [`ResponseMutUninit`] into [`ResponseMut`]. This shall be done after the
    /// payload was written into the [`ResponseMutUninit`].
    ///
    /// # Safety
    ///
    ///  * Must ensure that the payload was properly initialized.
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// # let service = node.service_builder(&"Whatever6".try_into()?)
    /// #     .request_response::<u64, u64>()
    /// #     .open_or_create()?;
    /// #
    /// # let client = service.client_builder().create()?;
    /// # let server = service.server_builder().create()?;
    /// # let pending_response = client.send_copy(0)?;
    /// # let active_request = server.receive()?.unwrap();
    ///
    /// let mut response = active_request.loan_uninit()?;
    /// response.payload_mut().write(789);
    /// // this is fine since the payload was initialized to 789
    /// let response = unsafe { response.assume_init() };
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub unsafe fn assume_init(mut self) -> ResponseMut<Service, ResponsePayload, ResponseHeader> {
        self.assume_init_was_called = true;
        ResponseMut {
            shared_state: self.shared_state.clone(),
            shared_loan_counter: self.shared_loan_counter.clone(),
            channel_id: self.channel_id,
            chunk: self.chunk.clone(),
            connection_id: self.connection_id,
            _response_header: PhantomData,
            _response_payload: PhantomData,
        }
    }
}

impl<
    Service: crate::service::Service,
    ResponsePayload: Debug + ZeroCopySend,
    ResponseHeader: Debug + ZeroCopySend,
> ResponseMutUninit<Service, [MaybeUninit<ResponsePayload>], ResponseHeader>
{
    /// Converts the [`ResponseMutUninit`] into [`ResponseMut`]. This shall be done after the
    /// payload was written into the [`ResponseMutUninit`].
    ///
    /// # Safety
    ///
    ///  * Must ensure that the payload was properly initialized.
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// # let service = node.service_builder(&"Whatever6".try_into()?)
    /// #     .request_response::<u64, [u64]>()
    /// #     .open_or_create()?;
    /// #
    /// # let client = service.client_builder().create()?;
    /// # let server = service.server_builder()
    ///                       .initial_max_slice_len(32)
    ///                       .create()?;
    /// # let pending_response = client.send_copy(0)?;
    /// # let active_request = server.receive()?.unwrap();
    ///
    /// let slice_length = 13;
    /// let mut response = active_request.loan_slice_uninit(slice_length)?;
    /// for element in response.payload_mut() {
    ///     element.write(1234);
    /// }
    /// // this is fine since the payload was initialized to 789
    /// let response = unsafe { response.assume_init() };
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub unsafe fn assume_init(mut self) -> ResponseMut<Service, [ResponsePayload], ResponseHeader> {
        self.assume_init_was_called = true;
        ResponseMut {
            shared_state: self.shared_state.clone(),
            shared_loan_counter: self.shared_loan_counter.clone(),
            channel_id: self.channel_id,
            chunk: self.chunk.clone(),
            connection_id: self.connection_id,
            _response_header: PhantomData,
            _response_payload: PhantomData,
        }
    }

    /// Writes the payload to the [`ResponseMutUninit`] and labels the [`ResponseMutUninit`] as
    /// initialized
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
    /// # let server = service.server_builder()
    ///                       .initial_max_slice_len(32)
    ///                       .create()?;
    /// # let pending_response = client.send_copy(0)?;
    /// # let active_request = server.receive()?.unwrap();
    ///
    /// let slice_length = 13;
    /// let mut response = active_request.loan_slice_uninit(slice_length)?;
    /// let response = response.write_from_fn(|index| index * 2 + 3);
    /// response.send()?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn write_from_fn<F: FnMut(usize) -> ResponsePayload>(
        mut self,
        mut initializer: F,
    ) -> ResponseMut<Service, [ResponsePayload], ResponseHeader> {
        for (i, element) in self.payload_mut().iter_mut().enumerate() {
            element.write(initializer(i));
        }

        // SAFETY: this is safe since the payload was initialized on the line above
        unsafe { self.assume_init() }
    }
}

impl<
    Service: crate::service::Service,
    ResponsePayload: Debug + Copy + ZeroCopySend,
    ResponseHeader: Debug + ZeroCopySend,
> ResponseMutUninit<Service, [MaybeUninit<ResponsePayload>], ResponseHeader>
{
    /// Writes the payload by mem copying the provided slice into the [`ResponseMutUninit`].
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// # let service = node.service_builder(&"Whatever6".try_into()?)
    /// #     .request_response::<u64, [u64]>()
    /// #     .open_or_create()?;
    /// #
    /// # let client = service.client_builder().create()?;
    /// # let server = service.server_builder()
    ///                       .initial_max_slice_len(32)
    ///                       .create()?;
    /// # let pending_response = client.send_copy(0)?;
    /// # let active_request = server.receive()?.unwrap();
    ///
    /// let slice_length = 4;
    /// let mut response = active_request.loan_slice_uninit(slice_length)?;
    /// let response = response.write_from_slice(&vec![1, 2, 3, 4]);
    /// response.send()?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn write_from_slice(
        mut self,
        value: &[ResponsePayload],
    ) -> ResponseMut<Service, [ResponsePayload], ResponseHeader> {
        self.payload_mut().copy_from_slice(unsafe {
            core::mem::transmute::<&[ResponsePayload], &[MaybeUninit<ResponsePayload>]>(value)
        });
        unsafe { self.assume_init() }
    }
}
