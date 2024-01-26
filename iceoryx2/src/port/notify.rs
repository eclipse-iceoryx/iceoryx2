// Copyright (c) 2024 Contributors to the Eclipse Foundation
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
//! use std::boxed::Box;
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let event_name = ServiceName::new("MyEventName")?;
//! let event_ipc = zero_copy::Service::new(&event_name)
//!     .event()
//!     .open_or_create()?;
//!
//! let event_local = process_local::Service::new(&event_name)
//!     .event()
//!     .open_or_create()?;
//!
//! let mut notifiers: Vec<Box<dyn Notify>> = vec![];
//!
//! notifiers.push(Box::new(event_ipc.notifier().default_event_id(EventId::new(123)).create()?));
//! notifiers.push(Box::new(event_local.notifier().default_event_id(EventId::new(456)).create()?));
//!
//! for notifier in &notifiers {
//!     notifier.notify()?;
//! }
//!
//! # Ok(())
//! # }
//! ```
//!
//! See also [`crate::port::notifier::Notifier`]

use super::event_id::EventId;

/// Failures that can occur when a new [`Notify`]er is created with the
/// [`crate::service::port_factory::notifier::PortFactoryNotifier`].
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum NotifierCreateError {
    ExceedsMaxSupportedNotifiers,
}

impl std::fmt::Display for NotifierCreateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::write!(f, "{}::{:?}", std::stringify!(Self), self)
    }
}

impl std::error::Error for NotifierCreateError {}

/// Defines the failures that can occur while a [`Notify::notify()`] call.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum NotifierConnectionUpdateFailure {
    OnlyPartialUpdate,
}

impl std::fmt::Display for NotifierConnectionUpdateFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::write!(f, "{}::{:?}", std::stringify!(Self), self)
    }
}

impl std::error::Error for NotifierConnectionUpdateFailure {}

/// The interface of the sending endpoint of an event based communication.
pub trait Notify {
    /// Notifies all [`crate::port::listener::Listener`] connected to the service with the default
    /// event id provided on creation.
    /// On success the number of
    /// [`crate::port::listener::Listener`]s that were notified otherwise it returns
    /// [`NotifierConnectionUpdateFailure`].
    fn notify(&self) -> Result<usize, NotifierConnectionUpdateFailure>;

    /// Notifies all [`crate::port::listener::Listener`] connected to the service with a custom
    /// [`EventId`].
    /// On success the number of
    /// [`crate::port::listener::Listener`]s that were notified otherwise it returns
    /// [`NotifierConnectionUpdateFailure`].
    fn notify_with_custom_event_id(
        &self,
        value: EventId,
    ) -> Result<usize, NotifierConnectionUpdateFailure>;
}
