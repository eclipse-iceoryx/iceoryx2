// Copyright (c) 2023 Contributors to the Eclipse Foundation
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
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # let service_name = ServiceName::new("My/Funk/ServiceName").unwrap();
//! #
//! # let service = zero_copy::Service::new(&service_name)
//! #     .publish_subscribe()
//! #     .typed::<u64>()
//! #     .open_or_create()?;
//! #
//! # let publisher = service.publisher().create()?;
//!
//! let sample = publisher.loan_uninit()?;
//! let sample = sample.write_payload(1234);
//!
//! println!("publisher port id: {:?}", sample.header().publisher_id());
//! sample.send()?;
//!
//! # Ok(())
//! # }
//! ```

use crate::{
    port::publisher::{DataSegment, PublisherSendError},
    raw_sample::RawSampleMut,
    service::header::publish_subscribe::Header,
};
use iceoryx2_cal::shared_memory::*;
use std::{
    fmt::{Debug, Formatter},
    mem::MaybeUninit,
    sync::Arc,
};

/// Acquired by a [`crate::port::publisher::Publisher`] via
/// [`crate::port::publisher::Publisher::loan()`] or
/// [`crate::port::publisher::Publisher::loan_uninit()`]. It stores the payload that will be sent
/// to all connected [`crate::port::subscriber::Subscriber`]s. If the [`SampleMut`] is not sent
/// it will release the loaned memory when going out of scope.
///
/// # Notes
///
/// Does not implement [`Send`] since it releases unsent samples in the [`crate::port::publisher::Publisher`] and the
/// [`crate::port::publisher::Publisher`] is not thread-safe!
///
/// The generic parameter `M` is either a `MessageType` or a [`core::mem::MaybeUninit<MessageType>`], depending
/// which API is used to obtain the sample.
pub struct SampleMut<MessageType: Debug + ?Sized, Service: crate::service::Service> {
    data_segment: Arc<DataSegment<Service>>,
    ptr: RawSampleMut<Header, MessageType>,
    pub(crate) offset_to_chunk: PointerOffset,
}

impl<MessageType: Debug + ?Sized, Service: crate::service::Service> Debug
    for SampleMut<MessageType, Service>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SampleMut<{}, {}> {{ data_segment: {:?}, offset_to_chunk: {:?} }}",
            core::any::type_name::<MessageType>(),
            core::any::type_name::<Service>(),
            self.data_segment,
            self.offset_to_chunk
        )
    }
}

impl<MessageType: Debug + ?Sized, Service: crate::service::Service> Drop
    for SampleMut<MessageType, Service>
{
    fn drop(&mut self) {
        self.data_segment.return_loaned_sample(self.offset_to_chunk);
    }
}

impl<MessageType: Debug, Service: crate::service::Service>
    SampleMut<MaybeUninit<MessageType>, Service>
{
    pub(crate) fn new(
        data_segment: &Arc<DataSegment<Service>>,
        ptr: RawSampleMut<Header, MaybeUninit<MessageType>>,
        offset_to_chunk: PointerOffset,
    ) -> Self {
        Self {
            data_segment: Arc::clone(data_segment),
            ptr,
            offset_to_chunk,
        }
    }
}

impl<MessageType: Debug, Service: crate::service::Service>
    SampleMut<MaybeUninit<MessageType>, Service>
{
    /// Writes the payload to the sample and labels the sample as initialized
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let service_name = ServiceName::new("My/Funk/ServiceName").unwrap();
    /// #
    /// # let service = zero_copy::Service::new(&service_name)
    /// #     .publish_subscribe()
    /// #     .typed::<u64>()
    /// #     .open_or_create()?;
    /// #
    /// # let publisher = service.publisher().create()?;
    ///
    /// let sample = publisher.loan_uninit()?;
    /// let sample = sample.write_payload(1234);
    ///
    /// sample.send()?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn write_payload(mut self, value: MessageType) -> SampleMut<MessageType, Service> {
        self.payload_mut().write(value);
        // SAFETY: this is safe since the payload was initialized on the line above
        unsafe { self.assume_init() }
    }

    /// Extracts the value of the [`core::mem::MaybeUninit<MessageType>`] container and labels the sample as initialized
    ///
    /// # Safety
    ///
    /// The caller must ensure that [`core::mem::MaybeUninit<MessageType>`] really is initialized. Calling this when
    /// the content is not fully initialized causes immediate undefined behavior.
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let service_name = ServiceName::new("My/Funk/ServiceName").unwrap();
    /// #
    /// # let service = zero_copy::Service::new(&service_name)
    /// #     .publish_subscribe()
    /// #     .typed::<u64>()
    /// #     .open_or_create()?;
    /// #
    /// # let publisher = service.publisher().create()?;
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
    pub unsafe fn assume_init(self) -> SampleMut<MessageType, Service> {
        // the transmute is not nice but safe since MaybeUninit is #[repr(transparent)] to the inner type
        std::mem::transmute(self)
    }
}

impl<
        M: Debug + ?Sized, // `M` is either a `MessageType` or a `MaybeUninit<MessageType>`
        Service: crate::service::Service,
    > SampleMut<M, Service>
{
    /// Returns a reference to the header of the sample.
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let service_name = ServiceName::new("My/Funk/ServiceName").unwrap();
    /// #
    /// # let service = zero_copy::Service::new(&service_name)
    /// #     .publish_subscribe()
    /// #     .typed::<u64>()
    /// #     .open_or_create()?;
    /// # let publisher = service.publisher().create()?;
    ///
    /// let sample = publisher.loan()?;
    /// println!("Sample Publisher Origin {:?}", sample.header().publisher_id());
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn header(&self) -> &Header {
        self.ptr.as_header_ref()
    }

    /// Returns a reference to the payload of the sample.
    ///
    /// # Notes
    ///
    /// The generic parameter `MessageType` can be packed into [`core::mem::MaybeUninit<MessageType>`], depending
    /// which API is used to obtain the sample. Obtaining a reference is safe for either type.
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let service_name = ServiceName::new("My/Funk/ServiceName").unwrap();
    /// #
    /// # let service = zero_copy::Service::new(&service_name)
    /// #     .publish_subscribe()
    /// #     .typed::<u64>()
    /// #     .open_or_create()?;
    /// # let publisher = service.publisher().create()?;
    ///
    /// let sample = publisher.loan()?;
    /// println!("Sample current payload {}", sample.payload());
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn payload(&self) -> &M {
        self.ptr.as_data_ref()
    }

    /// Returns a mutable reference to the payload of the sample.
    ///
    /// # Notes
    ///
    /// The generic parameter `MessageType` can be packed into [`core::mem::MaybeUninit<MessageType>`], depending
    /// which API is used to obtain the sample. Obtaining a reference is safe for either type.
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let service_name = ServiceName::new("My/Funk/ServiceName").unwrap();
    /// #
    /// # let service = zero_copy::Service::new(&service_name)
    /// #     .publish_subscribe()
    /// #     .typed::<u64>()
    /// #     .open_or_create()?;
    /// # let publisher = service.publisher().create()?;
    ///
    /// let mut sample = publisher.loan()?;
    /// *sample.payload_mut() = 4567;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn payload_mut(&mut self) -> &mut M {
        self.ptr.as_data_mut()
    }

    /// Send a previously loaned [`crate::port::publisher::Publisher::loan_uninit()`] or
    /// [`crate::port::publisher::Publisher::loan()`] [`SampleMut`] to all connected
    /// [`crate::port::subscriber::Subscriber`]s of the service.
    ///
    /// The payload of the [`SampleMut`] must be initialized before it can be sent. Have a look
    /// at [`SampleMut::write_payload()`] and [`SampleMut::assume_init()`]
    /// for more details.
    ///
    /// On success the number of [`crate::port::subscriber::Subscriber`]s that received
    /// the data is returned, otherwise a [`PublisherSendError`] describing the failure.
    pub fn send(self) -> Result<usize, PublisherSendError> {
        self.data_segment.send_sample(self.offset_to_chunk.value())
    }
}
