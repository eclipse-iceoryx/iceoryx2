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

use iceoryx2::{
    config::Config,
    node::{Node, NodeBuilder},
    port::{notifier::Notifier, publisher::Publisher},
    prelude::ServiceName,
    service::{static_config::StaticConfig, Service},
    tracker::service::Tracker,
};
use iceoryx2_bb_log::info;
use iceoryx2_cli::output::ServiceDescriptor;

#[derive(Debug)]
#[allow(dead_code)] // Fields used by subscribers
enum DiscoveryEvent {
    Added(StaticConfig),
    Removed(StaticConfig),
}

pub(crate) struct Monitor<S: Service> {
    #[allow(dead_code)]
    node: Node<S>, // Kept to maintain ownership of the node
    publisher: Publisher<S, DiscoveryEvent, ()>,
    notifier: Notifier<S>,
    tracker: Tracker<S>,
}

impl<S: Service> Monitor<S> {
    /// Creates a new service monitor.
    ///
    /// # Returns
    ///
    /// A new `Monitor` instance ready to track service changes
    pub fn new() -> Self {
        let node = NodeBuilder::new()
            .config(Config::global_config())
            .create::<S>()
            .expect("failed to create monitor node");

        let service_name = ServiceName::new("iox2://monitor/services")
            .expect("failed to create monitor service name");

        let publish_subscribe = node
            .service_builder(&service_name)
            .publish_subscribe::<DiscoveryEvent>()
            .create()
            .expect("failed to create publish-subscribe service");
        let publisher = publish_subscribe
            .publisher_builder()
            .create()
            .expect("failed to create publisher");

        let event = node
            .service_builder(&service_name)
            .event()
            .create()
            .expect("failed to create event service");
        let notifier = event
            .notifier_builder()
            .create()
            .expect("failed to create notifier");

        let tracker = Tracker::<S>::new();

        Monitor::<S> {
            node,
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
        let (added, removed) = self.tracker.sync(Config::global_config());
        let changes_detected = !added.is_empty() || !removed.is_empty();

        // Publish
        for id in added {
            if let Some(service) = self.tracker.get(&id) {
                info!("ADDED {:?}", ServiceDescriptor::from(service));
                // Clone required since the details are stored in the tracker.
                let event = DiscoveryEvent::Added(service.static_details.clone());
                let _ = self.publisher.send_copy(event);
            }
        }
        for service in removed {
            info!("REMOVED {:?}", ServiceDescriptor::from(&service));
            // The removed details are not stored in the tracker. Claim ownership.
            let event = DiscoveryEvent::Removed(service.static_details);
            let _ = self.publisher.send_copy(event);
        }

        // Notify
        if changes_detected {
            let _ = self.notifier.notify();
        }
    }
}
