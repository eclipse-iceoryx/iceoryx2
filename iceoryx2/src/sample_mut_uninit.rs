// Copyright (c) 2024 Contributors to the Eclipse Foundation
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
//! #     .publish_subscribe::<u64>()
//! #     .open_or_create()?;
//! #
//! # let publisher = service.publisher_builder().create()?;
//!
//! let sample = publisher.loan_uninit()?;
//! // write 1234 into sample
//! let mut sample = sample.write_payload(1234);
//! // override contents with 456 because its fun
//! *sample.payload_mut() = 456;
//!
//! println!("publisher port id: {:?}", sample.header().publisher_id());
//! sample.send()?;
//!
//! # Ok(())
//! # }
//! ```
//!
//! ## Slice API with callback initialization
//!
//! ```
//! use iceoryx2::prelude::*;
//! # fn main() -> Result<(), Box<dyn core::error::Error>> {
//! # let node = NodeBuilder::new().create::<ipc::Service>()?;
//! #
//! # let service = node.service_builder(&"My/Funk/ServiceName".try_into()?)
//! #     .publish_subscribe::<[usize]>()
//! #     .create()?;
//! #
//! # let publisher = service.publisher_builder().initial_max_slice_len(16).create()?;
//!
//! let slice_length = 12;
//! let sample = publisher.loan_slice_uninit(slice_length)?;
//! // initialize the n-th element of the slice with n * 1234
//! let mut sample = sample.write_from_fn(|n| n * 1234);
//! // override the content of the first element with 42
//! sample.payload_mut()[0] = 42;
//!
//! println!("publisher port id: {:?}", sample.header().publisher_id());
//! sample.send()?;
//!
//! # Ok(())
//! # }
//! ```
//!
//! ## Slice API with copy initialization
//!
//! ```
//! use iceoryx2::prelude::*;
//! # fn main() -> Result<(), Box<dyn core::error::Error>> {
//! # let node = NodeBuilder::new().create::<ipc::Service>()?;
//! #
//! # let service = node.service_builder(&"My/Funk/ServiceName".try_into()?)
//! #     .publish_subscribe::<[usize]>()
//! #     .create()?;
//! #
//! # let publisher = service.publisher_builder().initial_max_slice_len(16).create()?;
//!
//! let slice_length = 4;
//! let sample = publisher.loan_slice_uninit(slice_length)?;
//! // initialize the slice with the numbers 1, 2, 3, 4
//! let mut sample = sample.write_from_slice(&vec![1, 2, 3, 4]);
//!
//! println!("publisher port id: {:?}", sample.header().publisher_id());
//! sample.send()?;
//!
//! # Ok(())
//! # }
//! ```

use core::marker::PhantomData;
use core::{fmt::Debug, mem::MaybeUninit};
use iceoryx2_bb_concurrency::atomic::Ordering;
use iceoryx2_bb_flatbuffers::{ResizableMemory, ResizableMemoryBuilder};
use iceoryx2_cal::arc_sync_policy::ArcSyncPolicy;

use flatbuffers::{FlatBufferBuilder, WIPOffset};
use iceoryx2_bb_elementary_traits::{iceoryx_send::IceoryxSend, zero_copy_send::ZeroCopySend};
use iceoryx2_cal::shared_memory::ShmPointer;

use crate::port::details::chunk::ChunkMut;
use crate::{
    port::publisher::PublisherSharedState,
    sample_mut::{SampleMut, SampleMutSharedState},
    service::{header::publish_subscribe::Header, marker::Flatbuffer},
};

/// The memory used inside the [`FlatBufferBuilder`].
pub type FlatbufferMemory<Service> = ResizableMemory<ShmPointer, SampleMutSharedState<Service>>;

/// Acquired by a [`crate::port::publisher::Publisher`] via
///  * [`crate::port::publisher::Publisher::loan_uninit()`]
///  * [`crate::port::publisher::Publisher::loan_slice_uninit()`]
///
/// It stores the payload that will be sent
/// to all connected [`crate::port::subscriber::Subscriber`]s. If the [`SampleMut`] is not sent
/// it will release the loaned memory when going out of scope.
pub struct SampleMutUninit<
    Service: crate::service::Service,
    Payload: IceoryxSend + Debug + ?Sized,
    UserHeader: ZeroCopySend,
> {
    shared_state: SampleMutSharedState<Service>,
    chunk: ChunkMut,
    flatbuffer_builder: Option<FlatBufferBuilder<'static, FlatbufferMemory<Service>>>,
    _payload: PhantomData<Payload>,
    _user_header: PhantomData<UserHeader>,
}

unsafe impl<
    Service: crate::service::Service,
    Payload: IceoryxSend + Debug + ?Sized,
    UserHeader: ZeroCopySend,
> Send for SampleMutUninit<Service, Payload, UserHeader>
where
    Service::ArcThreadSafetyPolicy<PublisherSharedState<Service>>: Send + Sync,
{
}

impl<
    Service: crate::service::Service,
    Payload: IceoryxSend + Debug + ?Sized,
    UserHeader: ZeroCopySend,
> SampleMutUninit<Service, Payload, UserHeader>
{
    #[doc(hidden)]
    pub fn __internal_create_resizable_memory_builder(
        &self,
    ) -> ResizableMemory<ShmPointer, SampleMutSharedState<Service>> {
        let shared_state_guard = self.shared_state.state.lock();
        let guard = shared_state_guard.publisher_shared_state.lock();
        let allocation_strategy = guard.sender.data_segment.allocation_strategy();
        let reserved_header_len = guard.sender.message_type_details.all_headers_len();
        self.shared_state
            .state
            .lock()
            .slice_len
            .store(self.chunk.layout().size(), Ordering::Relaxed);

        ResizableMemoryBuilder::new(self.chunk.to_shm_pointer())
            .allocation_strategy(allocation_strategy)
            .initial_layout(self.chunk.layout())
            .reserved_header_len(reserved_header_len)
            .create(self.shared_state.clone())
    }

    #[doc(hidden)]
    pub fn __internal_finish_serialized(&mut self, payload_ptr: *const u8) {
        let message_type_details = self
            .shared_state
            .state
            .lock()
            .publisher_shared_state
            .lock()
            .sender
            .message_type_details;

        self.chunk.header = self
            .shared_state
            .state
            .lock()
            .shm_raw_ptr
            .load(Ordering::Relaxed) as *mut u8;
        self.chunk.user_header = message_type_details
            .user_header_ptr_from_header(self.chunk.header)
            .cast_mut();
        self.chunk.payload = message_type_details
            .payload_ptr_from_header(self.chunk.header)
            .cast_mut();

        let payload_offset = payload_ptr as usize - self.chunk.payload_ptr() as usize;

        let header = unsafe { &mut *self.chunk.header_mut_ptr().cast::<Header>() };
        header.number_of_elements = self.shared_state.slice_len() as u64;
        header.payload_offset = payload_offset as u64;
    }
}

impl<Service: crate::service::Service, Payload, UserHeader: ZeroCopySend>
    SampleMutUninit<Service, Flatbuffer<Payload>, UserHeader>
{
    pub(crate) fn new_flatbuffer(
        publisher_shared_state: &Service::ArcThreadSafetyPolicy<PublisherSharedState<Service>>,
        chunk: ChunkMut,
    ) -> Self {
        let mut new_self = Self {
            flatbuffer_builder: None,
            shared_state: SampleMutSharedState::new(
                publisher_shared_state,
                chunk.to_shm_pointer(),
                chunk.layout().size(),
            ),
            chunk,
            _payload: PhantomData,
            _user_header: PhantomData,
        };

        new_self.flatbuffer_builder = Some(FlatBufferBuilder::new_in(
            new_self.__internal_create_resizable_memory_builder(),
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
        root: WIPOffset<Payload>,
    ) -> SampleMut<Service, Flatbuffer<Payload>, UserHeader> {
        self.flatbuffer_builder().finish(root, None);
        let payload_ptr = self.flatbuffer_builder().finished_data().as_ptr();
        self.__internal_finish_serialized(payload_ptr);

        SampleMut {
            shared_state: self.shared_state,
            chunk: self.chunk,
            _payload: PhantomData,
            _user_header: PhantomData,
        }
    }
}

impl<
    Service: crate::service::Service,
    // It is important to restrict the Payload to ZeroCopySend since the flatbuffer builder
    // modifies the ptr to header and user header when growing.
    Payload: IceoryxSend + ZeroCopySend + Debug + ?Sized,
    UserHeader: ZeroCopySend,
> SampleMutUninit<Service, Payload, UserHeader>
{
    /// Returns a reference to the [`Header`] of the [`SampleMutUninit`].
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    ///
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// # let service = node.service_builder(&"My/Funk/ServiceName".try_into()?)
    /// #     .publish_subscribe::<u64>()
    /// #     .open_or_create()?;
    /// # let publisher = service.publisher_builder().create()?;
    ///
    /// let sample = publisher.loan_uninit()?;
    /// println!("Sample Publisher Origin {:?}", sample.header().publisher_id());
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn header(&self) -> &Header {
        unsafe { &*(self.chunk.header_ptr().cast()) }
    }

    /// Returns a reference to the user_header of the sample.
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    ///
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// # let service = node.service_builder(&"My/Funk/ServiceName".try_into()?)
    /// #     .publish_subscribe::<u64>()
    /// #     .user_header::<u64>()
    /// #     .open_or_create()?;
    /// # let publisher = service.publisher_builder().create()?;
    ///
    /// let sample = publisher.loan_uninit()?;
    /// println!("Sample Publisher Origin {:?}", sample.user_header());
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn user_header(&self) -> &UserHeader {
        unsafe { &*(self.chunk.user_header_ptr().cast()) }
    }

    /// Returns a mutable reference to the user_header of the sample.
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    ///
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// # let service = node.service_builder(&"My/Funk/ServiceName".try_into()?)
    /// #     .publish_subscribe::<u64>()
    /// #     .user_header::<u64>()
    /// #     .open_or_create()?;
    /// # let publisher = service.publisher_builder().create()?;
    ///
    /// let mut sample = publisher.loan_uninit()?;
    /// *sample.user_header_mut() = 123;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn user_header_mut(&mut self) -> &mut UserHeader {
        unsafe { &mut *(self.chunk.user_header_mut_ptr().cast()) }
    }
}

impl<
    Service: crate::service::Service,
    Payload: IceoryxSend + ZeroCopySend + Debug,
    UserHeader: ZeroCopySend,
> SampleMutUninit<Service, MaybeUninit<Payload>, UserHeader>
{
    pub(crate) fn new(
        publisher_shared_state: &Service::ArcThreadSafetyPolicy<PublisherSharedState<Service>>,
        chunk: ChunkMut,
    ) -> Self {
        Self {
            flatbuffer_builder: None,
            shared_state: SampleMutSharedState::new(
                publisher_shared_state,
                chunk.to_shm_pointer(),
                1,
            ),
            chunk,
            _payload: PhantomData,
            _user_header: PhantomData,
        }
    }

    /// Returns a reference to the payload of the sample.
    ///
    /// # Notes
    ///
    /// The generic parameter `Payload` is packed into a [`core::mem::MaybeUninit<Payload>`].
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    ///
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// # let service = node.service_builder(&"My/Funk/ServiceName".try_into()?)
    /// #     .publish_subscribe::<u64>()
    /// #     .open_or_create()?;
    /// # let publisher = service.publisher_builder().create()?;
    ///
    /// let mut sample = publisher.loan_uninit()?;
    /// sample.payload_mut().write(123);
    /// println!("Sample current payload {}", unsafe { sample.payload().assume_init_ref() });
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn payload(&self) -> &MaybeUninit<Payload> {
        unsafe { &*(self.chunk.payload_ptr().cast()) }
    }

    /// Returns a mutable reference to the payload of the sample.
    ///
    /// # Notes
    ///
    /// The generic parameter `Payload` is packed into a [`core::mem::MaybeUninit<Payload>`].
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    ///
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// # let service = node.service_builder(&"My/Funk/ServiceName".try_into()?)
    /// #     .publish_subscribe::<u64>()
    /// #     .open_or_create()?;
    /// # let publisher = service.publisher_builder().create()?;
    ///
    /// let mut sample = publisher.loan_uninit()?;
    /// sample.payload_mut().write(4567);
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn payload_mut(&mut self) -> &mut MaybeUninit<Payload> {
        unsafe { &mut *(self.chunk.payload_mut_ptr().cast()) }
    }

    /// Writes the payload to the sample and labels the sample as initialized
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// # let service = node.service_builder(&"My/Funk/ServiceName".try_into()?)
    /// #     .publish_subscribe::<u64>()
    /// #     .open_or_create()?;
    /// #
    /// # let publisher = service.publisher_builder().create()?;
    ///
    /// let sample = publisher.loan_uninit()?;
    /// let sample = sample.write_payload(1234);
    ///
    /// sample.send()?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn write_payload(mut self, value: Payload) -> SampleMut<Service, Payload, UserHeader>
    where
        Payload: ZeroCopySend,
    {
        self.payload_mut().write(value);
        unsafe { self.assume_init() }
    }

    /// Extracts the value of the [`core::mem::MaybeUninit<Payload>`] container and labels the sample as initialized
    ///
    /// # Safety
    ///
    /// The caller must ensure that [`core::mem::MaybeUninit<Payload>`] really is initialized. Calling this when
    /// the content is not fully initialized causes immediate undefined behavior.
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// # let service = node.service_builder(&"My/Funk/ServiceName".try_into()?)
    /// #     .publish_subscribe::<u64>()
    /// #     .open_or_create()?;
    /// #
    /// # let publisher = service.publisher_builder().create()?;
    ///
    /// let mut sample = publisher.loan_uninit()?;
    /// sample.payload_mut().write(1234);
    /// let sample = unsafe { sample.assume_init() };
    ///
    /// sample.send()?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub unsafe fn assume_init(self) -> SampleMut<Service, Payload, UserHeader> {
        // the transmute is not nice but safe since MaybeUninit is #[repr(transparent)] to the inner type
        SampleMut {
            shared_state: self.shared_state,
            chunk: self.chunk,
            _payload: PhantomData,
            _user_header: PhantomData,
        }
    }
}

impl<Service: crate::service::Service, Payload: Debug + ZeroCopySend, UserHeader: ZeroCopySend>
    SampleMutUninit<Service, [MaybeUninit<Payload>], UserHeader>
{
    pub(crate) fn new(
        publisher_shared_state: &Service::ArcThreadSafetyPolicy<PublisherSharedState<Service>>,
        chunk: ChunkMut,
        underyling_slice_len: usize,
    ) -> Self {
        Self {
            flatbuffer_builder: None,
            shared_state: SampleMutSharedState::new(
                publisher_shared_state,
                chunk.to_shm_pointer(),
                underyling_slice_len,
            ),
            chunk,
            _payload: PhantomData,
            _user_header: PhantomData,
        }
    }

    /// Returns a reference to the payload of the sample.
    ///
    /// # Notes
    ///
    /// The generic parameter `Payload` is packed into a [`core::mem::MaybeUninit<Payload>`].
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    ///
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// # let service = node.service_builder(&"My/Funk/ServiceName".try_into()?)
    /// #     .publish_subscribe::<[u64]>()
    /// #     .open_or_create()?;
    /// # let publisher = service.publisher_builder().create()?;
    ///
    /// let mut sample = publisher.loan_slice_uninit(1)?;
    /// sample.payload_mut()[0].write(123);
    /// println!("Sample current payload {:?}", unsafe { sample.payload().assume_init_ref() });
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn payload(&self) -> &[MaybeUninit<Payload>] {
        unsafe {
            &*core::ptr::slice_from_raw_parts(
                self.chunk.payload_ptr().cast(),
                self.shared_state.slice_len(),
            )
        }
    }

    /// Returns a mutable reference to the payload of the sample.
    ///
    /// # Notes
    ///
    /// The generic parameter `Payload` is packed into a [`core::mem::MaybeUninit<Payload>`].
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    ///
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// # let service = node.service_builder(&"My/Funk/ServiceName".try_into()?)
    /// #     .publish_subscribe::<[u64]>()
    /// #     .open_or_create()?;
    /// # let publisher = service.publisher_builder().create()?;
    ///
    /// let mut sample = publisher.loan_slice_uninit(1)?;
    /// sample.payload_mut()[0].write(4567);
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn payload_mut(&mut self) -> &mut [MaybeUninit<Payload>] {
        unsafe {
            &mut *core::ptr::slice_from_raw_parts_mut(
                self.chunk.payload_mut_ptr().cast(),
                self.shared_state.slice_len(),
            )
        }
    }

    /// Extracts the value of the slice of [`core::mem::MaybeUninit<Payload>`] and labels the sample as initialized
    ///
    /// # Safety
    ///
    /// The caller must ensure that every element of the slice of [`core::mem::MaybeUninit<Payload>`]
    /// is initialized. Calling this when the content is not fully initialized causes immediate undefined behavior.
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    /// use core::mem::MaybeUninit;
    ///
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// # let service = node.service_builder(&"My/Funk/ServiceName".try_into()?)
    /// #     .publish_subscribe::<[usize]>()
    /// #     .open_or_create()?;
    /// #
    /// # let publisher = service.publisher_builder().initial_max_slice_len(32).create()?;
    ///
    /// let slice_length = 10;
    /// let mut sample = publisher.loan_slice_uninit(slice_length)?;
    ///
    /// for element in sample.payload_mut() {
    ///     element.write(1234);
    /// }
    ///
    /// let sample = unsafe { sample.assume_init() };
    ///
    /// sample.send()?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub unsafe fn assume_init(self) -> SampleMut<Service, [Payload], UserHeader> {
        // the transmute is not nice but safe since MaybeUninit is #[repr(transparent)] to the inner type
        SampleMut {
            shared_state: self.shared_state,
            chunk: self.chunk,
            _payload: PhantomData,
            _user_header: PhantomData,
        }
    }

    /// Writes the payload to the sample and labels the sample as initialized
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// # let service = node.service_builder(&"My/Funk/ServiceName".try_into()?)
    /// #     .publish_subscribe::<[usize]>()
    /// #     .open_or_create()?;
    /// #
    /// # let publisher = service.publisher_builder().initial_max_slice_len(16).create()?;
    ///
    /// let slice_length = 12;
    /// let sample = publisher.loan_slice_uninit(slice_length)?;
    /// let sample = sample.write_from_fn(|n| n + 123);
    ///
    /// sample.send()?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn write_from_fn<F: FnMut(usize) -> Payload>(
        mut self,
        mut initializer: F,
    ) -> SampleMut<Service, [Payload], UserHeader> {
        for (i, element) in self.payload_mut().iter_mut().enumerate() {
            element.write(initializer(i));
        }

        // SAFETY: this is safe since the payload was initialized on the line above
        unsafe { self.assume_init() }
    }
}

impl<
    Service: crate::service::Service,
    Payload: Debug + Copy + ZeroCopySend,
    UserHeader: ZeroCopySend,
> SampleMutUninit<Service, [MaybeUninit<Payload>], UserHeader>
{
    /// Writes the payload by mem copying the provided slice into the [`SampleMutUninit`].
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// # let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// #
    /// # let service = node.service_builder(&"My/Funk/ServiceName".try_into()?)
    /// #     .publish_subscribe::<[usize]>()
    /// #     .open_or_create()?;
    /// #
    /// # let publisher = service.publisher_builder().initial_max_slice_len(16).create()?;
    ///
    /// let slice_length = 3;
    /// let sample = publisher.loan_slice_uninit(slice_length)?;
    /// let sample = sample.write_from_slice(&vec![1, 2, 3]);
    ///
    /// sample.send()?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn write_from_slice(
        mut self,
        value: &[Payload],
    ) -> SampleMut<Service, [Payload], UserHeader> {
        self.payload_mut().copy_from_slice(unsafe {
            core::mem::transmute::<&[Payload], &[MaybeUninit<Payload>]>(value)
        });
        unsafe { self.assume_init() }
    }
}
