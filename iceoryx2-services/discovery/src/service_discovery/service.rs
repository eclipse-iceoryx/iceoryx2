// Copyright (c) 2025 Contributors to the Eclipse Foundation
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

use crate::service_discovery::{SyncError, Tracker};
use iceoryx2::prelude::ZeroCopySend;
use iceoryx2::{
    config::Config as IceoryxConfig,
    node::{Node, NodeBuilder, NodeCreationFailure},
    port::{
        notifier::{Notifier, NotifierCreateError, NotifierNotifyError},
        publisher::{Publisher, PublisherCreateError},
        LoanError, SendError,
    },
    prelude::ServiceName,
    service::{
        builder::{
            event::EventOpenOrCreateError, publish_subscribe::PublishSubscribeOpenOrCreateError,
        },
        static_config::StaticConfig,
        Service as ServiceType, ServiceDetails,
    },
};

use once_cell::sync::Lazy;

const SERVICE_NAME: &str = "discovery/services/";

/// Events emitted by the service discovery service.
#[derive(Debug, ZeroCopySend, serde::Serialize, serde::Deserialize)]
#[allow(dead_code)] // Fields used by subscribers
#[repr(C)]
pub enum Discovery {
    /// A service has been added to the system.
    ///
    /// Contains the static configuration of the newly added service.
    Added(StaticConfig),

    /// A service has been removed from the system.
    ///
    /// Contains the static configuration of the removed service.
    Removed(StaticConfig),
}

/// The payload type used for publishing discovery changes
pub type Payload = Discovery;

/// Errors that can occur when creating the service discovery service.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum CreationError {
    /// The caller does not have sufficient permissions to create the service.
    InsufficientPermissions,

    /// Failed to create the underlying node.
    NodeCreationFailure,

    /// Failed to create the service.
    ServiceCreationFailure,

    /// Failed to sync services.
    SyncFailure,

    /// Failed to create the publisher for reasons other than it already existing.
    PublisherCreationError,

    /// A publisher to the service already exists.
    PublisherAlreadyExists,

    /// A notifier to the service already exists.
    NotifierAlreadyExists,
}

impl core::fmt::Display for CreationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        std::write!(f, "CreationError::{:?}", self)
    }
}

impl core::error::Error for CreationError {}

impl From<NodeCreationFailure> for CreationError {
    fn from(_: NodeCreationFailure) -> Self {
        CreationError::NodeCreationFailure
    }
}

impl From<PublishSubscribeOpenOrCreateError> for CreationError {
    fn from(_: PublishSubscribeOpenOrCreateError) -> Self {
        CreationError::ServiceCreationFailure
    }
}

impl From<PublisherCreateError> for CreationError {
    fn from(error: PublisherCreateError) -> Self {
        match error {
            PublisherCreateError::ExceedsMaxSupportedPublishers => {
                CreationError::PublisherAlreadyExists
            }
            PublisherCreateError::UnableToCreateDataSegment => {
                CreationError::PublisherCreationError
            }
        }
    }
}

impl From<EventOpenOrCreateError> for CreationError {
    fn from(_: EventOpenOrCreateError) -> Self {
        CreationError::ServiceCreationFailure
    }
}

impl From<NotifierCreateError> for CreationError {
    fn from(_: NotifierCreateError) -> Self {
        CreationError::NotifierAlreadyExists
    }
}

impl From<SyncError> for CreationError {
    fn from(error: SyncError) -> Self {
        match error {
            SyncError::InsufficientPermissions => CreationError::InsufficientPermissions,
            SyncError::ServiceLookupFailure => CreationError::SyncFailure,
        }
    }
}

/// Errors that can occur during the spin operation of the service discovery service.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum SpinError {
    /// The caller does not have sufficient permissions to execute the service.
    InsufficientPermissions,

    /// Failed to sync services with the iceoryx2 system.
    SyncFailure,

    /// Failed to publish a discovery event.
    PublishFailure,

    /// Failed to send a notification about service changes.
    NotifyFailure,
}

impl core::fmt::Display for SpinError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        std::write!(f, "SpinError::{:?}", self)
    }
}

impl core::error::Error for SpinError {}

impl From<SyncError> for SpinError {
    fn from(error: SyncError) -> Self {
        match error {
            SyncError::InsufficientPermissions => SpinError::InsufficientPermissions,
            SyncError::ServiceLookupFailure => SpinError::SyncFailure,
        }
    }
}

impl From<LoanError> for SpinError {
    fn from(_: LoanError) -> Self {
        SpinError::PublishFailure
    }
}

impl From<SendError> for SpinError {
    fn from(_: SendError) -> Self {
        SpinError::PublishFailure
    }
}

impl From<NotifierNotifyError> for SpinError {
    fn from(_: NotifierNotifyError) -> Self {
        SpinError::NotifyFailure
    }
}

/// Configuration for the service discovery service.
#[derive(Debug, Clone)]
pub struct Config {
    /// Whether or not to synchronize the discvery state on initialization.
    ///
    /// If enabled, updates for all pre-existing services will not be sent.
    pub sync_on_initialization: bool,

    /// Whether to include iceoryx-internal services in discovery results.
    pub include_internal: bool,

    /// Whether to publish discovery events.
    pub publish_events: bool,

    /// The maximum number of subscribers to the service permitted.
    pub max_subscribers: usize,

    /// The maximum number of samples the subscriber retains in its buffer.
    pub max_buffer_size: usize,

    /// The maximum number of samples subscribers are permitted to hold loans for.
    pub max_borrrowed_samples: usize,

    /// The number of older samples the subscriber can request from the service when starting.
    pub history_size: usize,

    /// Whether to send notifications on changes.
    pub send_notifications: bool,

    /// The maximum number of listeners to the service permitted.
    pub max_listeners: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            sync_on_initialization: true,
            include_internal: true,
            publish_events: true,
            max_subscribers: 10,
            max_buffer_size: 10,
            history_size: 10,
            max_borrrowed_samples: 10,
            send_notifications: true,
            max_listeners: 10,
        }
    }
}

/// The service discovery service.
///
/// This service is responsible for tracking and publishing information about
/// services in the system. It can detect when services are added or removed,
/// and notify interested parties about these changes.
///
/// # Type Parameters
///
/// * `S` - The service type that this discovery service operates on.
pub struct Service<S: ServiceType> {
    discovery_config: Config,
    iceoryx_config: IceoryxConfig,
    _node: Node<S>,
    publisher: Option<Publisher<S, Payload, ()>>,
    notifier: Option<Notifier<S>>,
    tracker: Tracker<S>,
}

impl<S: ServiceType> Service<S> {
    /// Creates the service discovery service.
    ///
    /// This function initializes a new service discovery service with the provided
    /// configuration. The service will track services of type `S` in the system.
    ///
    /// # Parameters
    ///
    /// * `discovery_config` - Configuration for the discovery service.
    /// * `iceoryx_config` - Configuration for the underlying iceoryx system.
    ///
    /// # Returns
    ///
    /// A result containing either the created service or an error if creation failed.
    pub fn create(
        discovery_config: &Config,
        iceoryx_config: &IceoryxConfig,
    ) -> Result<Self, CreationError> {
        let node = NodeBuilder::new().config(iceoryx_config).create::<S>()?;

        let mut publisher = None;
        if discovery_config.publish_events {
            let publish_subscribe = node
                .service_builder(service_name())
                .publish_subscribe::<Payload>()
                .subscriber_max_buffer_size(discovery_config.max_buffer_size)
                .subscriber_max_borrowed_samples(discovery_config.max_borrrowed_samples)
                .history_size(discovery_config.history_size)
                .max_subscribers(discovery_config.max_subscribers)
                .max_publishers(1)
                .open_or_create()?;

            publisher = Some(publish_subscribe.publisher_builder().create()?);
        }

        let mut notifier = None;
        if discovery_config.send_notifications {
            let event = node
                .service_builder(service_name())
                .event()
                .max_listeners(discovery_config.max_listeners)
                .max_notifiers(1)
                .open_or_create()?;

            let port = event.notifier_builder().create()?;
            notifier = Some(port);
        }

        let mut tracker = Tracker::<S>::new();

        if discovery_config.sync_on_initialization {
            tracker.sync(iceoryx_config)?;
        }

        Ok(Service::<S> {
            discovery_config: discovery_config.clone(),
            iceoryx_config: iceoryx_config.clone(),
            _node: node,
            publisher,
            notifier,
            tracker,
        })
    }

    /// Processes service changes and emits events/notifications.
    ///
    /// This function should be called periodically to detect changes in the service
    /// landscape and emit appropriate events and notifications. When services are
    /// added or removed, the provided callback functions are invoked.
    ///
    /// # Parameters
    ///
    /// * `on_added` - Callback function that is called for each service that was added
    /// * `on_removed` - Callback function that is called for each service that was removed
    ///
    /// # Returns
    ///
    /// A result containing `()` if successful.
    ///
    /// # Errors
    ///
    /// Returns a `SpinError` if there was an error publishing events or sending
    /// notifications.
    pub fn spin<
        FAddedService: FnMut(&ServiceDetails<S>),
        FRemovedService: FnMut(&ServiceDetails<S>),
    >(
        &mut self,
        mut on_added: FAddedService,
        mut on_removed: FRemovedService,
    ) -> Result<(), SpinError> {
        // Detect changes
        let (added_ids, removed_services) = self.tracker.sync(&self.iceoryx_config)?;
        let changes_detected = !added_ids.is_empty() || !removed_services.is_empty();

        // Publish
        for id in &added_ids {
            if let Some(service) = self.tracker.get(id) {
                if !self.discovery_config.include_internal
                    && ServiceName::has_iox2_prefix(service.static_details.name())
                {
                    continue;
                }
                if let Some(publisher) = &self.publisher {
                    let sample = publisher.loan_uninit()?;
                    let sample =
                        sample.write_payload(Discovery::Added(service.static_details.clone()));
                    sample.send()?;
                }
                on_added(service);
            }
        }

        for service in &removed_services {
            if !self.discovery_config.include_internal
                && ServiceName::has_iox2_prefix(service.static_details.name())
            {
                continue;
            }
            if let Some(publisher) = &self.publisher {
                let sample = publisher.loan_uninit()?;
                let sample =
                    sample.write_payload(Discovery::Removed(service.static_details.clone()));
                sample.send()?;
            }
            on_removed(service);
        }

        // Notify
        if let Some(notifier) = &mut self.notifier {
            if changes_detected {
                notifier.notify()?;
            }
        }

        Ok(())
    }
}

/// Returns the service name used by the service discovery service.
///
/// This function returns a reference to a lazily initialized static `ServiceName`
/// instance. The service name is constructed by concatenating the internal service
/// prefix with the discovery service name.
///
/// # Returns
///
/// A reference to the static `ServiceName` instance used for the discovery service.
///
/// # Panics
///
/// This function will panic during the first call if the service name is invalid,
/// which should never happen with the predefined constants.
pub fn service_name() -> &'static ServiceName {
    static SERVICE_NAME_INSTANCE: Lazy<ServiceName> = Lazy::new(|| {
        ServiceName::__internal_new_prefixed(SERVICE_NAME)
            .expect("shouldn't occur: invalid service name for service discovery service")
    });

    &SERVICE_NAME_INSTANCE
}
