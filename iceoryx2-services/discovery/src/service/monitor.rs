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

use crate::service::Tracker;
use iceoryx2::{
    config::Config as IceoryxConfig,
    node::{Node, NodeBuilder},
    port::{notifier::Notifier, publisher::Publisher},
    prelude::ServiceName,
    service::{static_config::StaticConfig, Service, ServiceDetails},
};
use iceoryx2_services_common::{is_internal_service, INTERNAL_SERVICE_PREFIX};

const SERVICE_DISCOVERY_SERVICE_NAME: &str = "discovery/services/";

/// Events emitted by the service discovery monitor
///
/// These events are published when services are added to or removed from the system.
#[derive(Debug)]
#[allow(dead_code)] // Fields used by subscribers
pub enum DiscoveryEvent {
    /// A service has been added to the system
    Added(StaticConfig),
    /// A service has been removed from the system
    Removed(StaticConfig),
}

/// Configuration options for the service monitor.
///
/// This struct provides a more self-documenting way to configure
/// the behavior of the `Monitor`.
#[derive(Debug, Clone)]
pub struct MonitorConfig {
    /// Custom service name for the monitor
    pub service_name: String,
    /// Whether to ignore iceoryx-internal services
    pub include_internal: bool,
    /// Whether to publish discovery events
    pub publish_events: bool,
    /// The maximum number of subscribers to the service permitted
    pub max_subscribers: usize,
    /// Whether to send notifications on changes
    pub send_notifications: bool,
    /// The maximum number of listeners to the service permitted
    pub max_listeners: usize,
}

/// Errors that can occur when creating a service monitor.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum CreationError {
    /// Failed to create the underlying node
    NodeCreationFailure,
    /// The provided service name is invalid
    InvalidServiceName,
    /// Failed to create the service for reasons other than it already existing
    ServiceCreationFailure,
    /// The service already exists in the system
    ServiceAlreadyExists,
    /// Failed to create the publisher for reasons other than it already existing
    PublisherCreationError,
    /// A publisher to the service already exists
    PublisherAlreadyExists,
    /// A notifier to the service already exists
    NotifierAlreadyExists,
}

/// Errors that can occur during the `spin` operation of the service monitor.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum SpinError {
    /// Failed to publish a discovery event
    PublishFailure,
    /// Failed to send a notification about service changes
    NotifyFailure,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            service_name: INTERNAL_SERVICE_PREFIX.to_owned() + SERVICE_DISCOVERY_SERVICE_NAME,
            include_internal: true,
            publish_events: true,
            max_subscribers: 10,
            send_notifications: true,
            max_listeners: 10,
        }
    }
}

/// A service monitor that tracks and publishes service discovery events.
///
/// The monitor detects when services are added or removed from the system and
/// publishes these events to subscribers. It also sends notifications when
/// changes occur.
///
/// # Type Parameters
///
/// * `S` - The service implementation type that provides communication capabilities
pub struct Monitor<S: Service> {
    monitor_config: MonitorConfig,
    iceoryx_config: IceoryxConfig,
    _node: Node<S>,
    publisher: Option<Publisher<S, DiscoveryEvent, ()>>,
    notifier: Option<Notifier<S>>,
    tracker: Tracker<S>,
}

impl<S: Service> Monitor<S> {
    /// Creates a new service monitor.
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration options for the monitor
    ///
    /// # Returns
    ///
    /// A new `Monitor` instance ready to track service changes
    pub fn create(
        monitor_config: &MonitorConfig,
        iceoryx_config: &IceoryxConfig,
    ) -> Result<Self, CreationError> {
        let node = NodeBuilder::new()
            .config(iceoryx_config)
            .create::<S>()
            .map_err(|_| CreationError::NodeCreationFailure)?;

        let service_name = ServiceName::new(monitor_config.service_name.as_str())
            .map_err(|_| CreationError::InvalidServiceName)?;

        let mut publisher = None;
        if monitor_config.publish_events {
            let publish_subscribe = node
                .service_builder(&service_name)
                .publish_subscribe::<DiscoveryEvent>()
                .subscriber_max_borrowed_samples(10)
                .history_size(10)
                .subscriber_max_buffer_size(10)
                .max_publishers(1)
                .max_subscribers(monitor_config.max_subscribers)
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
        if monitor_config.send_notifications {
            let event = node
                .service_builder(&service_name)
                .event()
                .max_notifiers(1)
                .max_listeners(monitor_config.max_listeners)
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

        Ok(Monitor::<S> {
            monitor_config: monitor_config.clone(),
            iceoryx_config: iceoryx_config.clone(),
            _node: node,
            publisher,
            notifier,
            tracker,
        })
    }

    /// Performs a single iteration of the service monitoring process.
    ///
    /// This method checks for added or removed services, publishes discovery events
    /// for these changes, and sends notifications when changes are detected.
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// - A vector of references to added service details stored in the tracker
    /// - A vector of removed service details (owned values)
    pub fn spin(&mut self) -> Result<(Vec<&ServiceDetails<S>>, Vec<ServiceDetails<S>>), SpinError> {
        // Detect changes
        let (added_ids, removed_services) = self.tracker.sync(&self.iceoryx_config);
        let changes_detected = !added_ids.is_empty() || !removed_services.is_empty();

        let mut added_services = Vec::new();
        for id in &added_ids {
            if let Some(service) = self.tracker.get(id) {
                if !self.monitor_config.include_internal
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
            if !self.monitor_config.include_internal
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
