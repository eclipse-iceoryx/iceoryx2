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
//! ```
//! use iceoryx2::prelude::*;
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # let service_name = ServiceName::new("My/Funk/ServiceName")?;
//! # let service = zero_copy::Service::new(&service_name)
//! #   .publish_subscribe()
//! #   .open_or_create::<u64>()?;
//! # let subscriber = service.subscriber().create()?;
//!
//! let mut received_samples: Vec<Box<dyn Sample<u64>>> = vec![];
//!
//! while let Some(sample) = subscriber.receive()? {
//!     println!("received: {:?}", *sample);
//!     println!("header timestamp {:?}, publisher id {:?}",
//!         sample.header().time_stamp(), sample.header().publisher_id());
//!     received_samples.push(Box::new(sample));
//! }
//!
//! # Ok(())
//! # }
//! ```
//!
//! See also [`crate::sample_impl::SampleImpl`].

use crate::service::header::publish_subscribe::Header;

/// It stores the payload and is acquired by the [`crate::port::subscriber::Subscriber`] whenever
/// it receives new data from a [`crate::port::publisher::Publisher`] via
/// [`crate::port::subscriber::Subscriber::receive()`].
pub trait Payload<MessageType> {
    /// Returns a reference to the payload of the sample
    fn payload(&self) -> &MessageType;

    /// Returns a reference to the header of the sample.
    fn header(&self) -> &Header;
}
