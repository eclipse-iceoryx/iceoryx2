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

use crate::service_discovery::Tracker;
use iceoryx2::{
    config::Config as IceoryxConfig,
    node::{Node, NodeBuilder},
    port::{notifier::Notifier, publisher::Publisher},
    prelude::ServiceName,
    service::Service as ServiceType,
    service::{static_config::StaticConfig, ServiceDetails},
};
use iceoryx2_services_common::{is_internal_service, INTERNAL_SERVICE_PREFIX};

use once_cell::sync::Lazy;

const SERVICE_DISCOVERY_SERVICE_NAME: &str = "discovery/services/";

/// Events emitted by the service discovery service.
#[derive(Debug)]
#[allow(dead_code)] // Fields used by subscribers
pub enum DiscoveryEvent {
    /// A service has been added to the system.
    ///
    /// Contains the static configuration of the newly added service.
    Added(StaticConfig),

    /// A service has been removed from the system.
    ///
    /// Contains the static configuration of the removed service.
    Removed(StaticConfig),
}

/// Errors that can occur when creating the service discovery service.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum CreationError {
    /// Failed to create the underlying node.
    NodeCreationFailure,

    /// Failed to create the service for reasons other than it already existing.
    ServiceCreationFailure,

    /// The service already exists in the system.
    ServiceAlreadyExists,

    /// Failed to create the publisher for reasons other than it already existing.
    PublisherCreationError,

    /// A publisher to the service already exists.
    PublisherAlreadyExists,

    /// A notifier to the service already exists.
    NotifierAlreadyExists,
}

/// Errors that can occur during the spin operation of the service discovery service.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum SpinError {
    /// Failed to publish a discovery event.
    PublishFailure,

    /// Failed to send a notification about service changes.
    NotifyFailure,
}

/// Configuration for the service discovery service.
#[derive(Debug, Clone)]
pub struct DiscoveryConfig {
    /// Whether to include iceoryx-internal services in discovery results.
    pub include_internal: bool,

    /// Whether to publish discovery events.
    pub publish_events: bool,

    /// The maximum number of subscribers to the service permitted.
    pub max_subscribers: usize,

    /// Whether to send notifications on changes.
    pub send_notifications: bool,

    /// The maximum number of listeners to the service permitted.
    pub max_listeners: usize,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            include_internal: true,
            publish_events: true,
            max_subscribers: 10,
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
    discovery_config: DiscoveryConfig,
    iceoryx_config: IceoryxConfig,
    _node: Node<S>,
    publisher: Option<Publisher<S, DiscoveryEvent, ()>>,
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
    ///
    /// # Errors
    ///
    pub fn create(
        discovery_config: &DiscoveryConfig,
        iceoryx_config: &IceoryxConfig,
    ) -> Result<Self, CreationError> {
        let node = NodeBuilder::new()
            .config(iceoryx_config)
            .create::<S>()
            .map_err(|_| CreationError::NodeCreationFailure)?;

        let mut publisher = None;
        if discovery_config.publish_events {
            let publish_subscribe = node
                .service_builder(service_name())
                .publish_subscribe::<DiscoveryEvent>()
                .subscriber_max_borrowed_samples(10)
                .history_size(10)
                .subscriber_max_buffer_size(10)
                .max_publishers(1)
                .max_subscribers(discovery_config.max_subscribers)
                .create()
                .map_err(|e| {
                    match e {
                        iceoryx2::service::builder::publish_subscribe::PublishSubscribeCreateError::AlreadyExists => CreationError::ServiceAlreadyExists,
                        iceoryx2::service::builder::publish_subscribe::PublishSubscribeCreateError::IsBeingCreatedByAnotherInstance => CreationError::ServiceAlreadyExists,
                        _ => CreationError::ServiceCreationFailure,
                    }
                })?;

            publisher = Some(
                publish_subscribe
                    .publisher_builder()
                    .create()
                    .map_err(|e| {match e {
                        iceoryx2::port::publisher::PublisherCreateError::ExceedsMaxSupportedPublishers => CreationError::PublisherAlreadyExists,
                        iceoryx2::port::publisher::PublisherCreateError::UnableToCreateDataSegment => CreationError::PublisherCreationError,
                    }})?
            );
        }

        let mut notifier = None;
        if discovery_config.send_notifications {
            let event = node
                .service_builder(service_name())
                .event()
                .max_notifiers(1)
                .max_listeners(discovery_config.max_listeners)
                .create()
                .map_err(|e| {
                    match e {
                        iceoryx2::service::builder::event::EventCreateError::IsBeingCreatedByAnotherInstance => CreationError::ServiceAlreadyExists,
                        iceoryx2::service::builder::event::EventCreateError::AlreadyExists => CreationError::ServiceAlreadyExists,
                        _ => CreationError::ServiceCreationFailure,
                    }
                })?;

            let port = event.notifier_builder().create().map_err(|e| match e {
                iceoryx2::port::notifier::NotifierCreateError::ExceedsMaxSupportedNotifiers => {
                    CreationError::NotifierAlreadyExists
                }
            })?;
            notifier = Some(port);
        }

        let tracker = Tracker::<S>::new();

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
    /// landscape and emit appropriate events and notifications.
    ///
    /// # Returns
    ///
    /// A result containing a tuple of:
    /// - A vector of references to services that were added since the last call
    /// - A vector of services that were removed since the last call
    ///
    /// # Errors
    ///
    /// Returns a `SpinError` if there was an error publishing events or sending
    /// notifications.
    pub fn spin(&mut self) -> Result<(Vec<&ServiceDetails<S>>, Vec<ServiceDetails<S>>), SpinError> {
        // Detect changes
        let (added_ids, removed_services) = self.tracker.sync(&self.iceoryx_config);
        let changes_detected = !added_ids.is_empty() || !removed_services.is_empty();

        let mut added_services = Vec::new();
        for id in &added_ids {
            if let Some(service) = self.tracker.get(id) {
                if !self.discovery_config.include_internal
                    && is_internal_service(service.static_details.name())
                {
                    continue;
                }

                if let Some(publisher) = &mut self.publisher {
                    // Clone required since the details are stored in the tracker.
                    let sample = publisher.loan_uninit().unwrap();
                    let sample =
                        sample.write_payload(DiscoveryEvent::Added(service.static_details.clone()));
                    sample.send().map_err(|e| match e {
                        _ => SpinError::PublishFailure,
                    })?;
                }

                // Collect references to added services (owned by the tracker)
                added_services.push(service);
            }
        }

        for service in &removed_services {
            if !self.discovery_config.include_internal
                && is_internal_service(service.static_details.name())
            {
                continue;
            }

            if let Some(publisher) = &mut self.publisher {
                // The removed details are not stored in the tracker. Claim ownership.
                let sample = publisher.loan_uninit().unwrap();
                let sample =
                    sample.write_payload(DiscoveryEvent::Removed(service.static_details.clone()));
                sample.send().map_err(|e| match e {
                    _ => SpinError::NotifyFailure,
                })?;
            }
        }

        // Notify
        if let Some(notifier) = &mut self.notifier {
            if changes_detected {
                let _ = notifier.notify();
            }
        }

        Ok((added_services, removed_services))
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
///
/// # Examples
///
pub fn service_name() -> &'static ServiceName {
    static SERVICE_NAME_INSTANCE: Lazy<ServiceName> = Lazy::new(|| {
        ServiceName::new(&(INTERNAL_SERVICE_PREFIX.to_owned() + SERVICE_DISCOVERY_SERVICE_NAME))
            .expect("service name is valid")
    });

    &SERVICE_NAME_INSTANCE
}
