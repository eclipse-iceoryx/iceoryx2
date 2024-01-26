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
//! #   .open_or_create::<u64>()?;
//! # let subscriber = service.subscriber().create()?;
//!
//! while let Some(sample) = subscriber.receive()? {
//!     println!("received: {:?}", *sample);
//!     println!("header timestamp {:?}, publisher id {:?}",
//!         sample.header().time_stamp(), sample.header().publisher_id());
//! }
//!
//! # Ok(())
//! # }
//! ```
//!
//! See also [`crate::sample::Sample`].

use std::{fmt::Debug, ops::Deref};

use crate::port::subscribe::internal::SubscribeMgmt;
use crate::service::header::publish_subscribe::Header;
use crate::{payload::Payload, raw_sample::RawSample};

/// It stores the payload and is acquired by the [`crate::port::subscriber::Subscriber`] whenever
/// it receives new data from a [`crate::port::publisher::Publisher`] via
/// [`crate::port::subscribe::Subscribe::receive()`].
#[derive(Debug)]
pub struct Sample<'subscriber, MessageType: Debug> {
    pub(crate) subscriber: &'subscriber dyn SubscribeMgmt,
    pub(crate) ptr: RawSample<Header, MessageType>,
    pub(crate) channel_id: usize,
}

impl<MessageType: Debug> Deref for Sample<'_, MessageType> {
    type Target = MessageType;
    fn deref(&self) -> &Self::Target {
        self.ptr.as_data_ref()
    }
}

impl<MessageType: Debug> Drop for Sample<'_, MessageType> {
    fn drop(&mut self) {
        self.subscriber
            .release_sample(self.channel_id, self.ptr.as_ptr() as usize);
    }
}

impl<'subscriber, MessageType: Debug> Payload<MessageType> for Sample<'subscriber, MessageType> {
    fn payload(&self) -> &MessageType {
        self.ptr.as_data_ref()
    }

    fn header(&self) -> &Header {
        self.ptr.as_header_ref()
    }
}
