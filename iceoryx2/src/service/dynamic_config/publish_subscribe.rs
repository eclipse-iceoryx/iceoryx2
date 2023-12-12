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
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let service_name = ServiceName::new("My/Funk/ServiceName")?;
//! let pubsub = zero_copy::Service::new(&service_name)
//!     .publish_subscribe()
//!     .open_or_create::<u64>()?;
//!
//! println!("number of active publishers:      {:?}", pubsub.dynamic_config().number_of_publishers());
//! println!("number of active subscribers:     {:?}", pubsub.dynamic_config().number_of_subscribers());
//! # Ok(())
//! # }
//! ```
use iceoryx2_bb_elementary::relocatable_container::RelocatableContainer;
use iceoryx2_bb_lock_free::mpmc::{container::*, unique_index_set::UniqueIndex};
use iceoryx2_bb_log::fatal_panic;
use iceoryx2_bb_memory::bump_allocator::BumpAllocator;

use crate::port::port_identifiers::{UniquePublisherId, UniqueSubscriberId};

#[derive(Debug, Clone, Copy)]
pub(crate) struct DynamicConfigSettings {
    pub number_of_subscribers: usize,
    pub number_of_publishers: usize,
}

/// The dynamic configuration of an [`crate::service::messaging_pattern::MessagingPattern::Event`]
/// based service. Contains dynamic parameters like the connected endpoints etc..
#[derive(Debug)]
pub struct DynamicConfig {
    pub(crate) subscribers: Container<UniqueSubscriberId>,
    pub(crate) publishers: Container<UniquePublisherId>,
}

impl DynamicConfig {
    pub(crate) fn new(config: &DynamicConfigSettings) -> Self {
        Self {
            subscribers: unsafe { Container::new_uninit(config.number_of_subscribers) },
            publishers: unsafe { Container::new_uninit(config.number_of_publishers) },
        }
    }

    pub(crate) unsafe fn init(&self, allocator: &BumpAllocator) {
        fatal_panic!(from "publish_subscribe::DynamicConfig::init",
            when self.subscribers.init(allocator),
            "This should never happen! Unable to initialize subscriber port id container.");
        fatal_panic!(from "publish_subscribe::DynamicConfig::init",
            when self.publishers.init(allocator),
            "This should never happen! Unable to initialize publisher port id container.");
    }

    pub(crate) fn memory_size(config: &DynamicConfigSettings) -> usize {
        Container::<UniqueSubscriberId>::memory_size(config.number_of_subscribers)
            + Container::<UniquePublisherId>::memory_size(config.number_of_publishers)
    }

    /// Returns how many [`crate::port::publisher::Publisher`] ports are currently connected.
    pub fn number_of_publishers(&self) -> usize {
        self.publishers.len()
    }

    /// Returns how many [`crate::port::subscriber::Subscriber`] ports are currently connected.
    pub fn number_of_subscribers(&self) -> usize {
        self.subscribers.len()
    }

    pub(crate) fn add_subscriber_id(&self, id: UniqueSubscriberId) -> Option<UniqueIndex> {
        unsafe { self.subscribers.add(id) }
    }

    pub(crate) fn add_publisher_id(&self, id: UniquePublisherId) -> Option<UniqueIndex> {
        unsafe { self.publishers.add(id) }
    }
}
