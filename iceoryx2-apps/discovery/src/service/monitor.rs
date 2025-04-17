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
    /// Custom service name for the monitor (defaults to "iox2://monitor/services")
    pub service_name: Option<ServiceName>,
    /// Whether to publish discovery events
    pub publish_events: bool,
    /// Whether to send notifications on changes
    pub send_notifications: bool,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            service_name: None,
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
    pub fn new(config: MonitorConfig) -> Self {
        let node = NodeBuilder::new()
            .config(IceoryxConfig::global_config())
            .create::<S>()
            .expect("failed to create monitor node");

        let service_name = config.service_name.unwrap_or_else(|| {
            ServiceName::new("iox2://monitor/services")
                .expect("failed to create monitor service name")
        });

        let mut publisher = None;
        if config.publish_events {
            let publish_subscribe = node
                .service_builder(&service_name)
                .publish_subscribe::<DiscoveryEvent>()
                .create()
                .expect("failed to create publish-subscribe service");
            let port = publish_subscribe
                .publisher_builder()
                .create()
                .expect("failed to create publisher");
            publisher = Some(port);
        }

        let mut notifier = None;
        if config.send_notifications {
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
            _node: node,
            publisher,
            notifier,
            tracker,
        }
    }

    /// Performs a single iteration of the service monitoring process.
    ///
    /// This method:
    /// 1. Detects added and removed services by syncing with the global configuration
    /// 2. Publishes the added and removed services
    /// 3. Sends a notification if any changes were detected
    pub fn spin(&mut self) {
        // Detect changes
        let (added, removed) = self.tracker.sync(IceoryxConfig::global_config());
        let changes_detected = !added.is_empty() || !removed.is_empty();

        // Publish
        for id in added {
            if let Some(service) = self.tracker.get(&id) {
                info!(
                    "ADDED {} {}",
                    service.static_details.messaging_pattern(),
                    service.static_details.name()
                );

                if let Some(publisher) = &mut self.publisher {
                    // Clone required since the details are stored in the tracker.
                    let event = DiscoveryEvent::Added(service.static_details.clone());
                    let _ = publisher.send_copy(event);
                }
            }
        }
        for service in removed {
            info!(
                "REMOVED {} {}",
                service.static_details.messaging_pattern(),
                service.static_details.name()
            );

            if let Some(publisher) = &mut self.publisher {
                // The removed details are not stored in the tracker. Claim ownership.
                let event = DiscoveryEvent::Removed(service.static_details);
                let _ = publisher.send_copy(event);
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
