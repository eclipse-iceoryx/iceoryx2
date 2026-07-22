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
//! # fn main() -> Result<(), Box<dyn core::error::Error>> {
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

use iceoryx2_bb_derive_macros::ZeroCopySend;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;

use crate::identifiers::{UniqueNodeId, UniquePublisherId};

/// Sample header used by
/// [`MessagingPattern::PublishSubscribe`](crate::service::messaging_pattern::MessagingPattern::PublishSubscribe)
#[derive(Debug, Copy, Clone, ZeroCopySend, PartialEq, Eq)]
#[repr(C)]
pub struct Header {
    node_id: UniqueNodeId,
    publisher_port_id: UniquePublisherId,
    pub(crate) metadata: u64,
}

impl Header {
    pub(crate) fn new(
        node_id: UniqueNodeId,
        publisher_port_id: UniquePublisherId,
        metadata: u64,
    ) -> Self {
        Self {
            node_id,
            publisher_port_id,
            metadata,
        }
    }

    /// Returns the [`UniqueNodeId`] of the source node that published the
    /// [`Sample`](crate::sample::Sample).
    pub fn node_id(&self) -> UniqueNodeId {
        self.node_id
    }

    /// Returns the [`UniquePublisherId`] of the source
    /// [`Publisher`](crate::port::publisher::Publisher).
    pub fn publisher_id(&self) -> UniquePublisherId {
        self.publisher_port_id
    }

    pub(crate) fn metadata(&self) -> u64 {
        self.metadata
    }
}
