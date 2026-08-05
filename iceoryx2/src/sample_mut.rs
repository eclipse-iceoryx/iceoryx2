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

use crate::port::details::chunk::ChunkMut;
use crate::port::details::chunk_mut_shared_state::ChunkMutSharedState;
use crate::service::marker::Flatbuffer;
use crate::{
    port::SendError, port::publisher::PublisherSharedState,
    service::header::publish_subscribe::Header,
};
use flatbuffers::InvalidFlatbuffer;
use iceoryx2_bb_elementary_traits::iceoryx_send::IceoryxSend;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use iceoryx2_bb_flatbuffers::FlatbufferError;

use core::fmt::{Debug, Formatter};
use core::marker::PhantomData;
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
    Payload: IceoryxSend + Debug + ?Sized,
    UserHeader: ZeroCopySend,
> {
    pub(crate) shared_state: ChunkMutSharedState<Service, PublisherSharedState<Service>>,
    pub(crate) chunk: ChunkMut,
    pub(crate) _payload: PhantomData<Payload>,
    pub(crate) _user_header: PhantomData<UserHeader>,
}

unsafe impl<
    Service: crate::service::Service,
    Payload: IceoryxSend + Debug + ?Sized,
    UserHeader: ZeroCopySend,
> Send for SampleMut<Service, Payload, UserHeader>
where
    Service::ArcThreadSafetyPolicy<PublisherSharedState<Service>>: Send + Sync,
{
}

impl<
    Service: crate::service::Service,
    Payload: IceoryxSend + Debug + ZeroCopySend,
    UserHeader: ZeroCopySend,
> Deref for SampleMut<Service, Payload, UserHeader>
{
    type Target = Payload;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.chunk.payload_ptr().cast() }
    }
}

impl<
    Service: crate::service::Service,
    Payload: IceoryxSend + Debug + ZeroCopySend,
    UserHeader: ZeroCopySend,
> Deref for SampleMut<Service, [Payload], UserHeader>
{
    type Target = [Payload];
    fn deref(&self) -> &Self::Target {
        unsafe {
            &*core::ptr::slice_from_raw_parts(
                self.chunk.payload_ptr().cast(),
                self.header().number_of_elements() as usize,
            )
        }
    }
}

impl<
    Service: crate::service::Service,
    Payload: IceoryxSend + Debug + ZeroCopySend,
    UserHeader: ZeroCopySend,
> DerefMut for SampleMut<Service, Payload, UserHeader>
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.chunk.payload_mut_ptr().cast() }
    }
}

impl<
    Service: crate::service::Service,
    Payload: IceoryxSend + Debug + ZeroCopySend,
    UserHeader: ZeroCopySend,
> DerefMut for SampleMut<Service, [Payload], UserHeader>
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            &mut *core::ptr::slice_from_raw_parts_mut(
                self.chunk.payload_mut_ptr().cast(),
                self.header().number_of_elements() as usize,
            )
        }
    }
}

impl<
    Service: crate::service::Service,
    Payload: IceoryxSend + Debug + ?Sized,
    UserHeader: ZeroCopySend,
> Debug for SampleMut<Service, Payload, UserHeader>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "SampleMut<{}, {}, {}> {{ shared_state: {:?}, chunk: {:?} }}",
            core::any::type_name::<Service>(),
            core::any::type_name::<Payload>(),
            core::any::type_name::<UserHeader>(),
            self.shared_state,
            self.chunk
        )
    }
}

impl<Service: crate::service::Service, Payload: Debug, UserHeader: ZeroCopySend>
    SampleMut<Service, Flatbuffer<Payload>, UserHeader>
{
    /// Returns the serialized flatbuffer data as bytes.
    pub fn payload_bytes(&self) -> &[u8] {
        let payload_offset = self.header().payload_offset() as usize;
        let payload_ptr = self.chunk.payload_ptr();
        let payload_len = self.header().number_of_elements() as usize;

        unsafe {
            core::slice::from_raw_parts(
                payload_ptr.add(payload_offset),
                payload_len - payload_offset,
            )
        }
    }

    /// Returns the root of the flatbuffer.
    pub fn payload_root<'a>(&'a self) -> Result<Payload::Inner, FlatbufferError<InvalidFlatbuffer>>
    where
        Payload: flatbuffers::Follow<'a> + flatbuffers::Verifiable,
    {
        Ok(flatbuffers::root::<Payload>(self.payload_bytes())?)
    }
}

impl<
    Service: crate::service::Service,
    Payload: IceoryxSend + Debug + ?Sized,
    UserHeader: ZeroCopySend,
> SampleMut<Service, Payload, UserHeader>
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
        unsafe { &*self.chunk.header_ptr().cast() }
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
        unsafe { &*self.chunk.user_header_ptr().cast() }
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
        unsafe { &mut *self.chunk.user_header_mut_ptr().cast() }
    }

    /// Send a [`SampleMut`], previously loaned with [`crate::port::publisher::Publisher::loan_uninit()`] or
    /// [`crate::port::publisher::Publisher::loan()`] to all connected
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
        self.shared_state
            .shared_state(|sample_state, publisher_shared_state| {
                publisher_shared_state
                    .send_sample(sample_state.offset_to_chunk(), self.chunk.layout().size())
            })
    }
}

impl<
    Service: crate::service::Service,
    Payload: IceoryxSend + ZeroCopySend + Debug,
    UserHeader: ZeroCopySend,
> SampleMut<Service, Payload, UserHeader>
{
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
    pub fn payload(&self) -> &Payload {
        unsafe { &*self.chunk.payload_ptr().cast() }
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
    pub fn payload_mut(&mut self) -> &mut Payload {
        unsafe { &mut *self.chunk.payload_mut_ptr().cast() }
    }
}

impl<
    Service: crate::service::Service,
    Payload: IceoryxSend + ZeroCopySend + Debug,
    UserHeader: ZeroCopySend,
> SampleMut<Service, [Payload], UserHeader>
{
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
    /// #     .publish_subscribe::<[u64]>()
    /// #     .open_or_create()?;
    /// # let publisher = service.publisher_builder().create()?;
    ///
    /// let sample = publisher.loan_slice(1)?;
    /// println!("Sample current payload {}", sample.payload()[0]);
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn payload(&self) -> &[Payload] {
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
    /// #     .publish_subscribe::<[u64]>()
    /// #     .open_or_create()?;
    /// # let publisher = service.publisher_builder().create()?;
    ///
    /// let mut sample = publisher.loan_slice(1)?;
    /// sample.payload_mut()[0] = 4567;
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn payload_mut(&mut self) -> &mut [Payload] {
        unsafe {
            &mut *core::ptr::slice_from_raw_parts_mut(
                self.chunk.payload_mut_ptr().cast(),
                self.shared_state.slice_len(),
            )
        }
    }
}
