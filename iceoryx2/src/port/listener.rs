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

//! # Example
//!
//! ## Process Directly
//!
//! **Note:**
//! This approach may lead to an infinite loop when one notifier sends [`EventId`](crate::port::event_id::EventId)s in a busy
//! loop and the [`Listener`](crate::port::listener::Listener) tries to collect all of them before continuing with other operations.
//! If the listening algorithm consists only of one loop taking care of [`EventId`](crate::port::event_id::EventId)s without any
//! other operations outside of the loop, this problem can be ignored.
//!
//! ```
//! use iceoryx2::prelude::*;
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let node = NodeBuilder::new().create::<zero_copy::Service>()?;
//! let event_name = ServiceName::new("MyEventName")?;
//! let event = node.service(&event_name)
//!     .event()
//!     .open_or_create()?;
//!
//! let mut listener = event.listener().create()?;
//!
//! for event_id in listener.try_wait_one()? {
//!     println!("event was triggered with id: {:?}", event_id);
//! }
//!
//! # Ok(())
//! # }
//! ```
//!
//! ## Process Batch Of Events
//!
//! ```
//! use iceoryx2::prelude::*;
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let node = NodeBuilder::new().create::<zero_copy::Service>()?;
//! let event_name = ServiceName::new("MyEventName")?;
//! let event = node.service(&event_name)
//!     .event()
//!     .open_or_create()?;
//!
//! let mut listener = event.listener().create()?;
//!
//! listener.try_wait_all(|id| {
//!     println!("event was triggered with id: {:?}", id);
//! })?;
//!
//! # Ok(())
//! # }
//! ```

use iceoryx2_bb_lock_free::mpmc::container::ContainerHandle;
use iceoryx2_bb_log::fail;
use iceoryx2_cal::dynamic_storage::DynamicStorage;
use iceoryx2_cal::event::{ListenerBuilder, ListenerWaitError, TriggerId};
use iceoryx2_cal::named_concept::NamedConceptBuilder;

use crate::service::naming_scheme::event_concept_name;
use crate::{port::port_identifiers::UniqueListenerId, service};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

use super::event_id::EventId;

/// Defines the failures that can occur when a [`Listener`] is created with the
/// [`crate::service::port_factory::listener::PortFactoryListener`].
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum ListenerCreateError {
    ExceedsMaxSupportedListeners,
    ResourceCreationFailed,
}

impl std::fmt::Display for ListenerCreateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::write!(f, "ListenerCreateError::{:?}", self)
    }
}

impl std::error::Error for ListenerCreateError {}

/// Represents the receiving endpoint of an event based communication.
#[derive(Debug)]
pub struct Listener<Service: service::Service> {
    dynamic_listener_handle: Option<ContainerHandle>,
    listener: <Service::Event as iceoryx2_cal::event::Event>::Listener,
    dynamic_storage: Arc<Service::DynamicStorage>,
    port_id: UniqueListenerId,
}

impl<Service: service::Service> Drop for Listener<Service> {
    fn drop(&mut self) {
        if let Some(handle) = self.dynamic_listener_handle {
            self.dynamic_storage
                .get()
                .event()
                .release_listener_handle(handle)
        }
    }
}

impl<Service: service::Service> Listener<Service> {
    pub(crate) fn new(service: &Service) -> Result<Self, ListenerCreateError> {
        let msg = "Failed to create listener";
        let origin = "Listener::new()";
        let port_id = UniqueListenerId::new();

        let event_name = event_concept_name(&port_id);
        let dynamic_storage = Arc::clone(&service.state().dynamic_storage);

        let listener = fail!(from origin,
                             when <Service::Event as iceoryx2_cal::event::Event>::ListenerBuilder::new(&event_name)
                                .trigger_id_max(TriggerId::new(service.state().static_config.event().event_id_max_value))
                                .create(),
                             with ListenerCreateError::ResourceCreationFailed,
                             "{} since the underlying event concept \"{}\" could not be created.", msg, event_name);

        let mut new_self = Self {
            dynamic_storage,
            dynamic_listener_handle: None,
            listener,
            port_id,
        };

        std::sync::atomic::compiler_fence(Ordering::SeqCst);

        // !MUST! be the last task otherwise a listener is added to the dynamic config without
        // the creation of all required channels
        let dynamic_listener_handle = match service
            .state()
            .dynamic_storage
            .get()
            .event()
            .add_listener_id(port_id)
        {
            Some(unique_index) => unique_index,
            None => {
                fail!(from origin, with ListenerCreateError::ExceedsMaxSupportedListeners,
                                 "{} since it would exceed the maximum supported amount of listeners of {}.",
                                 msg, service.state().static_config.event().max_listeners);
            }
        };

        new_self.dynamic_listener_handle = Some(dynamic_listener_handle);

        Ok(new_self)
    }

    /// Non-blocking wait for new [`EventId`]s. Collects either all [`EventId`]s that were received
    /// until the call of [`Listener::try_wait_all()`] or a reasonable batch that represent the
    /// currently available [`EventId`]s in buffer.
    /// For every received [`EventId`] the provided callback is called with the [`EventId`] as
    /// input argument.
    pub fn try_wait_all<F: FnMut(EventId)>(&self, callback: F) -> Result<(), ListenerWaitError> {
        use iceoryx2_cal::event::Listener;
        Ok(fail!(from self, when self.listener.try_wait_all(callback),
            "Failed to while calling try_wait on underlying event::Listener"))
    }

    /// Blocking wait for new [`EventId`]s until the provided timeout has passed. Collects either
    /// all [`EventId`]s that were received
    /// until the call of [`Listener::timed_wait_all()`] or a reasonable batch that represent the
    /// currently available [`EventId`]s in buffer.
    /// For every received [`EventId`] the provided callback is called with the [`EventId`] as
    /// input argument.
    pub fn timed_wait_all<F: FnMut(EventId)>(
        &self,
        callback: F,
        timeout: Duration,
    ) -> Result<(), ListenerWaitError> {
        use iceoryx2_cal::event::Listener;
        Ok(
            fail!(from self, when self.listener.timed_wait_all(callback, timeout),
            "Failed to while calling timed_wait({:?}) on underlying event::Listener", timeout),
        )
    }

    /// Blocking wait for new [`EventId`]s. Collects either
    /// all [`EventId`]s that were received
    /// until the call of [`Listener::timed_wait_all()`] or a reasonable batch that represent the
    /// currently available [`EventId`]s in buffer.
    /// For every received [`EventId`] the provided callback is called with the [`EventId`] as
    /// input argument.
    pub fn blocking_wait_all<F: FnMut(EventId)>(
        &self,
        callback: F,
    ) -> Result<(), ListenerWaitError> {
        use iceoryx2_cal::event::Listener;
        Ok(
            fail!(from self, when self.listener.blocking_wait_all(callback),
            "Failed to while calling blocking_wait on underlying event::Listener"),
        )
    }

    /// Non-blocking wait for a new [`EventId`]. If no [`EventId`] was notified it returns [`None`].
    /// On error it returns [`ListenerWaitError`] is returned which describes the error
    /// in detail.
    pub fn try_wait_one(&self) -> Result<Option<EventId>, ListenerWaitError> {
        use iceoryx2_cal::event::Listener;
        Ok(fail!(from self, when self.listener.try_wait_one(),
            "Failed to while calling try_wait on underlying event::Listener"))
    }

    /// Blocking wait for a new [`EventId`] until either an [`EventId`] was received or the timeout
    /// has passed. If no [`EventId`] was notified it returns [`None`].
    /// On error it returns [`ListenerWaitError`] is returned which describes the error
    /// in detail.
    pub fn timed_wait_one(&self, timeout: Duration) -> Result<Option<EventId>, ListenerWaitError> {
        use iceoryx2_cal::event::Listener;
        Ok(fail!(from self, when self.listener.timed_wait_one(timeout),
            "Failed to while calling timed_wait({:?}) on underlying event::Listener", timeout))
    }

    /// Blocking wait for a new [`EventId`].
    /// Sporadic wakeups can occur and if no [`EventId`] was notified it returns [`None`].
    /// On error it returns [`ListenerWaitError`] is returned which describes the error
    /// in detail.
    pub fn blocking_wait_one(&self) -> Result<Option<EventId>, ListenerWaitError> {
        use iceoryx2_cal::event::Listener;
        Ok(fail!(from self, when self.listener.blocking_wait_one(),
            "Failed to while calling blocking_wait on underlying event::Listener"))
    }

    /// Returns the [`UniqueListenerId`] of the [`Listener`]
    pub fn id(&self) -> UniqueListenerId {
        self.port_id
    }
}
