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

use alloc::collections::BTreeSet;
use alloc::format;

use iceoryx2::{
    node::Node,
    port::{listener::Listener, notifier::Notifier},
    prelude::EventId,
    service::{Service, builder::event, service_name::ServiceName},
};
use iceoryx2_log::{fail, trace};
use iceoryx2_services_tunnel_backend::types::service_description::{
    EventDescription, EventSettings, PatternSettings,
};

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
    EventIngestion,
    NotificationDelivery,
}

impl core::fmt::Display for SendError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "SendError::{self:?}")
    }
}

impl core::error::Error for SendError {}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum ReceiveError {
    NotificationPropagation,
    NotificationReceival,
}

impl core::fmt::Display for ReceiveError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ReceiveError::{self:?}")
    }
}

impl core::error::Error for ReceiveError {}

#[derive(Debug)]
pub(crate) struct EventPorts<S: Service> {
    pub(crate) name: ServiceName,
    pub(crate) notifier: Notifier<S>,
    pub(crate) listener: Listener<S>,
}

impl<S: Service> EventPorts<S> {
    pub(crate) fn new(
        name: &ServiceName,
        description: &EventDescription,
        node: &Node<S>,
    ) -> Result<Self, CreationError> {
        let origin = format!("EventPorts<{}>::new", core::any::type_name::<S>());

        let builder = node.service_builder(name).event();
        let builder = match &description.settings {
            PatternSettings::Value(settings) => apply_settings(builder, settings),
            PatternSettings::UnknownApplyDefaults => builder,
        };

        let service = fail!(
            from origin,
            when builder.open_or_create(),
            with CreationError::Service,
            "Failed to open or create service Event({})", name
        );
        let notifier = fail!(
            from origin,
            when service.notifier_builder().create(),
            with CreationError::Notifier,
            "Failed to create Notifier for Event({})", name
        );
        let listener = fail!(
            from origin,
            when service.listener_builder().create(),
            with CreationError::Listener,
            "Failed to create Listener for Event({})", name
        );
        Ok(EventPorts {
            name: *name,
            notifier,
            listener,
        })
    }

    pub(crate) fn send<IngestFn, IngestError>(
        &self,
        mut ingest: IngestFn,
    ) -> Result<bool, SendError>
    where
        IngestFn: for<'a> FnMut() -> Result<Option<EventId>, IngestError>,
    {
        let mut ingested = false;
        loop {
            let event_id = fail!(
                from self,
                when ingest(),
                with SendError::EventIngestion,
                "Failed to ingest event from backend"
            );

            match event_id {
                Some(event_id) => {
                    trace!(from self, "Sending Event({})", self.name);

                    fail!(
                        from self,
                        when self.notifier.__internal_notify(event_id, true),
                        with SendError::NotificationDelivery,
                        "Failed to send notification"
                    );

                    ingested = true;
                }
                None => break,
            }
        }

        Ok(ingested)
    }

    // TODO(#1103): Preserve ordering of events received over the backend.
    pub(crate) fn receive<PropagateFn, E>(
        &self,
        mut propagate: PropagateFn,
    ) -> Result<bool, ReceiveError>
    where
        PropagateFn: FnMut(EventId) -> Result<(), E>,
    {
        let mut propagated = false;
        let mut received_ids: BTreeSet<EventId> = BTreeSet::new();

        // Consolidate pending event ids
        if let Err(e) = self.listener.try_wait(|event| {
            received_ids.insert(event.id);
        }) {
            fail!(from self, with ReceiveError::NotificationReceival,
                "Failed to receive notifications from listener. [{e:?}]");
        }

        // Notify all ids once
        for event_id in received_ids {
            trace!(from self, "Received Event({})", self.name);
            fail!(
                from self,
                when propagate(event_id),
                with ReceiveError::NotificationPropagation,
                "Failed to propagate received event to backend"
            );

            propagated = true;
        }

        Ok(propagated)
    }
}

fn apply_settings<S: Service>(
    builder: event::Builder<S>,
    settings: &EventSettings,
) -> event::Builder<S> {
    let builder = builder
        .max_nodes(settings.max_nodes)
        .max_listeners(settings.max_listeners)
        .max_notifiers(settings.max_notifiers)
        .event_id_max_value(settings.event_id_max_value);
    let builder = match settings.deadline {
        Some(deadline) => builder.deadline(deadline),
        None => builder.disable_deadline(),
    };
    let builder = match settings.notifier_created_event {
        Some(value) => builder.notifier_created_event(EventId::new(value)),
        None => builder.disable_notifier_created_event(),
    };
    let builder = match settings.notifier_dropped_event {
        Some(value) => builder.notifier_dropped_event(EventId::new(value)),
        None => builder.disable_notifier_dropped_event(),
    };
    match settings.notifier_dead_event {
        Some(value) => builder.notifier_dead_event(EventId::new(value)),
        None => builder.disable_notifier_dead_event(),
    }
}
