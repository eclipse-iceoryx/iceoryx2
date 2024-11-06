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
//! # let node = NodeBuilder::new().create::<ipc::Service>()?;
//! let service = node.service_builder(&"My/Funk/ServiceName".try_into()?)
//!     .publish_subscribe::<u64>()
//!     .open_or_create()?;
//!
//! let subscriber = service.subscriber_builder().create()?;
//!
//! while let Some(sample) = subscriber.receive()? {
//!     println!("header: {:?}", sample.header());
//! }
//! # Ok(())
//! # }
//! ```

use crate::port::port_identifiers::UniquePublisherId;

/// Sample header used by
/// [`MessagingPattern::PublishSubscribe`](crate::service::messaging_pattern::MessagingPattern::PublishSubscribe)
#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct Header {
    publisher_port_id: UniquePublisherId,
    number_of_elements: u64,
}

impl Header {
    pub(crate) fn new(publisher_port_id: UniquePublisherId, number_of_elements: u64) -> Self {
        Self {
            publisher_port_id,
            number_of_elements,
        }
    }

    /// Returns the [`UniquePublisherId`] of the source [`crate::port::publisher::Publisher`].
    pub fn publisher_id(&self) -> UniquePublisherId {
        self.publisher_port_id
    }

    /// Returns how many elements are stored inside the sample's payload.
    ///
    /// # Details when using
    /// [`CustomPayloadMarker`](crate::service::builder::publish_subscribe::CustomPayloadMarker)
    ///
    /// In this case the number of elements relates to the element defined in the
    /// [`MessageTypeDetails`](crate::service::static_config::message_type_details::MessageTypeDetails).
    /// When the element has a `payload.size == 40` and the `Sample::payload().len() == 120` it
    /// means that it contains 3 elements (3 * 40 == 120).
    pub fn number_of_elements(&self) -> u64 {
        self.number_of_elements
    }
}
