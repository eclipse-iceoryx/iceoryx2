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
use std::alloc::Layout;

use crate::port::port_identifiers::UniquePublisherId;

/// Sample header used by
/// [`MessagingPattern::PublishSubscribe`](crate::service::messaging_pattern::MessagingPattern::PublishSubscribe)
#[derive(Debug)]
#[repr(C)]
pub struct Header {
    publisher_port_id: UniquePublisherId,
    payload_type_layout: Layout,
}

impl Header {
    pub(crate) fn new(publisher_port_id: UniquePublisherId, payload_type_layout: Layout) -> Self {
        Self {
            publisher_port_id,
            payload_type_layout,
        }
    }

    /// Returns the [`UniquePublisherId`] of the source [`crate::port::publisher::Publisher`].
    pub fn publisher_id(&self) -> UniquePublisherId {
        self.publisher_port_id
    }

    /// Returns the [`Layout`] of the corresponding payload.
    pub fn payload_type_layout(&self) -> Layout {
        self.payload_type_layout
    }
}
