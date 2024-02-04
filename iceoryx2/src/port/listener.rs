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

use iceoryx2_bb_log::fail;
use iceoryx2_cal::dynamic_storage::DynamicStorage;
use iceoryx2_cal::event::{ListenerBuilder, ListenerWaitError};
use iceoryx2_cal::named_concept::NamedConceptBuilder;

use crate::service::naming_scheme::event_concept_name;
use crate::{port::port_identifiers::UniqueListenerId, service};
use std::rc::Rc;
use std::time::Duration;

use super::event_id::EventId;
use super::listen::{Listen, ListenerCreateError};

/// Represents the receiving endpoint of an event based communication.
#[derive(Debug)]
pub struct Listener<Service: service::Service> {
    dynamic_listener_handle: u32,
    listener: <Service::Event as iceoryx2_cal::event::Event<EventId>>::Listener,
    cache: Vec<EventId>,
    dynamic_storage: Rc<Service::DynamicStorage>,
}

impl<Service: service::Service> Drop for Listener<Service> {
    fn drop(&mut self) {
        self.dynamic_storage
            .get()
            .event()
            .release_listener_id(self.dynamic_listener_handle)
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

        let new_self = Self {
            dynamic_storage,
            dynamic_listener_handle,
            listener,
            cache: vec![],
        };

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
}

impl<Service: service::Service> Listen for Listener<Service> {
    fn cache(&self) -> &[EventId] {
        &self.cache
    }

    fn try_wait(&mut self) -> Result<&[EventId], ListenerWaitError> {
        self.cache.clear();
        self.fill_cache()?;

        Ok(self.cache())
    }

    fn timed_wait(&mut self, timeout: Duration) -> Result<&[EventId], ListenerWaitError> {
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

    fn blocking_wait(&mut self) -> Result<&[EventId], ListenerWaitError> {
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
