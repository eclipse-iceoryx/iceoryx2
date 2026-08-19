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
//! # fn main() -> Result<(), Box<dyn core::error::Error>> {
//! # let node = NodeBuilder::new().create::<ipc::Service>()?;
//! # let service = node.service_builder(&"My/Funk/ServiceName".try_into()?)
//! #   .publish_subscribe::<u64>()
//! #   .open_or_create()?;
//! # let subscriber = service.subscriber_builder().create()?;
//!
//! while let Some(sample) = subscriber.receive()? {
//!     println!("received: {:?}", *sample);
//!     println!("header publisher id {:?}", sample.header().publisher_id());
//! }
//!
//! # Ok(())
//! # }
//! ```

use core::marker::PhantomData;
use core::{fmt::Debug, ops::Deref};

use flatbuffers::InvalidFlatbuffer;
use iceoryx2_bb_elementary_traits::iceoryx_send::IceoryxSend;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use iceoryx2_bb_flatbuffers::FlatbufferError;
use iceoryx2_cal::arc_sync_policy::ArcSyncPolicy;
use iceoryx2_cal::unique_id_generator::UniqueId;
use iceoryx2_cal::zero_copy_connection::ChannelId;

use crate::identifiers::UniquePublisherId;
use crate::payload::number_of_elements;
use crate::port::details::chunk::Chunk;
use crate::port::details::chunk_details::ChunkDetails;
use crate::port::subscriber::SubscriberSharedState;
use crate::service::header::publish_subscribe::Header;
use crate::service::marker::Flatbuffer;

/// It stores the payload and is acquired by the [`Subscriber`](crate::port::subscriber::Subscriber) whenever
/// it receives new data from a [`Publisher`](crate::port::publisher::Publisher) via
/// [`Subscriber::receive()`](crate::port::subscriber::Subscriber::receive()).
pub struct Sample<
    Service: crate::service::Service,
    Payload: IceoryxSend + Debug + ?Sized,
    UserHeader: ZeroCopySend,
> {
    pub(crate) chunk: Chunk,
    pub(crate) subscriber_shared_state:
        Service::ArcThreadSafetyPolicy<SubscriberSharedState<Service>>,
    pub(crate) details: ChunkDetails,
    pub(crate) _payload: PhantomData<Payload>,
    pub(crate) _user_header: PhantomData<UserHeader>,
}

unsafe impl<
    Service: crate::service::Service,
    Payload: IceoryxSend + Debug + ?Sized,
    UserHeader: ZeroCopySend,
> Send for Sample<Service, Payload, UserHeader>
where
    Service::ArcThreadSafetyPolicy<SubscriberSharedState<Service>>: Send + Sync,
{
}

impl<
    Service: crate::service::Service,
    Payload: IceoryxSend + Debug + ?Sized,
    UserHeader: ZeroCopySend,
> Debug for Sample<Service, Payload, UserHeader>
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Sample<{}, {}, {}> {{ chunk: {:?}, details: {:?} }}",
            core::any::type_name::<Payload>(),
            core::any::type_name::<UserHeader>(),
            core::any::type_name::<Service>(),
            self.chunk,
            self.details,
        )
    }
}

impl<
    Service: crate::service::Service,
    Payload: IceoryxSend + Debug + ZeroCopySend,
    UserHeader: ZeroCopySend,
> Deref for Sample<Service, Payload, UserHeader>
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
> Deref for Sample<Service, [Payload], UserHeader>
{
    type Target = [Payload];
    fn deref(&self) -> &Self::Target {
        let payload_size = self.subscriber_shared_state.lock().receiver.payload_size();
        unsafe {
            &*core::ptr::slice_from_raw_parts(
                self.chunk.payload_ptr().cast(),
                number_of_elements::<Payload, _>(self.header(), payload_size),
            )
        }
    }
}

impl<
    Service: crate::service::Service,
    Payload: IceoryxSend + Debug + ?Sized,
    UserHeader: ZeroCopySend,
> Drop for Sample<Service, Payload, UserHeader>
{
    fn drop(&mut self) {
        self.subscriber_shared_state
            .lock()
            .receiver
            .release_offset(&self.details, ChannelId::new(0));
    }
}

impl<Service: crate::service::Service, Payload: Debug, UserHeader: ZeroCopySend>
    Sample<Service, Flatbuffer<Payload>, UserHeader>
{
    /// Returns the serialized flatbuffer data as bytes.
    pub fn payload_bytes(&self) -> &[u8] {
        let payload_offset = self.header().payload_offset() as usize;
        let payload_ptr = self.chunk.payload_ptr();
        let payload_len = self.header().number_of_elements as usize;

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
    Payload: IceoryxSend + Debug + ZeroCopySend,
    UserHeader: ZeroCopySend,
> Sample<Service, Payload, UserHeader>
{
    /// Returns a reference to the payload of the [`Sample`]
    pub fn payload(&self) -> &Payload {
        self.deref()
    }
}

impl<
    Service: crate::service::Service,
    Payload: IceoryxSend + Debug + ZeroCopySend,
    UserHeader: ZeroCopySend,
> Sample<Service, [Payload], UserHeader>
{
    /// Returns a reference to the payload of the [`Sample`]
    pub fn payload(&self) -> &[Payload] {
        self.deref()
    }
}

impl<
    Service: crate::service::Service,
    Payload: IceoryxSend + Debug + ?Sized,
    UserHeader: ZeroCopySend,
> Sample<Service, Payload, UserHeader>
{
    /// Returns a reference to the user_header of the [`Sample`]
    pub fn user_header(&self) -> &UserHeader {
        unsafe { &*self.chunk.user_header_ptr().cast() }
    }

    /// Returns a reference to the [`Header`] of the [`Sample`].
    pub fn header(&self) -> &Header {
        unsafe { &*self.chunk.header_ptr().cast() }
    }

    /// Returns the [`UniquePublisherId`] of the [`Publisher`](crate::port::publisher::Publisher)
    pub fn origin(&self) -> UniquePublisherId {
        UniquePublisherId(unsafe { UniqueId::from_value(self.details.origin) })
    }
}
