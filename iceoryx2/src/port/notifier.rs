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
//!     .default_event_id(EventId::new(12))
//!     .create()?;
//!
//! // notify with default event id 123
//! notifier.notify()?;
//!
//! // notify with some custom event id
//! notifier.notify_with_custom_event_id(EventId::new(6))?;
//!
//! # Ok(())
//! # }
//! ```

use super::{event_id::EventId, port_identifiers::UniqueListenerId};
use crate::{
    port::port_identifiers::UniqueNotifierId,
    service::{self, naming_scheme::event_concept_name},
};
use iceoryx2_bb_lock_free::mpmc::container::{ContainerHandle, ContainerState};
use iceoryx2_bb_log::{debug, fail, warn};
use iceoryx2_cal::{dynamic_storage::DynamicStorage, event::NotifierBuilder};
use iceoryx2_cal::{event::Event, named_concept::NamedConceptBuilder};
use std::{
    cell::UnsafeCell,
    sync::{atomic::Ordering, Arc},
};

/// Failures that can occur when a new [`Notifier`] is created with the
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

/// Defines the failures that can occur while a [`Notifier::notify()`] call.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum NotifierNotifyError {
    EventIdOutOfBounds,
}

impl std::fmt::Display for NotifierNotifyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::write!(f, "{}::{:?}", std::stringify!(Self), self)
    }
}

impl std::error::Error for NotifierNotifyError {}

#[derive(Debug)]
struct Connection<Service: service::Service> {
    notifier: <Service::Event as Event>::Notifier,
    listener_id: UniqueListenerId,
}

#[derive(Debug, Default)]
struct ListenerConnections<Service: service::Service> {
    #[allow(clippy::type_complexity)]
    connections: Vec<UnsafeCell<Option<Connection<Service>>>>,
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

    fn create(&self, index: usize, listener_id: UniqueListenerId) {
        let msg = "Unable to establish connection to listener";
        let event_name = event_concept_name(&listener_id);
        if self.get(index).is_none() {
            match <Service::Event as iceoryx2_cal::event::Event>::NotifierBuilder::new(&event_name)
                .open()
            {
                Ok(notifier) => {
                    *self.get_mut(index) = Some(Connection {
                        notifier,
                        listener_id,
                    });
                }
                Err(
                    iceoryx2_cal::event::NotifierCreateError::DoesNotExist
                    | iceoryx2_cal::event::NotifierCreateError::InitializationNotYetFinalized,
                ) => (),
                Err(iceoryx2_cal::event::NotifierCreateError::VersionMismatch) => {
                    warn!(from self,
                        "{} since a version mismatch was detected! All entities must use the same iceoryx2 version!",
                        msg);
                }
                Err(iceoryx2_cal::event::NotifierCreateError::InsufficientPermissions) => {
                    warn!(from self, "{} since the permissions do not match. The service or the participants are maybe misconfigured.", msg);
                }
                Err(iceoryx2_cal::event::NotifierCreateError::InternalFailure) => {
                    debug!(from self, "{} due to an internal failure.", msg);
                }
            }
        }
    }

    fn get(&self, index: usize) -> &Option<Connection<Service>> {
        unsafe { &(*self.connections[index].get()) }
    }

    #[allow(clippy::mut_from_ref)]
    fn get_mut(&self, index: usize) -> &mut Option<Connection<Service>> {
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
    event_id_max_value: usize,
    dynamic_storage: Arc<Service::DynamicStorage>,
    dynamic_notifier_handle: Option<ContainerHandle>,
    port_id: UniqueNotifierId,
}

impl<Service: service::Service> Drop for Notifier<Service> {
    fn drop(&mut self) {
        if let Some(handle) = self.dynamic_notifier_handle {
            self.dynamic_storage
                .get()
                .event()
                .release_notifier_handle(handle)
        }
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
        let dynamic_storage = Arc::clone(&service.state().dynamic_storage);

        let mut new_self = Self {
            listener_connections: ListenerConnections::new(listener_list.capacity()),
            default_event_id,
            listener_list_state: unsafe { UnsafeCell::new(listener_list.get_state()) },
            dynamic_storage,
            event_id_max_value: service.state().static_config.event().event_id_max_value,
            dynamic_notifier_handle: None,
            port_id,
        };

        new_self.populate_listener_channels();

        std::sync::atomic::compiler_fence(Ordering::SeqCst);

        // !MUST! be the last task otherwise a notifier is added to the dynamic config without
        // the creation of all required channels
        let dynamic_notifier_handle = match new_self
            .dynamic_storage
            .get()
            .event()
            .add_notifier_id(port_id)
        {
            Some(handle) => handle,
            None => {
                fail!(from origin, with NotifierCreateError::ExceedsMaxSupportedNotifiers,
                            "{} since it would exceed the maximum supported amount of notifiers of {}.",
                            msg, service.state().static_config.event().max_notifiers);
            }
        };
        new_self.dynamic_notifier_handle = Some(dynamic_notifier_handle);

        Ok(new_self)
    }

    fn update_connections(&self) {
        if unsafe {
            self.dynamic_storage
                .get()
                .event()
                .listeners
                .update_state(&mut *self.listener_list_state.get())
        } {
            self.populate_listener_channels();
        }
    }

    fn populate_listener_channels(&self) {
        let mut visited_indices = vec![];
        visited_indices.resize(self.listener_connections.len(), None);

        unsafe {
            (*self.listener_list_state.get()).for_each(|index, listener_id| {
                visited_indices[index as usize] = Some(*listener_id);
            })
        };

        for (i, index) in visited_indices.iter().enumerate() {
            match index {
                Some(listener_id) => {
                    let create_connection = match self.listener_connections.get(i) {
                        None => true,
                        Some(connection) => {
                            let is_connected = connection.listener_id != *listener_id;
                            if is_connected {
                                self.listener_connections.remove(i);
                            }
                            is_connected
                        }
                    };

                    if create_connection {
                        self.listener_connections.create(i, *listener_id);
                    }
                }
                None => self.listener_connections.remove(i),
            }
        }
    }

    /// Returns the [`UniqueNotifierId`] of the [`Notifier`]
    pub fn id(&self) -> UniqueNotifierId {
        self.port_id
    }

    /// Notifies all [`crate::port::listener::Listener`] connected to the service with the default
    /// event id provided on creation.
    /// On success the number of
    /// [`crate::port::listener::Listener`]s that were notified otherwise it returns
    /// [`NotifierNotifyError`].
    pub fn notify(&self) -> Result<usize, NotifierNotifyError> {
        self.notify_with_custom_event_id(self.default_event_id)
    }

    /// Notifies all [`crate::port::listener::Listener`] connected to the service with a custom
    /// [`EventId`].
    /// On success the number of
    /// [`crate::port::listener::Listener`]s that were notified otherwise it returns
    /// [`NotifierNotifyError`].
    pub fn notify_with_custom_event_id(
        &self,
        value: EventId,
    ) -> Result<usize, NotifierNotifyError> {
        let msg = "Unable to notify event";
        self.update_connections();

        use iceoryx2_cal::event::Notifier;
        let mut number_of_triggered_listeners = 0;

        if self.event_id_max_value < value.as_value() {
            fail!(from self, with NotifierNotifyError::EventIdOutOfBounds,
                            "{} since the EventId {:?} exceeds the maximum supported EventId value of {}.",
                            msg, value, self.event_id_max_value);
        }

        for i in 0..self.listener_connections.len() {
            match self.listener_connections.get(i) {
                Some(ref connection) => match connection.notifier.notify(value) {
                    Err(iceoryx2_cal::event::NotifierNotifyError::Disconnected) => {
                        self.listener_connections.remove(i);
                    }
                    Err(e) => {
                        warn!(from self, "Unable to send notification via connection {:?} due to {:?}.",
                            connection, e)
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
