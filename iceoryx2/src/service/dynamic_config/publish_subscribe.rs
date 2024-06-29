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
//! let node = NodeBuilder::new().create::<zero_copy::Service>()?;
//! let pubsub = node.service_builder("My/Funk/ServiceName".try_into()?)
//!     .publish_subscribe::<u64>()
//!     .open_or_create()?;
//!
//! println!("number of active publishers:      {:?}", pubsub.dynamic_config().number_of_publishers());
//! println!("number of active subscribers:     {:?}", pubsub.dynamic_config().number_of_subscribers());
//! # Ok(())
//! # }
//! ```
use iceoryx2_bb_elementary::relocatable_container::RelocatableContainer;
use iceoryx2_bb_lock_free::mpmc::{container::*, unique_index_set::ReleaseMode};
use iceoryx2_bb_log::fatal_panic;
use iceoryx2_bb_memory::bump_allocator::BumpAllocator;

use crate::port::port_identifiers::{UniquePublisherId, UniqueSubscriberId};

#[derive(Debug, Clone, Copy)]
pub(crate) struct DynamicConfigSettings {
    pub number_of_subscribers: usize,
    pub number_of_publishers: usize,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct PublisherDetails {
    pub(crate) publisher_id: UniquePublisherId,
    pub(crate) number_of_samples: usize,
    pub(crate) max_slice_len: usize,
}

#[derive(Debug, Copy, Clone)]
pub(crate) struct SubscriberDetails {
    pub(crate) port_id: UniqueSubscriberId,
    pub(crate) buffer_size: usize,
}

/// The dynamic configuration of an [`crate::service::messaging_pattern::MessagingPattern::Event`]
/// based service. Contains dynamic parameters like the connected endpoints etc..
#[derive(Debug)]
pub struct DynamicConfig {
    pub(crate) subscribers: Container<SubscriberDetails>,
    pub(crate) publishers: Container<PublisherDetails>,
}

impl DynamicConfig {
    pub(crate) fn new(config: &DynamicConfigSettings) -> Self {
        Self {
            subscribers: unsafe { Container::new_uninit(config.number_of_subscribers) },
            publishers: unsafe { Container::new_uninit(config.number_of_publishers) },
        }
    }

    pub(crate) unsafe fn init(&self, allocator: &BumpAllocator) {
        fatal_panic!(from self,
            when self.subscribers.init(allocator),
            "This should never happen! Unable to initialize subscriber port id container.");
        fatal_panic!(from self,
            when self.publishers.init(allocator),
            "This should never happen! Unable to initialize publisher port id container.");
    }

    pub(crate) fn memory_size(config: &DynamicConfigSettings) -> usize {
        Container::<SubscriberDetails>::memory_size(config.number_of_subscribers)
            + Container::<PublisherDetails>::memory_size(config.number_of_publishers)
    }

    /// Returns how many [`crate::port::publisher::Publisher`] ports are currently connected.
    pub fn number_of_publishers(&self) -> usize {
        self.publishers.len()
    }

    /// Returns how many [`crate::port::subscriber::Subscriber`] ports are currently connected.
    pub fn number_of_subscribers(&self) -> usize {
        self.subscribers.len()
    }

    pub(crate) fn add_subscriber_id(&self, details: SubscriberDetails) -> Option<ContainerHandle> {
        unsafe { self.subscribers.add(details).ok() }
    }

    pub(crate) fn release_subscriber_handle(&self, handle: ContainerHandle) {
        unsafe { self.subscribers.remove(handle, ReleaseMode::Default) };
    }

    pub(crate) fn add_publisher_id(&self, details: PublisherDetails) -> Option<ContainerHandle> {
        unsafe { self.publishers.add(details).ok() }
    }

    pub(crate) fn release_publisher_handle(&self, handle: ContainerHandle) {
        unsafe { self.publishers.remove(handle, ReleaseMode::Default) };
    }
}
