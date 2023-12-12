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
//! use iceoryx2::service::header::publish_subscribe::Header;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # let service_name = ServiceName::new("My/Funk/ServiceName")?;
//! let service = zero_copy::Service::new(&service_name)
//!     .publish_subscribe()
//!     .open_or_create::<u64>()?;
//!
//! let subscriber = service.subscriber().create()?;
//!
//! while let Some(sample) = subscriber.receive()? {
//!     println!("header: {:?}", sample.header());
//! }
//! # Ok(())
//! # }
//! ```
use iceoryx2_bb_posix::clock::{Time, TimeBuilder};

use crate::port::port_identifiers::UniquePublisherId;

#[derive(Debug)]
#[repr(C)]
struct TimeStamp {
    seconds: u64,
    nanoseconds: u32,
}

/// Message header used by
/// [`MessagingPattern::PublishSubscribe`](crate::service::messaging_pattern::MessagingPattern::PublishSubscribe)
#[derive(Debug)]
#[repr(C)]
pub struct Header {
    publisher_port_id: UniquePublisherId,
    time_stamp: TimeStamp,
}

impl Header {
    pub(crate) fn new(publisher_port_id: UniquePublisherId) -> Self {
        let now = Time::now().unwrap();
        Self {
            publisher_port_id,
            time_stamp: TimeStamp {
                seconds: now.seconds(),
                nanoseconds: now.nanoseconds(),
            },
        }
    }

    /// Returns the [`UniquePublisherId`] of the source [`crate::port::publisher::Publisher`].
    pub fn publisher_id(&self) -> UniquePublisherId {
        self.publisher_port_id
    }

    /// Returns the [`Time`] when the [`crate::sample::Sample`] was delivered.
    pub fn time_stamp(&self) -> Time {
        TimeBuilder::new()
            .nanoseconds(self.time_stamp.nanoseconds)
            .seconds(self.time_stamp.seconds)
            .create()
    }
}
