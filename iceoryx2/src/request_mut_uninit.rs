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
//! #   .service_builder(&"My/Funk/ServiceName".try_into()?)
//! #   .request_response::<u64, u64>()
//! #   .open_or_create()?;
//! #
//! # let client = service.client_builder().create()?;
//! #
//!
//! // acquire uninitialized request
//! let request = client.loan_uninit()?;
//! // write payload and acquire an initialized request that can be sent
//! let request = request.write_payload(55712);
//!
//! let pending_response = request.send()?;
//!
//! # Ok(())
//! # }
//! ```

use flatbuffers::{FlatBufferBuilder, WIPOffset};
use iceoryx2_bb_elementary_traits::{iceoryx_send::IceoryxSend, zero_copy_send::ZeroCopySend};
use iceoryx2_bb_flatbuffers::ResizableMemory;
use iceoryx2_cal::{shared_memory::ShmPointer, zero_copy_connection::ChannelId};

use crate::{
    payload::number_of_elements,
    port::{
        client::ClientSharedState,
        details::{chunk::ChunkMut, chunk_mut_shared_state::ChunkMutSharedState},
    },
    request_mut::RequestMut,
    service::{self, marker::Flatbuffer},
};
use core::marker::PhantomData;
use core::{fmt::Debug, mem::MaybeUninit};

/// The memory used inside the [`FlatBufferBuilder`].
pub type FlatbufferMemory<Service> =
    ResizableMemory<ShmPointer, ChunkMutSharedState<Service, ClientSharedState<Service>>>;

/// A version of the [`RequestMut`] where the payload is not initialized which allows
/// true zero copy usage. To send a [`RequestMutUninit`] it must be first initialized
/// and converted into [`RequestMut`] with [`RequestMutUninit::assume_init()`].
pub struct RequestMutUninit<
    Service: crate::service::Service,
    RequestPayload: Debug + IceoryxSend + ?Sized,
    RequestHeader: Debug + ZeroCopySend,
    ResponsePayload: Debug + IceoryxSend + ?Sized,
    ResponseHeader: Debug + ZeroCopySend,
> {
    chunk: ChunkMut,
    shared_state: ChunkMutSharedState<Service, ClientSharedState<Service>>,
    channel_id: ChannelId,
    flatbuffer_builder: Option<FlatBufferBuilder<'static, FlatbufferMemory<Service>>>,
    assume_init_was_called: bool,
    _request_payload: PhantomData<RequestPayload>,
    _request_header: PhantomData<RequestHeader>,
    _response_payload: PhantomData<ResponsePayload>,
    _response_header: PhantomData<ResponseHeader>,
}

impl<
    Service: crate::service::Service,
    RequestPayload: Debug + IceoryxSend + ?Sized,
    RequestHeader: Debug + ZeroCopySend,
    ResponsePayload: Debug + IceoryxSend + ?Sized,
    ResponseHeader: Debug + ZeroCopySend,
> Drop
    for RequestMutUninit<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>
{
    fn drop(&mut self) {
        if !self.assume_init_was_called {
            let _ = self.shared_state.call(|s| -> Result<(), ()> {
                let header = unsafe { &*self.chunk.header_ptr().cast() };
                s.release_request(false, header);
                Ok(())
            });
        }
    }
}

unsafe impl<
    Service: crate::service::Service,
    RequestPayload: Debug + IceoryxSend + ?Sized,
    RequestHeader: Debug + ZeroCopySend,
    ResponsePayload: Debug + IceoryxSend + ?Sized,
    ResponseHeader: Debug + ZeroCopySend,
> Send for RequestMutUninit<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>
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
> Debug
    for RequestMutUninit<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "RequestMutUninit {{  chunk: {:?}, channel_id: {:?} }}",
            self.chunk, self.channel_id
        )
    }
}

impl<
    Service: crate::service::Service,
    RequestPayload,
    RequestHeader: Debug + ZeroCopySend,
    ResponsePayload: Debug + IceoryxSend + ?Sized,
    ResponseHeader: Debug + ZeroCopySend,
>
    RequestMutUninit<
        Service,
        Flatbuffer<RequestPayload>,
        RequestHeader,
        ResponsePayload,
        ResponseHeader,
    >
{
    pub(crate) fn new_flatbuffer(
        shared_state: &Service::ArcThreadSafetyPolicy<ClientSharedState<Service>>,
        chunk: ChunkMut,
        channel_id: ChannelId,
    ) -> Self {
        let mut new_self = Self {
            flatbuffer_builder: None,
            shared_state: ChunkMutSharedState::new(shared_state, &chunk).unwrap(),
            chunk,
            channel_id,
            assume_init_was_called: false,
            _request_header: PhantomData,
            _request_payload: PhantomData,
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
        root: WIPOffset<RequestPayload>,
    ) -> RequestMut<
        Service,
        Flatbuffer<RequestPayload>,
        RequestHeader,
        ResponsePayload,
        ResponseHeader,
    > {
        self.flatbuffer_builder().finish(root, None);
        let payload_ptr = self.flatbuffer_builder().finished_data().as_ptr();
        self.__internal_finish_serialized(payload_ptr);
        self.assume_init_was_called = true;

        RequestMut {
            chunk: self.chunk.clone(),
            shared_state: self.shared_state.clone(),
            was_sample_sent: false,
            channel_id: self.channel_id,
            _request_payload: PhantomData,
            _request_header: PhantomData,
            _response_header: PhantomData,
            _response_payload: PhantomData,
        }
    }
}

impl<
    Service: crate::service::Service,
    RequestPayload: Debug + IceoryxSend + ?Sized,
    RequestHeader: Debug + ZeroCopySend,
    ResponsePayload: Debug + IceoryxSend + ?Sized,
    ResponseHeader: Debug + ZeroCopySend,
> RequestMutUninit<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>
{
    pub(crate) fn new(
        shared_state: &Service::ArcThreadSafetyPolicy<ClientSharedState<Service>>,
        chunk: ChunkMut,
        channel_id: ChannelId,
    ) -> Self {
        Self {
            flatbuffer_builder: None,
            shared_state: ChunkMutSharedState::new(shared_state, &chunk).unwrap(),
            chunk,
            channel_id,
            assume_init_was_called: false,
            _request_payload: PhantomData,
            _request_header: PhantomData,
            _response_payload: PhantomData,
            _response_header: PhantomData,
        }
    }

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
                .cast::<crate::service::header::request_response::RequestHeader>()
        };
        header.number_of_elements = memory_structure.number_of_elements;
        header.payload_offset = memory_structure.payload_offset;
    }
}

impl<
    Service: crate::service::Service,
    RequestPayload: Debug + IceoryxSend + ZeroCopySend + ?Sized,
    RequestHeader: Debug + ZeroCopySend,
    ResponsePayload: Debug + IceoryxSend + ?Sized,
    ResponseHeader: Debug + ZeroCopySend,
> RequestMutUninit<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>
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
}

impl<
    Service: crate::service::Service,
    RequestPayload: Debug + IceoryxSend + ZeroCopySend,
    RequestHeader: Debug + ZeroCopySend,
    ResponsePayload: Debug + IceoryxSend + ?Sized,
    ResponseHeader: Debug + ZeroCopySend,
>
    RequestMutUninit<
        Service,
        MaybeUninit<RequestPayload>,
        RequestHeader,
        ResponsePayload,
        ResponseHeader,
    >
{
    /// Returns a reference to the user defined request payload.
    pub fn payload(&self) -> &MaybeUninit<RequestPayload> {
        unsafe { &*self.chunk.payload_ptr().cast() }
    }

    /// Returns a mutable reference to the user defined request payload.
    pub fn payload_mut(&mut self) -> &mut MaybeUninit<RequestPayload> {
        unsafe { &mut *self.chunk.payload_mut_ptr().cast() }
    }
}

impl<
    Service: crate::service::Service,
    RequestPayload: Debug + IceoryxSend + ZeroCopySend,
    RequestHeader: Debug + ZeroCopySend,
    ResponsePayload: Debug + IceoryxSend + ?Sized,
    ResponseHeader: Debug + ZeroCopySend,
>
    RequestMutUninit<
        Service,
        [MaybeUninit<RequestPayload>],
        RequestHeader,
        ResponsePayload,
        ResponseHeader,
    >
{
    /// Returns a reference to the user defined request payload.
    pub fn payload(&self) -> &[MaybeUninit<RequestPayload>] {
        let payload_size = self.shared_state.payload_size();

        unsafe {
            &*core::ptr::slice_from_raw_parts(
                self.chunk.payload_ptr().cast(),
                number_of_elements::<RequestPayload, _>(self.header(), payload_size),
            )
        }
    }

    /// Returns a mutable reference to the user defined request payload.
    pub fn payload_mut(&mut self) -> &mut [MaybeUninit<RequestPayload>] {
        let payload_size = self.shared_state.payload_size();

        unsafe {
            &mut *core::ptr::slice_from_raw_parts_mut(
                self.chunk.payload_mut_ptr().cast(),
                number_of_elements::<RequestPayload, _>(self.header(), payload_size),
            )
        }
    }
}

impl<
    Service: crate::service::Service,
    RequestPayload: Debug + ZeroCopySend + Sized,
    RequestHeader: Debug + ZeroCopySend,
    ResponsePayload: Debug + IceoryxSend + ?Sized,
    ResponseHeader: Debug + ZeroCopySend,
>
    RequestMutUninit<
        Service,
        MaybeUninit<RequestPayload>,
        RequestHeader,
        ResponsePayload,
        ResponseHeader,
    >
{
    /// Copies the provided payload into the uninitialized request and returns
    /// an initialized [`RequestMut`].
    pub fn write_payload(
        mut self,
        value: RequestPayload,
    ) -> RequestMut<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader> {
        self.payload_mut().write(value);
        unsafe { self.assume_init() }
    }

    /// When the payload is manually populated by using
    /// [`RequestMutUninit::payload_mut()`], then this function can be used
    /// to convert it into the initialized [`RequestMut`] version.
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    ///
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// # let service = node
    /// #    .service_builder(&"My/Funk/ServiceName".try_into()?)
    /// #    .request_response::<u64, u64>()
    /// #    .open_or_create()?;
    /// #
    /// # let client = service.client_builder().create()?;
    ///
    /// let mut request = client.loan_uninit()?;
    /// // use the MaybeUninit API to initialize the payload
    /// request.payload_mut().write(8283);
    /// // we have written the payload, initialize the request
    /// let request = unsafe { request.assume_init() };
    ///
    /// let pending_response = request.send()?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    /// # Safety
    ///
    /// The caller must ensure that [`core::mem::MaybeUninit<Payload>`] really is initialized.
    /// Sending the content when it is not fully initialized causes immediate undefined behavior.
    pub unsafe fn assume_init(
        mut self,
    ) -> RequestMut<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader> {
        self.assume_init_was_called = true;
        RequestMut {
            chunk: self.chunk.clone(),
            shared_state: self.shared_state.clone(),
            was_sample_sent: false,
            channel_id: self.channel_id,
            _request_payload: PhantomData,
            _request_header: PhantomData,
            _response_header: PhantomData,
            _response_payload: PhantomData,
        }
    }
}

impl<
    Service: crate::service::Service,
    RequestPayload: Debug + ZeroCopySend,
    RequestHeader: Debug + ZeroCopySend,
    ResponsePayload: Debug + IceoryxSend + ?Sized,
    ResponseHeader: Debug + ZeroCopySend,
>
    RequestMutUninit<
        Service,
        [MaybeUninit<RequestPayload>],
        RequestHeader,
        ResponsePayload,
        ResponseHeader,
    >
{
    /// When the payload is manually populated by using
    /// [`RequestMutUninit::payload_mut()`], then this function can be used
    /// to convert it into the initialized [`RequestMut`] version.
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    ///
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// let service = node
    ///    .service_builder(&"My/Funk/ServiceName".try_into()?)
    ///    .request_response::<[u64], u64>()
    ///    .open_or_create()?;
    ///
    /// let client = service.client_builder()
    ///                     .initial_max_slice_len(32)
    ///                     .create()?;
    ///
    /// let slice_length = 13;
    /// let mut request = client.loan_slice_uninit(slice_length)?;
    /// for element in request.payload_mut() {
    ///     element.write(1234);
    /// }
    /// // we have written the payload, initialize the request
    /// let request = unsafe { request.assume_init() };
    ///
    /// let pending_response = request.send()?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    /// # Safety
    ///
    /// The caller must ensure that [`core::mem::MaybeUninit<Payload>`] really is initialized.
    /// Sending the content when it is not fully initialized causes immediate undefined behavior.
    pub unsafe fn assume_init(
        self,
    ) -> RequestMut<Service, [RequestPayload], RequestHeader, ResponsePayload, ResponseHeader> {
        let this = core::mem::ManuallyDrop::new(self);
        RequestMut {
            chunk: this.chunk.clone(),
            shared_state: unsafe { core::ptr::read(&this.shared_state) },
            was_sample_sent: false,
            channel_id: this.channel_id,
            _request_payload: PhantomData,
            _request_header: PhantomData,
            _response_header: PhantomData,
            _response_payload: PhantomData,
        }
    }

    /// Writes the payload to the [`RequestMutUninit`] and labels the [`RequestMutUninit`] as
    /// initialized
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    ///
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// let service = node
    ///    .service_builder(&"My/Funk/ServiceName".try_into()?)
    ///    .request_response::<[usize], u64>()
    ///    .open_or_create()?;
    ///
    /// let client = service.client_builder()
    ///                     .initial_max_slice_len(32)
    ///                     .create()?;
    ///
    /// let slice_length = 13;
    /// let mut request = client.loan_slice_uninit(slice_length)?;
    /// let request = request.write_from_fn(|index| index + 123);
    ///
    /// let pending_response = request.send()?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn write_from_fn<F: FnMut(usize) -> RequestPayload>(
        mut self,
        mut initializer: F,
    ) -> RequestMut<Service, [RequestPayload], RequestHeader, ResponsePayload, ResponseHeader> {
        for (i, element) in self.payload_mut().iter_mut().enumerate() {
            element.write(initializer(i));
        }

        // SAFETY: this is safe since the payload was initialized on the line above
        unsafe { self.assume_init() }
    }
}

impl<
    Service: crate::service::Service,
    RequestPayload: Debug + Copy + ZeroCopySend,
    RequestHeader: Debug + ZeroCopySend,
    ResponsePayload: Debug + IceoryxSend + ?Sized,
    ResponseHeader: Debug + ZeroCopySend,
>
    RequestMutUninit<
        Service,
        [MaybeUninit<RequestPayload>],
        RequestHeader,
        ResponsePayload,
        ResponseHeader,
    >
{
    /// Writes the payload by mem copying the provided slice into the [`RequestMutUninit`].
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    ///
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// let service = node
    ///    .service_builder(&"My/Funk/ServiceName".try_into()?)
    ///    .request_response::<[u64], u64>()
    ///    .open_or_create()?;
    ///
    /// let client = service.client_builder()
    ///                     .initial_max_slice_len(32)
    ///                     .create()?;
    ///
    /// let slice_length = 3;
    /// let mut request = client.loan_slice_uninit(slice_length)?;
    /// let request = request.write_from_slice(&vec![1,2,3]);
    ///
    /// let pending_response = request.send()?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn write_from_slice(
        mut self,
        value: &[RequestPayload],
    ) -> RequestMut<Service, [RequestPayload], RequestHeader, ResponsePayload, ResponseHeader> {
        self.payload_mut().copy_from_slice(unsafe {
            core::mem::transmute::<&[RequestPayload], &[MaybeUninit<RequestPayload>]>(value)
        });
        unsafe { self.assume_init() }
    }
}
