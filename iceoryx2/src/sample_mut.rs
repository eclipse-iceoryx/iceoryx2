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
//! // initializes the payload with `Default::default()`
//! let mut sample = publisher.loan()?;
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
//! ## Slice API
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
//! // initializes every element of the slice with `Default::default()`
//! let mut sample = publisher.loan_slice(slice_length)?;
//! // override the content of the first element with 42
//! sample.payload_mut()[0] = 42;
//!
//! println!("publisher port id: {:?}", sample.header().publisher_id());
//! sample.send()?;
//!
//! # Ok(())
//! # }
//! ```

use crate::{
    port::publisher::PublisherSharedState, port::SendError, raw_sample::RawSampleMut,
    service::header::publish_subscribe::Header,
};
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use iceoryx2_cal::arc_sync_policy::ArcSyncPolicy;
use iceoryx2_cal::shared_memory::*;

use core::fmt::{Debug, Formatter};
use core::ops::{Deref, DerefMut};

/// Acquired by a [`crate::port::publisher::Publisher`] via
///  * [`crate::port::publisher::Publisher::loan()`],
///  * [`crate::port::publisher::Publisher::loan_slice()`]
///
/// It stores the payload that will be sent
/// to all connected [`crate::port::subscriber::Subscriber`]s. If the [`SampleMut`] is not sent
/// it will release the loaned memory when going out of scope.
pub struct SampleMut<
    Service: crate::service::Service,
    Payload: Debug + ZeroCopySend + ?Sized,
    UserHeader: ZeroCopySend,
> {
    pub(crate) publisher_shared_state:
        Service::ArcThreadSafetyPolicy<PublisherSharedState<Service>>,
    pub(crate) ptr: RawSampleMut<Header, UserHeader, Payload>,
    pub(crate) offset_to_chunk: PointerOffset,
    pub(crate) sample_size: usize,
}

unsafe impl<
        Service: crate::service::Service,
        Payload: Debug + ZeroCopySend + ?Sized,
        UserHeader: ZeroCopySend,
    > Send for SampleMut<Service, Payload, UserHeader>
where
    Service::ArcThreadSafetyPolicy<PublisherSharedState<Service>>: Send + Sync,
{
}

impl<
        Service: crate::service::Service,
        Payload: Debug + ZeroCopySend + ?Sized,
        UserHeader: ZeroCopySend,
    > Deref for SampleMut<Service, Payload, UserHeader>
{
    type Target = Payload;
    fn deref(&self) -> &Self::Target {
        self.ptr.as_payload_ref()
    }
}

impl<
        Service: crate::service::Service,
        Payload: Debug + ZeroCopySend + ?Sized,
        UserHeader: ZeroCopySend,
    > DerefMut for SampleMut<Service, Payload, UserHeader>
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.ptr.as_payload_mut()
    }
}

impl<
        Service: crate::service::Service,
        Payload: Debug + ZeroCopySend + ?Sized,
        UserHeader: ZeroCopySend,
    > Debug for SampleMut<Service, Payload, UserHeader>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "SampleMut<{}, {}, {}> {{ publisher_shared_state: {:?}, offset_to_chunk: {:?}, sample_size: {} }}",
            core::any::type_name::<Service>(),
            core::any::type_name::<Payload>(),
            core::any::type_name::<UserHeader>(),
            self.publisher_shared_state,
            self.offset_to_chunk,
            self.sample_size
        )
    }
}

impl<
        Service: crate::service::Service,
        Payload: Debug + ZeroCopySend + ?Sized,
        UserHeader: ZeroCopySend,
    > Drop for SampleMut<Service, Payload, UserHeader>
{
    fn drop(&mut self) {
        self.publisher_shared_state
            .lock()
            .sender
            .return_loaned_sample(self.offset_to_chunk);
    }
}

impl<
        Service: crate::service::Service,
        M: Debug + ZeroCopySend + ?Sized, // `M` is either a `Payload` or a `MaybeUninit<Payload>`
        UserHeader: ZeroCopySend,
    > SampleMut<Service, M, UserHeader>
{
    /// Returns a reference to the header of the sample.
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
    /// let sample = publisher.loan()?;
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
    /// let sample = publisher.loan()?;
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
    /// let mut sample = publisher.loan()?;
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
    /// The generic parameter `Payload` can be packed into [`core::mem::MaybeUninit<Payload>`], depending
    /// which API is used to obtain the sample. Obtaining a reference is safe for either type.
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
    /// let sample = publisher.loan()?;
    /// println!("Sample current payload {}", sample.payload());
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn payload(&self) -> &M {
        self.ptr.as_payload_ref()
    }

    /// Returns a mutable reference to the payload of the sample.
    ///
    /// # Notes
    ///
    /// The generic parameter `Payload` can be packed into [`core::mem::MaybeUninit<Payload>`], depending
    /// which API is used to obtain the sample. Obtaining a reference is safe for either type.
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
    /// let mut sample = publisher.loan()?;
    /// *sample.payload_mut() = 4567;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn payload_mut(&mut self) -> &mut M {
        self.ptr.as_payload_mut()
    }

    /// Send a previously loaned [`crate::port::publisher::Publisher::loan_uninit()`] or
    /// [`crate::port::publisher::Publisher::loan()`] [`SampleMut`] to all connected
    /// [`crate::port::subscriber::Subscriber`]s of the service.
    ///
    /// On success the number of [`crate::port::subscriber::Subscriber`]s that received
    /// the data is returned, otherwise a [`SendError`] describing the failure.
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
    /// let mut sample = publisher.loan()?;
    /// *sample.payload_mut() = 4567;
    ///
    /// sample.send()?;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn send(self) -> Result<usize, SendError> {
        self.publisher_shared_state
            .lock()
            .send_sample(self.offset_to_chunk, self.sample_size)
    }
}
