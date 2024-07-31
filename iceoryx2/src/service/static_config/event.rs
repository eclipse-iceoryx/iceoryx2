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
//! let node = NodeBuilder::new().create::<ipc::Service>()?;
//! let event = node.service_builder(&"MyEventName".try_into()?)
//!     .event()
//!     .open_or_create()?;
//!
//! println!("max listeners:                {:?}", event.static_config().max_listeners());
//! println!("max notifiers:                {:?}", event.static_config().max_notifiers());
//! println!("event id max value:           {:?}", event.static_config().event_id_max_value());
//!
//! # Ok(())
//! # }
//! ```
use crate::config;
use serde::{Deserialize, Serialize};

/// The static configuration of an [`MessagingPattern::Event`](crate::service::messaging_pattern::MessagingPattern::Event)
/// based service. Contains all parameters that do not change during the lifetime of a
/// [`Service`](crate::service::Service).
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct StaticConfig {
    pub(crate) max_notifiers: usize,
    pub(crate) max_listeners: usize,
    pub(crate) max_nodes: usize,
    pub(crate) event_id_max_value: usize,
}

impl StaticConfig {
    pub(crate) fn new(config: &config::Config) -> Self {
        Self {
            max_notifiers: config.defaults.event.max_notifiers,
            max_listeners: config.defaults.event.max_listeners,
            max_nodes: config.defaults.event.max_nodes,
            event_id_max_value: config.defaults.event.event_id_max_value,
        }
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
}
