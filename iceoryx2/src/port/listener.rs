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
//! # fn main() -> Result<(), Box<dyn core::error::Error>> {
//! let node = NodeBuilder::new().create::<ipc::Service>()?;
//! let event = node.service_builder(&"MyEventName".try_into()?)
//!     .event()
//!     .open_or_create()?;
//!
//! let mut listener = event.listener_builder().create()?;
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
//! # fn main() -> Result<(), Box<dyn core::error::Error>> {
//! let node = NodeBuilder::new().create::<ipc::Service>()?;
//! let event = node.service_builder(&"MyEventName".try_into()?)
//!     .event()
//!     .open_or_create()?;
//!
//! let mut listener = event.listener_builder().create()?;
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
use iceoryx2_bb_posix::file_descriptor::{FileDescriptor, FileDescriptorBased};
use iceoryx2_bb_posix::file_descriptor_set::SynchronousMultiplexing;
use iceoryx2_cal::arc_sync_policy::ArcSyncPolicy;
use iceoryx2_cal::dynamic_storage::DynamicStorage;
use iceoryx2_cal::event::{ListenerBuilder, ListenerWaitError, NamedConceptMgmt, TriggerId};
use iceoryx2_cal::named_concept::{NamedConceptBuilder, NamedConceptRemoveError};

use crate::config::Config;
use crate::service::config_scheme::event_config;
use crate::service::dynamic_config::event::ListenerDetails;
use crate::service::naming_scheme::event_concept_name;
use crate::service::{NoResource, ServiceState};
use crate::{port::port_identifiers::UniqueListenerId, service};
use alloc::sync::Arc;
use core::sync::atomic::Ordering;
use core::time::Duration;

use super::event_id::EventId;

/// Defines the failures that can occur when a [`Listener`] is created with the
/// [`crate::service::port_factory::listener::PortFactoryListener`].
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum ListenerCreateError {
    /// The maximum amount of [`Listener`]s that can connect to a
    /// [`Service`](crate::service::Service) is
    /// defined in [`crate::config::Config`]. When this is exceeded no more [`Listener`]s
    /// can be created for a specific [`Service`](crate::service::Service).
    ExceedsMaxSupportedListeners,
    /// An underlying resource of the [`Service`](crate::service::Service) could not be created
    ResourceCreationFailed,
    /// Caused by a failure when instantiating a [`ArcSyncPolicy`] defined in the
    /// [`Service`](crate::service::Service) as `ArcThreadSafetyPolicy`.
    FailedToDeployThreadsafetyPolicy,
}

impl core::fmt::Display for ListenerCreateError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ListenerCreateError::{self:?}")
    }
}

impl core::error::Error for ListenerCreateError {}

/// Represents the receiving endpoint of an event based communication.
#[derive(Debug)]
pub struct Listener<Service: service::Service> {
    dynamic_listener_handle: Option<ContainerHandle>,
    listener:
        Service::ArcThreadSafetyPolicy<<Service::Event as iceoryx2_cal::event::Event>::Listener>,
    service_state: Arc<ServiceState<Service, NoResource>>,
    listener_id: UniqueListenerId,
}

unsafe impl<Service: service::Service> Send for Listener<Service> where
    Service::ArcThreadSafetyPolicy<<Service::Event as iceoryx2_cal::event::Event>::Listener>:
        Send + Sync
{
}

unsafe impl<Service: service::Service> Sync for Listener<Service> where
    Service::ArcThreadSafetyPolicy<<Service::Event as iceoryx2_cal::event::Event>::Listener>:
        Send + Sync
{
}

impl<Service: service::Service> FileDescriptorBased for Listener<Service>
where
    <Service::Event as iceoryx2_cal::event::Event>::Listener: FileDescriptorBased,
{
    fn file_descriptor(&self) -> &FileDescriptor {
        let fd = self.listener.lock().file_descriptor() as *const FileDescriptor;
        // the file descriptor and its reference never changes during the lifetime
        // of the listener port
        unsafe { &*fd }
    }
}

impl<Service: service::Service> SynchronousMultiplexing for Listener<Service> where
    <Service::Event as iceoryx2_cal::event::Event>::Listener: SynchronousMultiplexing
{
}

impl<Service: service::Service> Drop for Listener<Service> {
    fn drop(&mut self) {
        if let Some(handle) = self.dynamic_listener_handle {
            self.service_state
                .dynamic_storage
                .get()
                .event()
                .release_listener_handle(handle)
        }
    }
}

impl<Service: service::Service> Listener<Service> {
    pub(crate) fn new(
        service: Arc<ServiceState<Service, NoResource>>,
    ) -> Result<Self, ListenerCreateError> {
        let msg = "Failed to create listener";
        let origin = "Listener::new()";
        let listener_id = UniqueListenerId::new();

        let event_name = event_concept_name(&listener_id);
        let event_config = event_config::<Service>(service.shared_node.config());

        let listener = fail!(from origin,
                             when <Service::Event as iceoryx2_cal::event::Event>::ListenerBuilder::new(&event_name).config(&event_config)
                                .trigger_id_max(TriggerId::new(service.static_config.event().event_id_max_value))
                                .create(),
                             with ListenerCreateError::ResourceCreationFailed,
                             "{} since the underlying event concept \"{}\" could not be created.", msg, event_name);
        let listener = Service::ArcThreadSafetyPolicy::new(listener);
        let listener = match listener {
            Ok(v) => v,
            Err(e) => {
                fail!(from origin, with ListenerCreateError::FailedToDeployThreadsafetyPolicy,
                      "{msg} since the threadsafety policy could not be instantiated ({e:?}).");
            }
        };

        let mut new_self = Self {
            service_state: service.clone(),
            dynamic_listener_handle: None,
            listener,
            listener_id,
        };

        core::sync::atomic::compiler_fence(Ordering::SeqCst);

        // !MUST! be the last task otherwise a listener is added to the dynamic config without
        // the creation of all required channels
        let dynamic_listener_handle = match service.dynamic_storage.get().event().add_listener_id(
            ListenerDetails {
                listener_id,
                node_id: *service.shared_node.id(),
            },
        ) {
            Some(unique_index) => unique_index,
            None => {
                fail!(from origin, with ListenerCreateError::ExceedsMaxSupportedListeners,
                                 "{} since it would exceed the maximum supported amount of listeners of {}.",
                                 msg, service.static_config.event().max_listeners);
            }
        };

        new_self.dynamic_listener_handle = Some(dynamic_listener_handle);

        Ok(new_self)
    }

    /// Returns the deadline of the corresponding [`Service`](crate::service::Service).
    pub fn deadline(&self) -> Option<Duration> {
        self.service_state
            .static_config
            .event()
            .deadline
            .map(|v| v.value)
    }

    /// Non-blocking wait for new [`EventId`]s. Collects all [`EventId`]s that were received and
    /// calls the provided callback is with the [`EventId`] as input argument.
    pub fn try_wait_all<F: FnMut(EventId)>(&self, callback: F) -> Result<(), ListenerWaitError> {
        use iceoryx2_cal::event::Listener;
        fail!(from self, when self.listener.lock().try_wait_all(callback),
            "Failed to while calling try_wait on underlying event::Listener");
        Ok(())
    }

    /// Blocking wait for new [`EventId`]s until the provided timeout has passed. Unblocks as soon
    /// as an [`EventId`] was received and then collects all [`EventId`]s that were received and
    /// calls the provided callback is with the [`EventId`] as input argument.
    pub fn timed_wait_all<F: FnMut(EventId)>(
        &self,
        callback: F,
        timeout: Duration,
    ) -> Result<(), ListenerWaitError> {
        use iceoryx2_cal::event::Listener;
        fail!(from self, when self.listener.lock().timed_wait_all(callback, timeout),
            "Failed to while calling timed_wait({:?}) on underlying event::Listener", timeout);
        Ok(())
    }

    /// Blocking wait for new [`EventId`]s. Unblocks as soon
    /// as an [`EventId`] was received and then collects all [`EventId`]s that were received and
    /// calls the provided callback is with the [`EventId`] as input argument.
    pub fn blocking_wait_all<F: FnMut(EventId)>(
        &self,
        callback: F,
    ) -> Result<(), ListenerWaitError> {
        use iceoryx2_cal::event::Listener;
        fail!(from self, when self.listener.lock().blocking_wait_all(callback),
            "Failed to while calling blocking_wait on underlying event::Listener");
        Ok(())
    }

    /// Non-blocking wait for a new [`EventId`]. If no [`EventId`] was notified it returns [`None`].
    /// On error it returns [`ListenerWaitError`] is returned which describes the error
    /// in detail.
    pub fn try_wait_one(&self) -> Result<Option<EventId>, ListenerWaitError> {
        use iceoryx2_cal::event::Listener;
        Ok(fail!(from self, when self.listener.lock().try_wait_one(),
            "Failed to while calling try_wait on underlying event::Listener"))
    }

    /// Blocking wait for a new [`EventId`] until either an [`EventId`] was received or the timeout
    /// has passed. If no [`EventId`] was notified it returns [`None`].
    /// On error it returns [`ListenerWaitError`] is returned which describes the error
    /// in detail.
    pub fn timed_wait_one(&self, timeout: Duration) -> Result<Option<EventId>, ListenerWaitError> {
        use iceoryx2_cal::event::Listener;
        Ok(
            fail!(from self, when self.listener.lock().timed_wait_one(timeout),
            "Failed to while calling timed_wait({:?}) on underlying event::Listener", timeout),
        )
    }

    /// Blocking wait for a new [`EventId`].
    /// Sporadic wakeups can occur and if no [`EventId`] was notified it returns [`None`].
    /// On error it returns [`ListenerWaitError`] is returned which describes the error
    /// in detail.
    pub fn blocking_wait_one(&self) -> Result<Option<EventId>, ListenerWaitError> {
        use iceoryx2_cal::event::Listener;
        Ok(
            fail!(from self, when self.listener.lock().blocking_wait_one(),
            "Failed to while calling blocking_wait on underlying event::Listener"),
        )
    }

    /// Returns the [`UniqueListenerId`] of the [`Listener`]
    pub fn id(&self) -> UniqueListenerId {
        self.listener_id
    }
}

pub(crate) unsafe fn remove_connection_of_listener<Service: service::Service>(
    listener_id: &UniqueListenerId,
    config: &Config,
) -> Result<(), NamedConceptRemoveError> {
    let origin = format!(
        "remove_connection_of_listener::<{}>({:?})",
        core::any::type_name::<Service>(),
        listener_id
    );
    let msg = "Unable to remove the listener connection";
    let event_name = event_concept_name(listener_id);
    let event_config = event_config::<Service>(config);

    fail!(from origin,
            when <Service::Event as NamedConceptMgmt>::remove_cfg(&event_name, &event_config),
            "{} since the underlying concept could not be removed.", msg);
    Ok(())
}
