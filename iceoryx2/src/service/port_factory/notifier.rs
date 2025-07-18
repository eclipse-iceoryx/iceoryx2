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
//! use iceoryx2::port::event_id::EventId;
//!
//! # fn main() -> Result<(), Box<dyn core::error::Error>> {
//! let node = NodeBuilder::new().create::<ipc::Service>()?;
//! let event = node.service_builder(&"MyEventName".try_into()?)
//!     .event()
//!     .open_or_create()?;
//!
//! let listener = event.notifier_builder()
//!                     .default_event_id(EventId::new(1234))
//!                     .create()?;
//! # Ok(())
//! # }
//! ```
use core::fmt::Debug;

use crate::port::{event_id::EventId, notifier::Notifier, notifier::NotifierCreateError};
use iceoryx2_bb_log::fail;

use crate::service;

use super::event::PortFactory;

/// Factory to create a new [`Notifier`] port/endpoint for
/// [`MessagingPattern::Event`](crate::service::messaging_pattern::MessagingPattern::Event) based
/// communication.
#[derive(Debug, Clone)]
pub struct PortFactoryNotifier<'factory, Service: service::Service> {
    pub(crate) factory: &'factory PortFactory<Service>,
    default_event_id: EventId,
}

unsafe impl<Service: service::Service> Send for PortFactoryNotifier<'_, Service> {}

impl<'factory, Service: service::Service> PortFactoryNotifier<'factory, Service> {
    pub(crate) fn new(factory: &'factory PortFactory<Service>) -> Self {
        Self {
            factory,
            default_event_id: EventId::default(),
        }
    }

    /// Sets a default [`EventId`] for the [`Notifier`] that is used in
    /// [`Notifier::notify()`]
    pub fn default_event_id(mut self, value: EventId) -> Self {
        self.default_event_id = value;
        self
    }

    /// Creates a new [`Notifier`] port or returns a [`NotifierCreateError`] on failure.
    pub fn create(self) -> Result<Notifier<Service>, NotifierCreateError> {
        Ok(
            fail!(from self, when Notifier::new(self.factory.service.clone(), self.default_event_id),
                    "Failed to create new Notifier port."),
        )
    }
}
