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
//! println!("name:                         {:?}", event.name());
//! println!("service id:                   {:?}", event.service_id());
//! println!("deadline:                     {:?}", event.static_config().deadline());
//! println!("max listeners:                {:?}", event.static_config().max_listeners());
//! println!("max notifiers:                {:?}", event.static_config().max_notifiers());
//! println!("number of active listeners:   {:?}", event.dynamic_config().number_of_listeners());
//! println!("number of active notifiers:   {:?}", event.dynamic_config().number_of_notifiers());
//!
//! let listener = event.listener_builder().create()?;
//! let notifier = event.notifier_builder().create()?;
//! # Ok(())
//! # }
//! ```
use iceoryx2_bb_elementary::CallbackProgression;
use iceoryx2_cal::dynamic_storage::DynamicStorage;

use crate::node::NodeListFailure;
use crate::service::attribute::AttributeSet;
use crate::service::service_id::ServiceId;
use crate::service::{self, static_config, NoResource, ServiceState};
use crate::service::{dynamic_config, ServiceName};

use super::listener::PortFactoryListener;
use super::nodes;
use super::notifier::PortFactoryNotifier;

extern crate alloc;
use alloc::sync::Arc;

/// The factory for
/// [`MessagingPattern::Event`](crate::service::messaging_pattern::MessagingPattern::Event). It can
/// acquire dynamic and static service informations and create [`crate::port::notifier::Notifier`]
/// or [`crate::port::listener::Listener`] ports.
#[derive(Debug)]
pub struct PortFactory<Service: service::Service> {
    pub(crate) service: Arc<ServiceState<Service, NoResource>>,
}

unsafe impl<Service: service::Service> Send for PortFactory<Service> {}
unsafe impl<Service: service::Service> Sync for PortFactory<Service> {}

impl<Service: service::Service> crate::service::port_factory::PortFactory for PortFactory<Service> {
    type Service = Service;
    type StaticConfig = static_config::event::StaticConfig;
    type DynamicConfig = dynamic_config::event::DynamicConfig;

    fn name(&self) -> &ServiceName {
        self.service.static_config.name()
    }

    fn service_id(&self) -> &ServiceId {
        self.service.static_config.service_id()
    }

    fn attributes(&self) -> &AttributeSet {
        self.service.static_config.attributes()
    }

    fn static_config(&self) -> &static_config::event::StaticConfig {
        self.service.static_config.event()
    }

    fn dynamic_config(&self) -> &dynamic_config::event::DynamicConfig {
        self.service.dynamic_storage.get().event()
    }

    fn nodes<F: FnMut(crate::node::NodeState<Service>) -> CallbackProgression>(
        &self,
        callback: F,
    ) -> Result<(), NodeListFailure> {
        nodes(
            self.service.dynamic_storage.get(),
            self.service.shared_node.config(),
            callback,
        )
    }
}

impl<Service: service::Service> PortFactory<Service> {
    pub(crate) fn new(service: ServiceState<Service, NoResource>) -> Self {
        Self {
            service: Arc::new(service),
        }
    }

    /// Returns a [`PortFactoryNotifier`] to create a new [`crate::port::notifier::Notifier`] port
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    ///
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// let event = node.service_builder(&"MyEventName".try_into()?)
    ///     .event()
    ///     .open_or_create()?;
    ///
    /// let notifier = event.notifier_builder().create()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn notifier_builder(&self) -> PortFactoryNotifier<'_, Service> {
        PortFactoryNotifier::new(self)
    }

    /// Returns a [`PortFactoryListener`] to create a new [`crate::port::listener::Listener`] port
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2::prelude::*;
    ///
    /// # fn main() -> Result<(), Box<dyn core::error::Error>> {
    /// let node = NodeBuilder::new().create::<ipc::Service>()?;
    /// let event = node.service_builder(&"MyEventName".try_into()?)
    ///     .event()
    ///     .open_or_create()?;
    ///
    /// let listener = event.listener_builder().create()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn listener_builder(&self) -> PortFactoryListener<'_, Service> {
        PortFactoryListener { factory: self }
    }
}
