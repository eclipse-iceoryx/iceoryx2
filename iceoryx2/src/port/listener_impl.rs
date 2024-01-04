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

use iceoryx2_bb_lock_free::mpmc::unique_index_set::UniqueIndex;
use iceoryx2_bb_log::fail;
use iceoryx2_cal::dynamic_storage::DynamicStorage;
use iceoryx2_cal::event::{ListenerBuilder, ListenerWaitError};
use iceoryx2_cal::named_concept::NamedConceptBuilder;

use crate::service::naming_scheme::event_concept_name;
use crate::{port::port_identifiers::UniqueListenerId, service};
use std::{marker::PhantomData, time::Duration};

use super::event_id::EventId;
use super::listener::{Listener, ListenerCreateError};

/// Represents the receiving endpoint of an event based communication.
#[derive(Debug)]
pub struct ListenerImpl<'a, 'config: 'a, Service: service::Details<'config>> {
    _dynamic_config_guard: Option<UniqueIndex<'a>>,
    listener: <Service::Event as iceoryx2_cal::event::Event<EventId>>::Listener,
    cache: Vec<EventId>,
    _phantom_a: PhantomData<&'a Service>,
    _phantom_b: PhantomData<&'config ()>,
}

impl<'a, 'config: 'a, Service: service::Details<'config>> ListenerImpl<'a, 'config, Service> {
    pub(crate) fn new(service: &'a Service) -> Result<Self, ListenerCreateError> {
        let msg = "Failed to create listener";
        let origin = "Listener::new()";
        let port_id = UniqueListenerId::new();

        let event_name = event_concept_name(&port_id);
        let listener = fail!(from origin,
                             when <Service::Event as iceoryx2_cal::event::Event<EventId>>::ListenerBuilder::new(&event_name).create(),
                             with ListenerCreateError::ResourceCreationFailed,
                             "{} since the underlying event concept \"{}\" could not be created.", msg, event_name);

        let mut new_self = Self {
            _dynamic_config_guard: None,
            listener,
            cache: vec![],
            _phantom_a: PhantomData,
            _phantom_b: PhantomData,
        };

        // !MUST! be the last task otherwise a listener is added to the dynamic config without
        // the creation of all required channels
        new_self._dynamic_config_guard = Some(
            match service
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
            },
        );

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

impl<'a, 'config: 'a, Service: service::Details<'config>> Listener
    for ListenerImpl<'a, 'config, Service>
{
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
