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
//! ```
//! use iceoryx2::prelude::*;
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let event_name = ServiceName::new("MyEventName")?;
//! let event = zero_copy::Service::new(&event_name)
//!     .event()
//!     .open_or_create()?;
//!
//! let mut listener = event.listener().create()?;
//!
//! for event_id in listener.try_wait()? {
//!     println!("event was triggered with id: {:?}", event_id);
//! }
//!
//! # Ok(())
//! # }
//! ```
//!
//! See also [`crate::port::listener::Listener`]

use iceoryx2_bb_lock_free::mpmc::container::ContainerHandle;
use iceoryx2_bb_log::fail;
use iceoryx2_cal::dynamic_storage::DynamicStorage;
use iceoryx2_cal::event::{ListenerBuilder, ListenerWaitError};
use iceoryx2_cal::named_concept::NamedConceptBuilder;

use crate::service::naming_scheme::event_concept_name;
use crate::{port::port_identifiers::UniqueListenerId, service};
use std::rc::Rc;
use std::sync::atomic::Ordering;
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
        std::write!(f, "{}::{:?}", std::stringify!(Self), self)
    }
}

impl std::error::Error for ListenerCreateError {}

/// Represents the receiving endpoint of an event based communication.
#[derive(Debug)]
pub struct Listener<Service: service::Service> {
    dynamic_listener_handle: Option<ContainerHandle>,
    listener: <Service::Event as iceoryx2_cal::event::Event<EventId>>::Listener,
    cache: Vec<EventId>,
    dynamic_storage: Rc<Service::DynamicStorage>,
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
        let dynamic_storage = Rc::clone(&service.state().dynamic_storage);

        let listener = fail!(from origin,
                             when <Service::Event as iceoryx2_cal::event::Event<EventId>>::ListenerBuilder::new(&event_name).create(),
                             with ListenerCreateError::ResourceCreationFailed,
                             "{} since the underlying event concept \"{}\" could not be created.", msg, event_name);

        let mut new_self = Self {
            dynamic_storage,
            dynamic_listener_handle: None,
            listener,
            cache: vec![],
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

    fn fill_cache(&mut self) -> Result<(), ListenerWaitError> {
        use iceoryx2_cal::event::Listener;
        while let Some(id) = fail!(from self,
                when self.listener.try_wait(),
                "Failed to try_wait on Listener port since the underlying Listener concept failed.")
        {
            self.cache.push(id);
        }

        Ok(())
    }

    /// Returns the cached [`EventId`]s. Whenever [`Listener::try_wait()`],
    /// [`Listener::timed_wait()`] or [`Listener::blocking_wait()`] is called the cache is reset
    /// and filled with the events that where signaled since the last call. This cache can be
    /// accessed until a new wait call resets and fills it again.
    pub fn cache(&self) -> &[EventId] {
        &self.cache
    }

    /// Non-blocking wait for new [`EventId`]s. If no [`EventId`]s were notified the returned slice
    /// is empty. On error it returns [`ListenerWaitError`] is returned which describes the error
    /// in detail.
    pub fn try_wait(&mut self) -> Result<&[EventId], ListenerWaitError> {
        self.cache.clear();
        self.fill_cache()?;

        Ok(self.cache())
    }

    /// Blocking wait for new [`EventId`]s until either an [`EventId`] was received or the timeout
    /// has passed. If no [`EventId`]s were notified the returned slice
    /// is empty. On error it returns [`ListenerWaitError`] is returned which describes the error
    /// in detail.
    pub fn timed_wait(&mut self, timeout: Duration) -> Result<&[EventId], ListenerWaitError> {
        use iceoryx2_cal::event::Listener;
        self.cache.clear();

        if let Some(id) = fail!(from self,
            when self.listener.timed_wait(timeout),
            "Failed to timed_wait with timeout {:?} on Listener port since the underlying Listener concept failed.", timeout)
        {
            self.cache.push(id);
            self.fill_cache()?;
        }

        Ok(self.cache())
    }

    /// Blocking wait for new [`EventId`]s until either an [`EventId`].
    /// Sporadic wakeups can occur and if no [`EventId`]s were notified the returned slice
    /// is empty. On error it returns [`ListenerWaitError`] is returned which describes the error
    /// in detail.
    pub fn blocking_wait(&mut self) -> Result<&[EventId], ListenerWaitError> {
        use iceoryx2_cal::event::Listener;
        self.cache.clear();

        if let Some(id) = fail!(from self,
            when self.listener.blocking_wait(),
            "Failed to blocking_wait on Listener port since the underlying Listener concept failed.")
        {
            self.cache.push(id);
            self.fill_cache()?;
        }

        Ok(self.cache())
    }
}
