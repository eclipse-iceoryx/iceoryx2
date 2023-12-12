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
//! println!("max listeners:                {:?}", event.static_config().max_supported_listeners());
//! println!("max notifiers:                {:?}", event.static_config().max_supported_notifiers());
//!
//! # Ok(())
//! # }
//! ```
use crate::config;
use serde::{Deserialize, Serialize};

/// The static configuration of an [`crate::service::messaging_pattern::MessagingPattern::Event`]
/// based service. Contains all parameters that do not change during the lifetime of a
/// [`crate::service::Service`].
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct StaticConfig {
    pub(crate) max_notifiers: usize,
    pub(crate) max_listeners: usize,
}

impl StaticConfig {
    pub(crate) fn new(config: &config::Config) -> Self {
        Self {
            max_notifiers: config.defaults.event.max_notifiers,
            max_listeners: config.defaults.event.max_listeners,
        }
    }

    /// Returns the maximum supported amount of [`crate::port::notifier::Notifier`] ports
    pub fn max_supported_notifiers(&self) -> usize {
        self.max_notifiers
    }

    /// Returns the maximum supported amount of [`crate::port::listener::Listener`] ports
    pub fn max_supported_listeners(&self) -> usize {
        self.max_listeners
    }
}
