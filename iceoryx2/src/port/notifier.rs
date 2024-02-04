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
//! let notifier = event
//!     .notifier()
//!     .default_event_id(EventId::new(123))
//!     .create()?;
//!
//! // notify with default event id 123
//! notifier.notify()?;
//!
//! // notify with some custom event id
//! notifier.notify_with_custom_event_id(EventId::new(456))?;
//!
//! # Ok(())
//! # }
//! ```
//!
//! See also [`crate::port::notifier::Notifier`]

use crate::{
    port::port_identifiers::UniqueNotifierId,
    service::{self, naming_scheme::event_concept_name},
};
use iceoryx2_bb_lock_free::mpmc::container::ContainerState;
use iceoryx2_bb_log::{fail, warn};
use iceoryx2_cal::named_concept::NamedConceptBuilder;
use iceoryx2_cal::{dynamic_storage::DynamicStorage, event::NotifierBuilder};
use std::{cell::UnsafeCell, rc::Rc};

use super::{
    event_id::EventId,
    notify::{NotifierConnectionUpdateFailure, NotifierCreateError, Notify},
    port_identifiers::UniqueListenerId,
};

#[derive(Debug, Default)]
struct ListenerConnections<Service: service::Service> {
    #[allow(clippy::type_complexity)]
    connections:
        Vec<UnsafeCell<Option<<Service::Event as iceoryx2_cal::event::Event<EventId>>::Notifier>>>,
}

impl<Service: service::Service> ListenerConnections<Service> {
    fn new(size: usize) -> Self {
        let mut new_self = Self {
            connections: vec![],
        };

        new_self.connections.reserve(size);
        for _ in 0..size {
            new_self.connections.push(UnsafeCell::new(None))
        }

        new_self
    }

    fn create(&self, index: usize, listener_id: UniqueListenerId) -> Result<(), ()> {
        let event_name = event_concept_name(&listener_id);
        if self.get(index).is_none() {
            let notifier = fail!(from self, when <Service::Event as iceoryx2_cal::event::Event<EventId>>::NotifierBuilder::new(&event_name).open(),
                                    with (),
                                    "Unable to establish a connection to Listener port {:?}.", listener_id);
            *self.get_mut(index) = Some(notifier);
        }

        Ok(())
    }

    fn get(
        &self,
        index: usize,
    ) -> &Option<<Service::Event as iceoryx2_cal::event::Event<EventId>>::Notifier> {
        unsafe { &(*self.connections[index].get()) }
    }

    #[allow(clippy::mut_from_ref)]
    fn get_mut(
        &self,
        index: usize,
    ) -> &mut Option<<Service::Event as iceoryx2_cal::event::Event<EventId>>::Notifier> {
        unsafe { &mut (*self.connections[index].get()) }
    }

    fn len(&self) -> usize {
        self.connections.len()
    }

    fn remove(&self, index: usize) {
        *self.get_mut(index) = None;
    }
}

/// Represents the sending endpoint of an event based communication.
#[derive(Debug)]
pub struct Notifier<Service: service::Service> {
    listener_connections: ListenerConnections<Service>,
    listener_list_state: UnsafeCell<ContainerState<UniqueListenerId>>,
    default_event_id: EventId,
    dynamic_storage: Rc<Service::DynamicStorage>,
    dynamic_notifier_handle: u32,
}

impl<Service: service::Service> Drop for Notifier<Service> {
    fn drop(&mut self) {
        self.dynamic_storage
            .get()
            .event()
            .release_notifier_id(self.dynamic_notifier_handle)
    }
}

impl<Service: service::Service> Notifier<Service> {
    pub(crate) fn new(
        service: &Service,
        default_event_id: EventId,
    ) -> Result<Self, NotifierCreateError> {
        let msg = "Unable to create Notifier port";
        let origin = "Notifier::new()";
        let port_id = UniqueNotifierId::new();

        let listener_list = &service.state().dynamic_storage.get().event().listeners;
        let dynamic_storage = Rc::clone(&service.state().dynamic_storage);

        let dynamic_notifier_handle = match dynamic_storage.get().event().add_notifier_id(port_id) {
            Some(handle) => handle,
            None => {
                fail!(from origin, with NotifierCreateError::ExceedsMaxSupportedNotifiers,
                            "{} since it would exceed the maximum supported amount of notifiers of {}.",
                            msg, service.state().static_config.event().max_notifiers);
            }
        };

        let new_self = Self {
            listener_connections: ListenerConnections::new(listener_list.capacity()),
            default_event_id,
            listener_list_state: unsafe { UnsafeCell::new(listener_list.get_state()) },
            dynamic_storage,
            dynamic_notifier_handle,
        };

        if let Err(e) = new_self.populate_listener_channels() {
            warn!(from new_self, "The new Notifier port is unable to connect to every Listener port, caused by {:?}.", e);
        }

        Ok(new_self)
    }

    fn update_connections(&self) -> Result<(), NotifierConnectionUpdateFailure> {
        if unsafe {
            self.dynamic_storage
                .get()
                .event()
                .listeners
                .update_state(&mut *self.listener_list_state.get())
        } {
            fail!(from self, when self.populate_listener_channels(),
                with NotifierConnectionUpdateFailure::OnlyPartialUpdate,
                "Connections were updated only partially since at least one connection to a Listener port failed.");
        }

        Ok(())
    }

    fn populate_listener_channels(&self) -> Result<(), ()> {
        let mut visited_indices = vec![];
        visited_indices.resize(self.listener_connections.len(), None);

        unsafe {
            (*self.listener_list_state.get()).for_each(|index, listener_id| {
                visited_indices[index as usize] = Some(*listener_id);
            })
        };

        for (i, index) in visited_indices.iter().enumerate() {
            match index {
                Some(listener_id) => match self.listener_connections.create(i, *listener_id) {
                    Ok(()) => (),
                    Err(()) => {
                        fail!(from self, with (),
                            "Unable to establish connection to Listener port {:?}.", *listener_id);
                    }
                },
                None => self.listener_connections.remove(i),
            }
        }

        Ok(())
    }
}

impl<Service: service::Service> Notify for Notifier<Service> {
    fn notify(&self) -> Result<usize, NotifierConnectionUpdateFailure> {
        self.notify_with_custom_event_id(self.default_event_id)
    }

    fn notify_with_custom_event_id(
        &self,
        value: EventId,
    ) -> Result<usize, NotifierConnectionUpdateFailure> {
        fail!(from self, when self.update_connections(),
            "Unable to notify event since the connections could not be updated.");

        use iceoryx2_cal::event::Notifier;
        let mut number_of_triggered_listeners = 0;

        for i in 0..self.listener_connections.len() {
            match self.listener_connections.get(i) {
                Some(ref connection) => match connection.notify(value) {
                    Err(e) => {
                        warn!(from self, "Unable to send notification via connection {:?} due to {:?}.", connection, e)
                    }
                    Ok(_) => {
                        number_of_triggered_listeners += 1;
                    }
                },
                None => (),
            }
        }

        Ok(number_of_triggered_listeners)
    }
}
