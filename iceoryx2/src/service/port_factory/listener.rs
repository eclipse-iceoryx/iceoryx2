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
//! let listener = event.listener().create()?;
//! # Ok(())
//! # }
//! ```
use std::fmt::Debug;

use iceoryx2_bb_log::fail;

use crate::port::listener::{Listener, ListenerCreateError};
use crate::service;

use super::event::PortFactory;

/// Factory to create a new [`Listener`] port/endpoint for
/// [`MessagingPattern::Event`](crate::service::messaging_pattern::MessagingPattern::Event) based
/// communication.
#[derive(Debug)]
pub struct PortFactoryListener<'factory, 'config, Service: service::Details<'config>> {
    pub(crate) factory: &'factory PortFactory<'config, Service>,
}

impl<'factory, 'config, Service: service::Details<'config>>
    PortFactoryListener<'factory, 'config, Service>
{
    /// Creates the [`Listener`] port or returns a [`ListenerCreateError`] on failure.
    pub fn create(&self) -> Result<Listener<'factory, 'config, Service>, ListenerCreateError> {
        Ok(fail!(from self, when Listener::new(&self.factory.service),
                    "Failed to create new Listener port."))
    }
}
