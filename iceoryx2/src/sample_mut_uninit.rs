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

use core::{fmt::Debug, mem::MaybeUninit};
use iceoryx2_bb_concurrency::atomic::Ordering;
use iceoryx2_bb_flatbuffers::{ResizableMemory, ResizableMemoryBuilder};
use iceoryx2_cal::arc_sync_policy::ArcSyncPolicy;

use flatbuffers::{FlatBufferBuilder, WIPOffset};
use iceoryx2_bb_elementary_traits::{iceoryx_send::IceoryxSend, zero_copy_send::ZeroCopySend};
use iceoryx2_cal::shared_memory::ShmPointer;

use crate::port::details::chunk::ChunkMut;
use crate::service::static_config::message_type_details::{MessageTypeDetails, TypeVariant};
use crate::{
    port::publisher::PublisherSharedState,
    raw_sample::RawSampleMut,
    sample_mut::{SampleMut, SampleMutSharedState},
    service::{header::publish_subscribe::Header, marker::Flatbuffer},
};

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
    ptr: RawSampleMut<Header, UserHeader, Payload>,
    sample_size: usize,
    flatbuffer_builder: Option<
        FlatBufferBuilder<'static, ResizableMemory<ShmPointer, SampleMutSharedState<Service>>>,
    >,
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

/// The memory used inside the [`FlatbufferBuilder`].
pub type FlatbufferMemory<Service> = ResizableMemory<ShmPointer, SampleMutSharedState<Service>>;

impl<Service: crate::service::Service, Payload, UserHeader: ZeroCopySend>
    SampleMutUninit<Service, Flatbuffer<Payload>, UserHeader>
{
    pub(crate) fn new_flatbuffer(
        publisher_shared_state: &Service::ArcThreadSafetyPolicy<PublisherSharedState<Service>>,
        chunk: ChunkMut,
        ptr: RawSampleMut<Header, UserHeader, Flatbuffer<Payload>>,
    ) -> Self {
        let shared_state = SampleMutSharedState::new(
            publisher_shared_state,
            chunk.to_shm_pointer(),
            chunk.layout().size(),
        );
        let allocation_strategy = publisher_shared_state
            .lock()
            .sender
            .data_segment
            .allocation_strategy();
        let reserved_header_len = MessageTypeDetails::from::<
            crate::service::header::publish_subscribe::Header,
            UserHeader,
            u8,
        >(TypeVariant::Dynamic)
        .all_headers_len();

        let resizable_memory = ResizableMemoryBuilder::new(chunk.to_shm_pointer())
            .allocation_strategy(allocation_strategy)
            .initial_layout(chunk.layout())
            .reserved_header_len(reserved_header_len)
            .create(shared_state.clone())
            .unwrap();

        Self {
            flatbuffer_builder: Some(FlatBufferBuilder::new_in(resizable_memory)),
            shared_state,
            ptr,
            sample_size: chunk.layout().size(),
        }
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
        let flatbuffer_payload_start = self.flatbuffer_builder().finished_data().as_ptr() as usize;

        let message_type_details = MessageTypeDetails::from::<
            crate::service::header::publish_subscribe::Header,
            UserHeader,
            u8,
        >(TypeVariant::Dynamic);

        let state = self.shared_state.state.lock();
        let header = state.shm_raw_ptr.load(Ordering::Relaxed) as *mut u8;
        let user_header = message_type_details
            .user_header_ptr_from_header(header)
            .cast_mut();
        let payload = message_type_details
            .payload_ptr_from_header(header)
            .cast_mut();

        let payload_offset = flatbuffer_payload_start - payload as usize;
        let mut ptr: RawSampleMut<Header, UserHeader, Flatbuffer<Payload>> = unsafe {
            RawSampleMut::new_unchecked(header.cast(), user_header.cast(), payload.cast())
        };

        ptr.as_header_mut().metadata = payload_offset as u64;
        ptr.as_header_mut().number_of_elements =
            state.number_of_elements.load(Ordering::Relaxed) as u64;
        drop(state);

        SampleMut {
            shared_state: self.shared_state,
            sample_size: self.sample_size,
            ptr,
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
        self.ptr.as_header_ref()
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
        self.ptr.as_user_header_ref()
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
        self.ptr.as_user_header_mut()
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
    pub fn payload(&self) -> &Payload {
        self.ptr.as_payload_ref()
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
    pub fn payload_mut(&mut self) -> &mut Payload
    where
        Payload: ZeroCopySend,
    {
        self.ptr.as_payload_mut()
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
        ptr: RawSampleMut<Header, UserHeader, MaybeUninit<Payload>>,
        shm_pointer: ShmPointer,
        sample_size: usize,
    ) -> Self {
        Self {
            flatbuffer_builder: None,
            shared_state: SampleMutSharedState::new(publisher_shared_state, shm_pointer, 1),
            ptr,
            sample_size,
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
            sample_size: self.sample_size,
            ptr: unsafe { self.ptr.assume_init() },
        }
    }
}

impl<Service: crate::service::Service, Payload: Debug + ZeroCopySend, UserHeader: ZeroCopySend>
    SampleMutUninit<Service, [MaybeUninit<Payload>], UserHeader>
{
    pub(crate) fn new(
        publisher_shared_state: &Service::ArcThreadSafetyPolicy<PublisherSharedState<Service>>,
        ptr: RawSampleMut<Header, UserHeader, [MaybeUninit<Payload>]>,
        shm_pointer: ShmPointer,
        sample_size: usize,
    ) -> Self {
        Self {
            flatbuffer_builder: None,
            shared_state: SampleMutSharedState::new(
                publisher_shared_state,
                shm_pointer,
                ptr.as_payload_ref().len(),
            ),
            ptr,
            sample_size,
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
            sample_size: self.sample_size,
            ptr: unsafe { self.ptr.assume_init() },
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
