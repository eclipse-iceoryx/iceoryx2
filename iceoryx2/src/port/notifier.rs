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
//! # fn main() -> Result<(), Box<dyn core::error::Error>> {
//! let node = NodeBuilder::new().create::<ipc::Service>()?;
//! let event = node.service_builder(&"MyEventName".try_into()?)
//!     .event()
//!     .open_or_create()?;
//!
//! let notifier = event
//!     .notifier_builder()
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
    service::{
        self,
        config_scheme::event_config,
        dynamic_config::event::{ListenerDetails, NotifierDetails},
        naming_scheme::event_concept_name,
        ServiceState,
    },
};
use iceoryx2_bb_elementary::CallbackProgression;
use iceoryx2_bb_lock_free::mpmc::container::{ContainerHandle, ContainerState};
use iceoryx2_bb_log::{debug, fail, warn};
use iceoryx2_cal::{dynamic_storage::DynamicStorage, event::NotifierBuilder};
use iceoryx2_cal::{event::Event, named_concept::NamedConceptBuilder};

use core::{cell::UnsafeCell, sync::atomic::Ordering, time::Duration};

extern crate alloc;
use alloc::sync::Arc;

/// Failures that can occur when a new [`Notifier`] is created with the
/// [`crate::service::port_factory::notifier::PortFactoryNotifier`].
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum NotifierCreateError {
    /// The maximum amount of [`Notifier`]s that can connect to a
    /// [`Service`](crate::service::Service) is
    /// defined in [`crate::config::Config`]. When this is exceeded no more [`Notifier`]s
    /// can be created for a specific [`Service`](crate::service::Service).
    ExceedsMaxSupportedNotifiers,
}

impl core::fmt::Display for NotifierCreateError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        std::write!(f, "NotifierCreateError::{:?}", self)
    }
}

impl core::error::Error for NotifierCreateError {}

/// Defines the failures that can occur while a [`Notifier::notify()`] call.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum NotifierNotifyError {
    /// A [`Notifier::notify_with_custom_event_id()`] was called and the provided [`EventId`]
    /// is greater than the maximum supported [`EventId`] by the
    /// [`Service`](crate::service::Service)
    EventIdOutOfBounds,
    /// The notification was delivered to all [`Listener`](crate::port::listener::Listener) ports
    /// but the deadline contract, the maximum time span between two notifications, of the
    /// [`Service`](crate::service::Service) was violated.
    MissedDeadline,
    /// The notification was delivered but the elapsed system time could not be acquired.
    /// Therefore, it is unknown if the deadline was missed or not.
    UnableToAcquireElapsedTime,
}

impl core::fmt::Display for NotifierNotifyError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        std::write!(f, "NotifierNotifyError::{:?}", self)
    }
}

impl core::error::Error for NotifierNotifyError {}

#[derive(Debug)]
struct Connection<Service: service::Service> {
    notifier: <Service::Event as Event>::Notifier,
    listener_id: UniqueListenerId,
}

#[derive(Debug)]
struct ListenerConnections<Service: service::Service> {
    #[allow(clippy::type_complexity)]
    connections: Vec<UnsafeCell<Option<Connection<Service>>>>,
    service_state: Arc<ServiceState<Service>>,
}

impl<Service: service::Service> ListenerConnections<Service> {
    fn new(size: usize, service_state: Arc<ServiceState<Service>>) -> Self {
        let mut new_self = Self {
            connections: vec![],
            service_state,
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
        let event_config = event_config::<Service>(self.service_state.shared_node.config());
        if self.get(index).is_none() {
            match <Service::Event as iceoryx2_cal::event::Event>::NotifierBuilder::new(&event_name)
                .config(&event_config)
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
                Err(iceoryx2_cal::event::NotifierCreateError::Interrupt) => {
                    debug!(from self, "{} since an interrupt signal was received.", msg);
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
    listener_list_state: UnsafeCell<ContainerState<ListenerDetails>>,
    default_event_id: EventId,
    event_id_max_value: usize,
    dynamic_notifier_handle: Option<ContainerHandle>,
    notifier_id: UniqueNotifierId,
    on_drop_notification: Option<EventId>,
}

impl<Service: service::Service> Drop for Notifier<Service> {
    fn drop(&mut self) {
        if let Some(event_id) = self.on_drop_notification {
            if let Err(e) = self.notify_with_custom_event_id(event_id) {
                warn!(from self, "Unable to send notifier_dropped_event {:?} due to ({:?}).",
                    event_id, e);
            }
        }

        if let Some(handle) = self.dynamic_notifier_handle {
            self.listener_connections
                .service_state
                .dynamic_storage
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
        let mut new_self = Self::new_without_auto_event_emission(service, default_event_id)?;

        let static_config = service.__internal_state().static_config.event();
        new_self.on_drop_notification = static_config.notifier_dropped_event.map(EventId::new);

        if let Some(event_id) = static_config.notifier_created_event() {
            match new_self.notify_with_custom_event_id(event_id) {
                Ok(_)
                | Err(
                    NotifierNotifyError::MissedDeadline
                    | NotifierNotifyError::UnableToAcquireElapsedTime,
                ) => (),
                Err(e) => {
                    warn!(from new_self,
                        "The new notifier was unable to send out the notifier_created_event: {:?} due to ({:?}).",
                        event_id, e);
                }
            }
        }

        Ok(new_self)
    }

    pub(crate) fn new_without_auto_event_emission(
        service: &Service,
        default_event_id: EventId,
    ) -> Result<Self, NotifierCreateError> {
        let msg = "Unable to create Notifier port";
        let origin = "Notifier::new()";
        let notifier_id = UniqueNotifierId::new();

        let listener_list = &service
            .__internal_state()
            .dynamic_storage
            .get()
            .event()
            .listeners;

        let static_config = service.__internal_state().static_config.event();
        let mut new_self = Self {
            listener_connections: ListenerConnections::new(
                listener_list.capacity(),
                service.__internal_state().clone(),
            ),
            default_event_id,
            listener_list_state: UnsafeCell::new(unsafe { listener_list.get_state() }),
            event_id_max_value: static_config.event_id_max_value,
            dynamic_notifier_handle: None,
            notifier_id,
            on_drop_notification: None,
        };

        new_self.populate_listener_channels();

        core::sync::atomic::compiler_fence(Ordering::SeqCst);

        // !MUST! be the last task otherwise a notifier is added to the dynamic config without
        // the creation of all required channels
        let dynamic_notifier_handle = match new_self
            .listener_connections
            .service_state
            .dynamic_storage
            .get()
            .event()
            .add_notifier_id(NotifierDetails {
                notifier_id,
                node_id: *service.__internal_state().shared_node.id(),
            }) {
            Some(handle) => handle,
            None => {
                fail!(from origin, with NotifierCreateError::ExceedsMaxSupportedNotifiers,
                            "{} since it would exceed the maximum supported amount of notifiers of {}.",
                            msg, service.__internal_state().static_config.event().max_notifiers);
            }
        };
        new_self.dynamic_notifier_handle = Some(dynamic_notifier_handle);

        Ok(new_self)
    }

    fn update_connections(&self) {
        if unsafe {
            self.listener_connections
                .service_state
                .dynamic_storage
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
            (*self.listener_list_state.get()).for_each(|h, listener_id| {
                visited_indices[h.index() as usize] = Some(*listener_id);
                CallbackProgression::Continue
            })
        };

        for (i, index) in visited_indices.iter().enumerate() {
            match index {
                Some(details) => {
                    let create_connection = match self.listener_connections.get(i) {
                        None => true,
                        Some(connection) => {
                            let is_connected = connection.listener_id != details.listener_id;
                            if is_connected {
                                self.listener_connections.remove(i);
                            }
                            is_connected
                        }
                    };

                    if create_connection {
                        self.listener_connections.create(i, details.listener_id);
                    }
                }
                None => self.listener_connections.remove(i),
            }
        }
    }

    /// Returns the [`UniqueNotifierId`] of the [`Notifier`]
    pub fn id(&self) -> UniqueNotifierId {
        self.notifier_id
    }

    /// Notifies all [`crate::port::listener::Listener`] connected to the service with the default
    /// event id provided on creation.
    /// On success the number of
    /// [`crate::port::listener::Listener`]s that were notified otherwise it returns
    /// [`NotifierNotifyError`].
    pub fn notify(&self) -> Result<usize, NotifierNotifyError> {
        self.notify_with_custom_event_id(self.default_event_id)
    }

    /// Returns the deadline of the corresponding [`Service`](crate::service::Service).
    pub fn deadline(&self) -> Option<Duration> {
        self.listener_connections
            .service_state
            .static_config
            .event()
            .deadline
            .map(|v| v.value)
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
            if let Some(ref connection) = self.listener_connections.get(i) {
                match connection.notifier.notify(value) {
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
                }
            }
        }

        if let Some(deadline) = self
            .listener_connections
            .service_state
            .static_config
            .event()
            .deadline
        {
            let msg = "The notification was sent";
            let duration_since_creation = fail!(from self, when deadline.creation_time.elapsed(),
                                with NotifierNotifyError::UnableToAcquireElapsedTime,
                                "{} but the elapsed system time could not be acquired which is required for deadline handling.",
                                msg);

            let previous_duration_since_creation = self
                .listener_connections
                .service_state
                .dynamic_storage
                .get()
                .event()
                .elapsed_time_since_last_notification
                .swap(duration_since_creation.as_nanos() as u64, Ordering::Relaxed);

            let duration_since_last_notification = Duration::from_nanos(
                duration_since_creation.as_nanos() as u64 - previous_duration_since_creation,
            );

            if deadline.value < duration_since_last_notification {
                fail!(from self, with NotifierNotifyError::MissedDeadline,
                "{} but the deadline was hit. The service requires a notification after {:?} but {:?} passed without a notification.",
                msg, deadline.value, duration_since_last_notification);
            }
        }

        Ok(number_of_triggered_listeners)
    }
}
