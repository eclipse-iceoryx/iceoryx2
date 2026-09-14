// Copyright (c) 2026 Contributors to the Eclipse Foundation
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

use iceoryx2::node::Node;
use iceoryx2::port::event_id::EventId;
use iceoryx2::port::listener::Listener;
use iceoryx2::port::notifier::{Notifier, NotifierNotifyError};
use iceoryx2::service::Service;
use iceoryx2::service::builder::event;
use iceoryx2::service::service_name::ServiceName;
use iceoryx2_link_backend::service_description::EventSettings;
use iceoryx2_log::{fail, origin, warn};

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum CreationError {
    Service,
    Notifier,
    Listener,
}

impl core::fmt::Display for CreationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "CreationError::{self:?}")
    }
}

impl core::error::Error for CreationError {}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum SendError {
    Ingestion,
    Delivery,
}

impl core::fmt::Display for SendError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "SendError::{self:?}")
    }
}

impl core::error::Error for SendError {}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum ReceiveError {
    Receive,
    Propagation,
}

impl core::fmt::Display for ReceiveError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ReceiveError::{self:?}")
    }
}

impl core::error::Error for ReceiveError {}

pub(crate) struct EventPorts<S: Service> {
    name: ServiceName,
    notifier: Notifier<S>,
    listener: Listener<S>,
    /// Whether an id beyond the service's ceiling was warned about.
    warned_out_of_bounds: bool,
}

impl<S: Service> EventPorts<S> {
    pub(crate) fn open(
        node: &Node<S>,
        name: &ServiceName,
        settings: &EventSettings,
    ) -> Result<Self, CreationError> {
        let origin = origin!("EventPorts::open");
        let service = fail!(
            from origin,
            when apply_settings(node.service_builder(name).event(), settings).open_or_create(),
            with CreationError::Service,
            "Failed to open or create {}", name
        );
        let notifier = fail!(
            from origin,
            when service.notifier_builder().create(),
            with CreationError::Notifier,
            "Failed to create the notifier of {}", name
        );
        let listener = fail!(
            from origin,
            when service.listener_builder().create(),
            with CreationError::Listener,
            "Failed to create the listener of {}", name
        );
        Ok(Self {
            name: *name,
            notifier,
            listener,
            warned_out_of_bounds: false,
        })
    }

    /// Hands every pending notification to `propagate`. The link's own
    /// notifications never reach its listener, see [`Self::send`].
    ///
    /// Returns the number of notifications propagated.
    pub(crate) fn receive<E>(
        &mut self,
        mut propagate: impl FnMut(EventId) -> Result<(), E>,
    ) -> Result<u64, ReceiveError> {
        let origin = origin!("EventPorts::receive");
        let mut received = 0;
        let mut failed = false;
        fail!(
            from origin,
            when self.listener.try_wait(|activation| {
                if failed {
                    return;
                }

                match propagate(activation.id) {
                    Ok(()) => received += 1,
                    Err(_) => failed = true,
                }
            }),
            with ReceiveError::Receive,
            "Failed to wait on the listener of {}", self.name
        );
        if failed {
            fail!(
                from origin,
                with ReceiveError::Propagation,
                "Failed to propagate a notification of {}", self.name
            );
        }
        Ok(received)
    }

    /// Notifies every listener of the service but the link's own with
    /// each id `ingest` hands over, until it has nothing more. An id the
    /// service cannot hold is dropped, warned about once.
    ///
    /// Returns the number of notifications sent.
    pub(crate) fn send<E>(
        &mut self,
        mut ingest: impl FnMut() -> Result<Option<EventId>, E>,
    ) -> Result<u64, SendError> {
        let origin = origin!("EventPorts::send");
        let mut sent = 0;
        loop {
            let id = fail!(
                from origin,
                when ingest(),
                with SendError::Ingestion,
                "Failed to ingest a notification for {}", self.name
            );
            let Some(id) = id else {
                break;
            };

            // The link's listener would otherwise receive what the link
            // notified, and propagate it back to where it came from.
            match self.notifier.__internal_notify(id, true) {
                Ok(_) => sent += 1,
                Err(NotifierNotifyError::EventIdOutOfBounds) => {
                    if !self.warned_out_of_bounds {
                        self.warned_out_of_bounds = true;
                        warn!(
                            from origin,
                            "Dropped a notification of {} whose id {:?} exceeds the local ceiling",
                            self.name, id
                        );
                    }
                }
                Err(error) => {
                    fail!(
                        from origin,
                        with SendError::Delivery,
                        "Failed to notify {}: {:?}", self.name, error
                    );
                }
            }
        }
        Ok(sent)
    }
}

fn apply_settings<S: Service>(
    builder: event::Builder<S>,
    settings: &EventSettings,
) -> event::Builder<S> {
    let builder = builder
        .max_notifiers(settings.max_notifiers)
        .max_listeners(settings.max_listeners)
        .max_nodes(settings.max_nodes)
        .event_id_max_value(settings.event_id_max_value);
    let builder = match settings.deadline {
        Some(deadline) => builder.deadline(deadline),
        None => builder.disable_deadline(),
    };
    let builder = match settings.notifier_created_event {
        Some(id) => builder.notifier_created_event(id),
        None => builder.disable_notifier_created_event(),
    };
    let builder = match settings.notifier_dropped_event {
        Some(id) => builder.notifier_dropped_event(id),
        None => builder.disable_notifier_dropped_event(),
    };
    match settings.notifier_dead_event {
        Some(id) => builder.notifier_dead_event(id),
        None => builder.disable_notifier_dead_event(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use alloc::vec::Vec;
    use iceoryx2::node::NodeBuilder;
    use iceoryx2::service::local;
    use iceoryx2::testing::{generate_isolated_config, generate_service_name};
    use iceoryx2_bb_testing::assert_that;

    fn received(listener: &Listener<local::Service>) -> Vec<EventId> {
        let mut ids = Vec::new();
        listener
            .try_wait(|activation| ids.push(activation.id))
            .expect("waiting succeeds");
        ids
    }

    #[test]
    fn notifications_cross_the_ports_in_both_directions() {
        const APP_ID: usize = 3;
        const PORTS_ID: usize = 5;

        let config = generate_isolated_config();
        let app_node = NodeBuilder::new()
            .config(&config)
            .create::<local::Service>()
            .expect("node is created");
        let link_node = NodeBuilder::new()
            .config(&config)
            .create::<local::Service>()
            .expect("node is created");
        let service_name = generate_service_name();
        let app_service = app_node
            .service_builder(&service_name)
            .event()
            .create()
            .expect("service is created");
        let app_notifier = app_service
            .notifier_builder()
            .create()
            .expect("notifier is created");
        let app_listener = app_service
            .listener_builder()
            .create()
            .expect("listener is created");

        let mut sut = EventPorts::open(
            &link_node,
            &service_name,
            &EventSettings::from_config(&config),
        )
        .expect("ports open on the existing service");

        // Out of the local system: the app notifies, the ports receive.
        app_notifier
            .notify_with_custom_event_id(EventId::new(APP_ID))
            .expect("notification is sent");
        let mut ids = Vec::new();
        let propagated = sut
            .receive(|id| {
                ids.push(id);
                Ok::<(), ()>(())
            })
            .expect("receiving succeeds");
        assert_that!(propagated, eq 1);
        assert_that!(ids, eq alloc::vec![EventId::new(APP_ID)]);
        // The app's own listener hears the app's notifier too.
        assert_that!(received(&app_listener), eq alloc::vec![EventId::new(APP_ID)]);

        // Into the local system: the ports notify, the app receives.
        let mut pending = alloc::vec![EventId::new(PORTS_ID)];
        let sent = sut
            .send(|| Ok::<_, ()>(pending.pop()))
            .expect("sending succeeds");
        assert_that!(sent, eq 1);
        assert_that!(received(&app_listener), eq alloc::vec![EventId::new(PORTS_ID)]);
    }

    #[test]
    fn the_ports_own_notifications_do_not_come_back() {
        const EVENT_ID: usize = 7;

        let config = generate_isolated_config();
        let link_node = NodeBuilder::new()
            .config(&config)
            .create::<local::Service>()
            .expect("node is created");
        let service_name = generate_service_name();

        let mut sut = EventPorts::open(
            &link_node,
            &service_name,
            &EventSettings::from_config(&config),
        )
        .expect("ports open");
        let mut pending = alloc::vec![EventId::new(EVENT_ID)];
        sut.send(|| Ok::<_, ()>(pending.pop()))
            .expect("sending succeeds");

        let propagated = sut
            .receive(|_| Ok::<(), ()>(()))
            .expect("receiving succeeds");
        assert_that!(propagated, eq 0);
    }

    #[test]
    fn an_id_beyond_the_ceiling_is_dropped() {
        const CEILING: usize = 4;
        const BEYOND_CEILING: usize = 9;

        let config = generate_isolated_config();
        let link_node = NodeBuilder::new()
            .config(&config)
            .create::<local::Service>()
            .expect("node is created");
        let service_name = generate_service_name();
        let mut settings = EventSettings::from_config(&config);
        settings.event_id_max_value = CEILING;

        let mut sut = EventPorts::open(&link_node, &service_name, &settings).expect("ports open");
        let mut pending = alloc::vec![EventId::new(BEYOND_CEILING)];
        let sent = sut
            .send(|| Ok::<_, ()>(pending.pop()))
            .expect("sending succeeds");

        assert_that!(sent, eq 0);
    }
}
