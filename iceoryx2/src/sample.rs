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
//! # let node = NodeBuilder::new().create::<zero_copy::Service>()?;
//! # let service = node.service_builder("My/Funk/ServiceName".try_into()?)
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

use std::sync::Arc;
use std::{fmt::Debug, ops::Deref};

use iceoryx2_bb_log::{fatal_panic, warn};
use iceoryx2_cal::zero_copy_connection::{PointerOffset, ZeroCopyReceiver, ZeroCopyReleaseError};

use crate::port::details::publisher_connections::PublisherConnections;
use crate::port::port_identifiers::UniquePublisherId;
use crate::raw_sample::RawSample;
use crate::service::header::publish_subscribe::Header;

#[derive(Debug)]
pub(crate) struct SampleDetails<Service: crate::service::Service> {
    pub(crate) publisher_connections: Arc<PublisherConnections<Service>>,
    pub(crate) channel_id: usize,
    pub(crate) offset: PointerOffset,
    pub(crate) origin: UniquePublisherId,
}

/// It stores the payload and is acquired by the [`Subscriber`](crate::port::subscriber::Subscriber) whenever
/// it receives new data from a [`Publisher`](crate::port::publisher::Publisher) via
/// [`Subscriber::receive()`](crate::port::subscriber::Subscriber::receive()).
pub struct Sample<PayloadType: Debug + ?Sized, Service: crate::service::Service> {
    pub(crate) ptr: RawSample<Header, PayloadType>,
    pub(crate) details: SampleDetails<Service>,
}

impl<PayloadType: Debug + ?Sized, Service: crate::service::Service> Debug
    for Sample<PayloadType, Service>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Sample<{}, {}> {{ details: {:?} }}",
            core::any::type_name::<PayloadType>(),
            core::any::type_name::<Service>(),
            self.details
        )
    }
}

impl<PayloadType: Debug + ?Sized, Service: crate::service::Service> Deref
    for Sample<PayloadType, Service>
{
    type Target = PayloadType;
    fn deref(&self) -> &Self::Target {
        self.ptr.as_payload_ref()
    }
}

impl<PayloadType: Debug + ?Sized, Service: crate::service::Service> Drop
    for Sample<PayloadType, Service>
{
    fn drop(&mut self) {
        match self
            .details
            .publisher_connections
            .get(self.details.channel_id)
        {
            Some(c) => {
                if c.publisher_id == self.details.origin {
                    match c.receiver.release(self.details.offset) {
                        Ok(()) => (),
                        Err(ZeroCopyReleaseError::RetrieveBufferFull) => {
                            fatal_panic!(from self, "This should never happen! The publishers retrieve channel is full and the sample cannot be returned.");
                        }
                    }
                }
            }
            None => {
                warn!(from self, "Unable to release sample since the connection is broken. The sample will be discarded and has to be reclaimed manually by the publisher.");
            }
        }
    }
}

impl<PayloadType: Debug + ?Sized, Service: crate::service::Service> Sample<PayloadType, Service> {
    /// Returns a reference to the payload of the [`Sample`]
    pub fn payload(&self) -> &PayloadType {
        self.ptr.as_payload_ref()
    }

    /// Returns a reference to the [`Header`] of the [`Sample`].
    pub fn header(&self) -> &Header {
        self.ptr.as_header_ref()
    }

    /// Returns the [`UniquePublisherId`] of the [`Publisher`](crate::port::publisher::Publisher)
    pub fn origin(&self) -> UniquePublisherId {
        self.details.origin
    }
}
