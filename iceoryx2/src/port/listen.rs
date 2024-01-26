// Copyright (c) 2023 - 2024 Contributors to the Eclipse Foundation
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
//! let mut listeners: Vec<Box<dyn Listen>> = vec![];
//!
//! listeners.push(Box::new(event_ipc.listener().create()?));
//! listeners.push(Box::new(event_local.listener().create()?));
//!
//! for listener in &mut listeners {
//!     for event_id in listener.try_wait()? {
//!         println!("event was triggered with id: {:?}", event_id);
//!     }
//! }
//!
//! # Ok(())
//! # }
//! ```
//!
//! See also [`crate::port::listener::Listener`]

use std::time::Duration;

use iceoryx2_cal::event::ListenerWaitError;

use super::event_id::EventId;

/// Defines the failures that can occur when a [`Listen`]er is created with the
/// [`crate::service::port_factory::listener::PortFactoryListener`].
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum ListenerCreateError {
    ExceedsMaxSupportedListeners,
    ResourceCreationFailed,
}

impl std::fmt::Display for ListenerCreateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::write!(f, "{}::{:?}", std::stringify!(Self), self)
    }
}

impl std::error::Error for ListenerCreateError {}

/// The interface of the receiving endpoint of an event based communication.
pub trait Listen {
    /// Returns the cached [`EventId`]s. Whenever [`Listen::try_wait()`],
    /// [`Listen::timed_wait()`] or [`Listen::blocking_wait()`] is called the cache is reset
    /// and filled with the events that where signaled since the last call. This cache can be
    /// accessed until a new wait call resets and fills it again.
    fn cache(&self) -> &[EventId];

    /// Non-blocking wait for new [`EventId`]s. If no [`EventId`]s were notified the returned slice
    /// is empty. On error it returns [`ListenerWaitError`] is returned which describes the error
    /// in detail.
    fn try_wait(&mut self) -> Result<&[EventId], ListenerWaitError>;

    /// Blocking wait for new [`EventId`]s until either an [`EventId`] was received or the timeout
    /// has passed. If no [`EventId`]s were notified the returned slice
    /// is empty. On error it returns [`ListenerWaitError`] is returned which describes the error
    /// in detail.
    fn timed_wait(&mut self, timeout: Duration) -> Result<&[EventId], ListenerWaitError>;

    /// Blocking wait for new [`EventId`]s until either an [`EventId`].
    /// Sporadic wakeups can occur and if no [`EventId`]s were notified the returned slice
    /// is empty. On error it returns [`ListenerWaitError`] is returned which describes the error
    /// in detail.
    fn blocking_wait(&mut self) -> Result<&[EventId], ListenerWaitError>;
}
