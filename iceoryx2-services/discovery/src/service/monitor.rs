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
    service::{static_config::StaticConfig, Service},
};
use iceoryx2_bb_log::info;
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
    /// Whether to send notifications on changes
    pub send_notifications: bool,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            service_name: INTERNAL_SERVICE_PREFIX.to_owned() + SERVICE_DISCOVERY_SERVICE_NAME,
            include_internal: true,
            publish_events: true,
            send_notifications: true,
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
    pub fn new(monitor_config: &MonitorConfig, iceoryx_config: &IceoryxConfig) -> Self {
        let node = NodeBuilder::new()
            .config(iceoryx_config)
            .create::<S>()
            .expect("failed to create monitor node");

        let service_name = ServiceName::new(monitor_config.service_name.as_str())
            .expect("failed to create monitor service name");

        let mut publisher = None;
        if monitor_config.publish_events {
            let publish_subscribe = node
                .service_builder(&service_name)
                .publish_subscribe::<DiscoveryEvent>()
                // TODO: Work out how to handle pub-sub config ...
                .subscriber_max_borrowed_samples(10)
                .history_size(1)
                .subscriber_max_buffer_size(10)
                .create()
                .expect("failed to create publish-subscribe service");

            publisher = Some(
                publish_subscribe
                    .publisher_builder()
                    .create()
                    .expect("failed to create publisher"),
            );
        }

        let mut notifier = None;
        if monitor_config.send_notifications {
            let event = node
                .service_builder(&service_name)
                .event()
                .create()
                .expect("failed to create event service");
            let port = event
                .notifier_builder()
                .create()
                .expect("failed to create notifier");
            notifier = Some(port);
        }

        let tracker = Tracker::<S>::new();

        Monitor::<S> {
            monitor_config: monitor_config.clone(),
            iceoryx_config: iceoryx_config.clone(),
            _node: node,
            publisher,
            notifier,
            tracker,
        }
    }

    /// Performs a single iteration of the service monitoring process.
    ///
    /// This method checks for added or removed services, publishes discovery events
    /// for these changes, and sends notifications when changes are detected.
    pub fn spin(&mut self) {
        // Detect changes
        let (added, removed) = self.tracker.sync(&self.iceoryx_config);
        let changes_detected = !added.is_empty() || !removed.is_empty();

        // Publish
        for id in added {
            if let Some(service) = self.tracker.get(&id) {
                if !self.monitor_config.include_internal
                    && is_internal_service(service.static_details.name())
                {
                    continue;
                }
                info!(
                    "ADDED {} {}",
                    service.static_details.messaging_pattern(),
                    service.static_details.name()
                );

                if let Some(publisher) = &mut self.publisher {
                    // Clone required since the details are stored in the tracker.
                    let sample = publisher.loan_uninit().unwrap();
                    let sample =
                        sample.write_payload(DiscoveryEvent::Added(service.static_details.clone()));
                    let _ = sample.send();
                }
            }
        }
        for service in removed {
            if !self.monitor_config.include_internal
                && is_internal_service(service.static_details.name())
            {
                continue;
            }
            info!(
                "REMOVED {} {}",
                service.static_details.messaging_pattern(),
                service.static_details.name()
            );

            if let Some(publisher) = &mut self.publisher {
                // The removed details are not stored in the tracker. Claim ownership.
                let sample = publisher.loan_uninit().unwrap();
                let sample = sample.write_payload(DiscoveryEvent::Removed(service.static_details));
                let _ = sample.send();
            }
        }

        // Notify
        if let Some(notifier) = &mut self.notifier {
            if changes_detected {
                let _ = notifier.notify();
            }
        }
    }
}
