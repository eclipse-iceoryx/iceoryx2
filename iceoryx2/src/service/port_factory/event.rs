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
//! println!("name:                         {:?}", event.name());
//! println!("uuid:                         {:?}", event.uuid());
//! println!("max listeners:                {:?}", event.static_config().max_supported_listeners());
//! println!("max notifiers:                {:?}", event.static_config().max_supported_notifiers());
//! println!("number of active listeners:   {:?}", event.dynamic_config().number_of_listeners());
//! println!("number of active notifiers:   {:?}", event.dynamic_config().number_of_notifiers());
//!
//! let listener = event.listener().create()?;
//! let notifier = event.notifier().create()?;
//! # Ok(())
//! # }
//! ```
use iceoryx2_cal::dynamic_storage::DynamicStorage;

use crate::service::{self, static_config};
use crate::service::{dynamic_config, ServiceName};
use std::marker::PhantomData;

use super::listener::PortFactoryListener;
use super::notifier::PortFactoryNotifier;

/// The factory for
/// [`MessagingPattern::Event`](crate::service::messaging_pattern::MessagingPattern::Event). It can
/// acquire dynamic and static service informations and create [`crate::port::notifier::Notifier`]
/// or [`crate::port::listener::Listener`] ports.
#[derive(Debug)]
pub struct PortFactory<'config, Service: service::Details<'config>> {
    pub(crate) service: Service,
    _phantom_lifetime_b: PhantomData<&'config ()>,
}

unsafe impl<'config, Service: service::Details<'config>> Send for PortFactory<'config, Service> {}
unsafe impl<'config, Service: service::Details<'config>> Sync for PortFactory<'config, Service> {}

impl<'config, Service: service::Details<'config>> PortFactory<'config, Service> {
    pub(crate) fn new(service: Service) -> Self {
        Self {
            service,
            _phantom_lifetime_b: PhantomData,
        }
    }

    /// Returns the [`ServiceName`] of the [`crate::service::Service`]
    pub fn name(&self) -> &ServiceName {
        self.service.state().static_config.service_name()
    }

    /// Returns the uuid of the [`crate::service::Service`]
    pub fn uuid(&self) -> &str {
        self.service.state().static_config.uuid()
    }

    /// Returns the [`static_config::event::StaticConfig`] of the [`crate::service::Service`].
    /// Contains all settings that never change during the lifetime of the service.
    pub fn static_config(&self) -> &static_config::event::StaticConfig {
        self.service.state().static_config.event()
    }

    /// Returns the [`dynamic_config::event::DynamicConfig`] of the [`crate::service::Service`].
    /// Contains all dynamic settings, like the current participants etc..
    pub fn dynamic_config(&self) -> &dynamic_config::event::DynamicConfig {
        self.service.state().dynamic_storage.get().event()
    }

    /// Returns a [`PortFactoryNotifier`] to create a new [`crate::port::notifier::Notifier`] port
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let event_name = ServiceName::new("MyEventName")?;
    /// let event = zero_copy::Service::new(&event_name)
    ///     .event()
    ///     .open_or_create()?;
    ///
    /// let notifier = event.notifier().create()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn notifier<'a>(&'a self) -> PortFactoryNotifier<'a, 'config, Service> {
        PortFactoryNotifier::new(self)
    }

    /// Returns a [`PortFactoryListener`] to create a new [`crate::port::listener::Listener`] port
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let event_name = ServiceName::new("MyEventName")?;
    /// let event = zero_copy::Service::new(&event_name)
    ///     .event()
    ///     .open_or_create()?;
    ///
    /// let listener = event.listener().create()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn listener<'a>(&'a self) -> PortFactoryListener<'a, 'config, Service> {
        PortFactoryListener { factory: self }
    }
}
