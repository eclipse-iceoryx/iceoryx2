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
//! # fn main() -> Result<(), Box<dyn core::error::Error>> {
//! let node = NodeBuilder::new().create::<ipc::Service>()?;
//! let event = node.service_builder(&"MyEventName".try_into()?)
//!     .event()
//!     .open_or_create()?;
//!
//! println!("max listeners:                {:?}", event.static_config().max_listeners());
//! println!("max notifiers:                {:?}", event.static_config().max_notifiers());
//! println!("event id max value:           {:?}", event.static_config().event_id_max_value());
//! println!("deadline:                     {:?}", event.static_config().deadline());
//! println!("notifier created event:       {:?}", event.static_config().notifier_created_event());
//! println!("notifier dropped event:       {:?}", event.static_config().notifier_dropped_event());
//! println!("notifier dead event:          {:?}", event.static_config().notifier_dead_event());
//!
//! # Ok(())
//! # }
//! ```
use core::time::Duration;

use crate::{config, prelude::EventId};
use iceoryx2_bb_derive_macros::ZeroCopySend;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use iceoryx2_bb_posix::clock::Time;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, ZeroCopySend, Serialize, Deserialize)]
#[repr(C)]
pub(crate) struct Deadline {
    pub(crate) creation_time: Time,
    pub(crate) value: Duration,
}

/// The static configuration of an [`MessagingPattern::Event`](crate::service::messaging_pattern::MessagingPattern::Event)
/// based service. Contains all parameters that do not change during the lifetime of a
/// [`Service`](crate::service::Service).
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, ZeroCopySend, Serialize, Deserialize)]
#[repr(C)]
pub struct StaticConfig {
    pub(crate) max_notifiers: usize,
    pub(crate) max_listeners: usize,
    pub(crate) max_nodes: usize,
    pub(crate) event_id_max_value: usize,
    pub(crate) deadline: Option<Deadline>,
    pub(crate) notifier_created_event: Option<usize>,
    pub(crate) notifier_dropped_event: Option<usize>,
    pub(crate) notifier_dead_event: Option<usize>,
}

impl StaticConfig {
    pub(crate) fn new(config: &config::Config) -> Self {
        Self {
            max_notifiers: config.defaults.event.max_notifiers,
            max_listeners: config.defaults.event.max_listeners,
            max_nodes: config.defaults.event.max_nodes,
            deadline: config.defaults.event.deadline.map(|v| Deadline {
                creation_time: Time::default(),
                value: v,
            }),
            event_id_max_value: config.defaults.event.event_id_max_value,
            notifier_created_event: config.defaults.event.notifier_created_event,
            notifier_dropped_event: config.defaults.event.notifier_dropped_event,
            notifier_dead_event: config.defaults.event.notifier_dead_event,
        }
    }

    /// Returns the deadline of the service. If no new notification is signaled from any
    /// [`Notifier`](crate::port::notifier::Notifier) after the given deadline, it is rated
    /// as an error and all [`Listener`](crate::port::listener::Listener) that are attached
    /// to a [`WaitSet`](crate::waitset::WaitSet) are woken up and notified about the missed
    /// deadline.
    pub fn deadline(&self) -> Option<Duration> {
        self.deadline.map(|v| v.value)
    }

    /// Returns the maximum supported amount of [`Node`](crate::node::Node)s that can open the
    /// [`Service`](crate::service::Service) in parallel.
    pub fn max_nodes(&self) -> usize {
        self.max_nodes
    }

    /// Returns the maximum supported amount of [`crate::port::notifier::Notifier`] ports
    pub fn max_notifiers(&self) -> usize {
        self.max_notifiers
    }

    /// Returns the maximum supported amount of [`crate::port::listener::Listener`] ports
    pub fn max_listeners(&self) -> usize {
        self.max_listeners
    }

    /// Returns the largest event_id that is supported by the service
    pub fn event_id_max_value(&self) -> usize {
        self.event_id_max_value
    }

    /// Returns the emitted [`EventId`] when a new notifier is created.
    pub fn notifier_created_event(&self) -> Option<EventId> {
        self.notifier_created_event.map(EventId::new)
    }

    /// Returns the emitted [`EventId`] when a notifier is dropped.
    pub fn notifier_dropped_event(&self) -> Option<EventId> {
        self.notifier_dropped_event.map(EventId::new)
    }

    /// Returns the emitted [`EventId`] when a notifier is identified as dead.
    pub fn notifier_dead_event(&self) -> Option<EventId> {
        self.notifier_dead_event.map(EventId::new)
    }
}
