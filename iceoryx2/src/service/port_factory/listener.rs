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
//! let listener = event.listener_builder().create()?;
//! # Ok(())
//! # }
//! ```
use core::fmt::Debug;

use iceoryx2_bb_log::fail;

use crate::port::{listener::Listener, listener::ListenerCreateError};
use crate::service;

use super::event::PortFactory;

/// Factory to create a new [`Listener`] port/endpoint for
/// [`MessagingPattern::Event`](crate::service::messaging_pattern::MessagingPattern::Event) based
/// communication.
#[derive(Debug, Clone)]
pub struct PortFactoryListener<'factory, Service: service::Service> {
    pub(crate) factory: &'factory PortFactory<Service>,
}

unsafe impl<Service: service::Service> Send for PortFactoryListener<'_, Service> {}

impl<Service: service::Service> PortFactoryListener<'_, Service> {
    /// Creates the [`Listener`] port or returns a [`ListenerCreateError`] on failure.
    pub fn create(self) -> Result<Listener<Service>, ListenerCreateError> {
        Ok(
            fail!(from self, when Listener::new(self.factory.service.clone()),
                    "Failed to create new Listener port."),
        )
    }
}
