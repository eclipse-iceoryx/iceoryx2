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

use core::{fmt::Debug, ops::Deref};

use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use iceoryx2_bb_posix::unique_system_id::UniqueSystemId;
use iceoryx2_cal::arc_sync_policy::ArcSyncPolicy;
use iceoryx2_cal::zero_copy_connection::ChannelId;

use crate::port::details::chunk_details::ChunkDetails;
use crate::port::port_identifiers::UniquePublisherId;
use crate::port::subscriber::SubscriberSharedState;
use crate::raw_sample::RawSample;
use crate::service::header::publish_subscribe::Header;

/// It stores the payload and is acquired by the [`Subscriber`](crate::port::subscriber::Subscriber) whenever
/// it receives new data from a [`Publisher`](crate::port::publisher::Publisher) via
/// [`Subscriber::receive()`](crate::port::subscriber::Subscriber::receive()).
pub struct Sample<
    Service: crate::service::Service,
    Payload: Debug + ?Sized + ZeroCopySend,
    UserHeader: ZeroCopySend,
> {
    pub(crate) ptr: RawSample<Header, UserHeader, Payload>,
    pub(crate) subscriber_shared_state:
        Service::ArcThreadSafetyPolicy<SubscriberSharedState<Service>>,
    pub(crate) details: ChunkDetails,
}

unsafe impl<
        Service: crate::service::Service,
        Payload: Debug + ZeroCopySend + ?Sized,
        UserHeader: ZeroCopySend,
    > Send for Sample<Service, Payload, UserHeader>
where
    Service::ArcThreadSafetyPolicy<SubscriberSharedState<Service>>: Send + Sync,
{
}

impl<
        Service: crate::service::Service,
        Payload: Debug + ZeroCopySend + ?Sized,
        UserHeader: ZeroCopySend,
    > Debug for Sample<Service, Payload, UserHeader>
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Sample<{}, {}, {}> {{ ptr: {:?}, details: {:?} }}",
            core::any::type_name::<Payload>(),
            core::any::type_name::<UserHeader>(),
            core::any::type_name::<Service>(),
            self.ptr,
            self.details,
        )
    }
}

impl<
        Service: crate::service::Service,
        Payload: Debug + ZeroCopySend + ?Sized,
        UserHeader: ZeroCopySend,
    > Deref for Sample<Service, Payload, UserHeader>
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
    > Drop for Sample<Service, Payload, UserHeader>
{
    fn drop(&mut self) {
        self.subscriber_shared_state
            .lock()
            .receiver
            .release_offset(&self.details, ChannelId::new(0));
    }
}

impl<
        Service: crate::service::Service,
        Payload: Debug + ZeroCopySend + ?Sized,
        UserHeader: ZeroCopySend,
    > Sample<Service, Payload, UserHeader>
{
    /// Returns a reference to the payload of the [`Sample`]
    pub fn payload(&self) -> &Payload {
        self.ptr.as_payload_ref()
    }

    /// Returns a reference to the user_header of the [`Sample`]
    pub fn user_header(&self) -> &UserHeader {
        self.ptr.as_user_header_ref()
    }

    /// Returns a reference to the [`Header`] of the [`Sample`].
    pub fn header(&self) -> &Header {
        self.ptr.as_header_ref()
    }

    /// Returns the [`UniquePublisherId`] of the [`Publisher`](crate::port::publisher::Publisher)
    pub fn origin(&self) -> UniquePublisherId {
        UniquePublisherId(UniqueSystemId::from(self.details.origin))
    }
}
