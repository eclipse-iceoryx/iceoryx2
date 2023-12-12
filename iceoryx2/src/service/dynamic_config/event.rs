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

//! # Examples
//!
//! ```
//! use iceoryx2::prelude::*;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let event_name = ServiceName::new("MyEventName")?;
//! let event = zero_copy::Service::new(&event_name)
//!     .event()
//!     .open_or_create()?;
//!
//! println!("number of active listeners:   {:?}", event.dynamic_config().number_of_listeners());
//! println!("number of active notifiers:   {:?}", event.dynamic_config().number_of_notifiers());
//! # Ok(())
//! # }
//! ```
use iceoryx2_bb_elementary::relocatable_container::RelocatableContainer;
use iceoryx2_bb_lock_free::mpmc::{container::*, unique_index_set::UniqueIndex};
use iceoryx2_bb_log::fatal_panic;
use iceoryx2_bb_memory::bump_allocator::BumpAllocator;

use crate::port::port_identifiers::{UniqueListenerId, UniqueNotifierId};

#[derive(Debug, Clone, Copy)]
pub(crate) struct DynamicConfigSettings {
    pub number_of_listeners: usize,
    pub number_of_notifiers: usize,
}

/// The dynamic configuration of an [`crate::service::messaging_pattern::MessagingPattern::Event`]
/// based service. Contains dynamic parameters like the connected endpoints etc..
#[derive(Debug)]
pub struct DynamicConfig {
    pub(crate) listeners: Container<UniqueListenerId>,
    pub(crate) notifiers: Container<UniqueNotifierId>,
}

impl DynamicConfig {
    pub(crate) fn new(config: &DynamicConfigSettings) -> Self {
        Self {
            listeners: unsafe { Container::new_uninit(config.number_of_listeners) },
            notifiers: unsafe { Container::new_uninit(config.number_of_notifiers) },
        }
    }

    pub(crate) unsafe fn init(&self, allocator: &BumpAllocator) {
        fatal_panic!(from "event::DynamicConfig::init",
            when self.listeners.init(allocator),
            "This should never happen! Unable to initialize listener port id container.");
        fatal_panic!(from "event::DynamicConfig::init",
            when self.notifiers.init(allocator),
            "This should never happen! Unable to initialize notifier port id container.");
    }

    pub(crate) fn memory_size(config: &DynamicConfigSettings) -> usize {
        Container::<UniqueListenerId>::memory_size(config.number_of_listeners)
            + Container::<UniqueNotifierId>::memory_size(config.number_of_notifiers)
    }

    /// Returns the how many [`crate::port::listener::Listener`] ports are currently connected.
    pub fn number_of_listeners(&self) -> usize {
        self.listeners.len()
    }

    /// Returns the how many [`crate::port::notifier::Notifier`] ports are currently connected.
    pub fn number_of_notifiers(&self) -> usize {
        self.notifiers.len()
    }

    pub(crate) fn add_listener_id(&self, id: UniqueListenerId) -> Option<UniqueIndex> {
        unsafe { self.listeners.add(id) }
    }

    pub(crate) fn add_notifier_id(&self, id: UniqueNotifierId) -> Option<UniqueIndex> {
        unsafe { self.notifiers.add(id) }
    }
}
