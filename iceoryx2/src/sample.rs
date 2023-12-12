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

use std::{fmt::Debug, ops::Deref};

use crate::{port::subscriber::Subscriber, raw_sample::RawSample, service};

/// It stores the payload and is acquired by the [`Subscriber`] whenever it receives new data from a
/// [`crate::port::publisher::Publisher`] via [`Subscriber::receive()`].
#[derive(Debug)]
pub struct Sample<
    'a,
    'subscriber,
    'config,
    Service: service::Details<'config>,
    Header: Debug,
    MessageType: Debug,
> {
    pub(crate) subscriber: &'subscriber Subscriber<'a, 'config, Service, MessageType>,
    pub(crate) ptr: RawSample<Header, MessageType>,
    pub(crate) channel_id: usize,
}

impl<'config, Service: service::Details<'config>, Header: Debug, MessageType: Debug> Deref
    for Sample<'_, '_, 'config, Service, Header, MessageType>
{
    type Target = MessageType;
    fn deref(&self) -> &Self::Target {
        self.ptr.as_data_ref()
    }
}

impl<
        'a,
        'subscriber,
        'config,
        Service: service::Details<'config>,
        Header: Debug,
        MessageType: Debug,
    > Drop for Sample<'a, 'subscriber, 'config, Service, Header, MessageType>
{
    fn drop(&mut self) {
        self.subscriber.release_sample(self.channel_id, self.ptr);
    }
}

impl<
        'a,
        'subscriber,
        'config,
        Service: service::Details<'config>,
        Header: Debug,
        MessageType: Debug,
    > Sample<'a, 'subscriber, 'config, Service, Header, MessageType>
{
    /// Returns a reference to the payload of the sample
    pub fn payload(&self) -> &MessageType {
        self.ptr.as_data_ref()
    }

    /// Returns a reference to the header of the sample. In publish subscribe communication the
    /// default header is [`crate::service::header::publish_subscribe::Header`].
    pub fn header(&self) -> &Header {
        self.ptr.as_header_ref()
    }
}
