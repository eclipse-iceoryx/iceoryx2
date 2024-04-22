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
//! # let service_name = ServiceName::new("My/Funk/ServiceName")?;
//! # let service = zero_copy::Service::new(&service_name)
//! #   .publish_subscribe()
//! #   .typed::<u64>()
//! #   .open_or_create()?;
//! # let subscriber = service.subscriber().create()?;
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

/// It stores the payload and is acquired by the [`Subscriber`](crate::port::subscriber::Subscriber) whenever
/// it receives new data from a [`Publisher`](crate::port::publisher::Publisher) via
/// [`Subscriber::receive()`](crate::port::subscriber::Subscriber::receive()).
#[derive(Debug)]
pub struct Sample<MessageType: Debug, Service: crate::service::Service> {
    pub(crate) publisher_connections: Arc<PublisherConnections<Service>>,
    pub(crate) ptr: RawSample<Header, MessageType>,
    pub(crate) channel_id: usize,
    pub(crate) offset: PointerOffset,
    pub(crate) origin: UniquePublisherId,
}

impl<MessageType: Debug, Service: crate::service::Service> Deref for Sample<MessageType, Service> {
    type Target = MessageType;
    fn deref(&self) -> &Self::Target {
        self.ptr.as_data_ref()
    }
}

impl<MessageType: Debug, Service: crate::service::Service> Drop for Sample<MessageType, Service> {
    fn drop(&mut self) {
        match self.publisher_connections.get(self.channel_id) {
            Some(c) => {
                if c.publisher_id == self.origin {
                    match c.receiver.release(self.offset) {
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

impl<MessageType: Debug, Service: crate::service::Service> Sample<MessageType, Service> {
    /// Returns a reference to the payload of the [`Sample`]
    pub fn payload(&self) -> &MessageType {
        self.ptr.as_data_ref()
    }

    /// Returns a reference to the [`Header`] of the [`Sample`].
    pub fn header(&self) -> &Header {
        self.ptr.as_header_ref()
    }

    /// Returns the [`UniquePublisherId`] of the [`Publisher`](crate::port::publisher::Publisher)
    pub fn origin(&self) -> UniquePublisherId {
        self.origin
    }
}
