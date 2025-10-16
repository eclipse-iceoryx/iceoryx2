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
    node::Node,
    port::{listener::Listener, notifier::Notifier},
    prelude::EventId,
    service::{static_config::StaticConfig, Service},
};
use iceoryx2_bb_container::hash_set::HashSet;
use iceoryx2_bb_log::{fail, trace};

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
}

impl core::fmt::Display for ReceiveError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ReceiveError::{self:?}")
    }
}

impl core::error::Error for ReceiveError {}

#[derive(Debug)]
pub(crate) struct EventPorts<S: Service> {
    pub(crate) static_config: StaticConfig,
    pub(crate) notifier: Notifier<S>,
    pub(crate) listener: Listener<S>,
}

impl<S: Service> EventPorts<S> {
    pub(crate) fn new(static_config: &StaticConfig, node: &Node<S>) -> Result<Self, CreationError> {
        let origin = format!("EventPorts<{}>::new", core::any::type_name::<S>());

        let event_config = static_config.event();
        let service = fail!(
            from origin,
            when node
                .service_builder(static_config.name())
                .event()
                .max_nodes(event_config.max_nodes())
                .max_listeners(event_config.max_listeners())
                .max_notifiers(event_config.max_notifiers())
                .event_id_max_value(event_config.event_id_max_value())
                .open_or_create(),
            with CreationError::Service,
            "Failed to open or create service {}({})", static_config.messaging_pattern(), static_config.name()
        );

        let notifier = fail!(
            from origin,
            when service.notifier_builder().create(),
            with CreationError::Notifier,
            "Failed to create Notifier for {}({})", static_config.messaging_pattern(), static_config.name()
        );

        let listener = fail!(
            from origin,
            when service.listener_builder().create(),
            with CreationError::Listener,
            "Failed to create Listener for {}({})", static_config.messaging_pattern(), static_config.name()
        );

        Ok(EventPorts {
            static_config: static_config.clone(),
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
                    trace!(
                        from self,
                        "Sending {}({})",
                        self.static_config.messaging_pattern(),
                        self.static_config.name()
                    );

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
        let mut received_ids: HashSet<EventId> = HashSet::new();

        // Consolidate pending event ids
        while let Ok(event_id) = self.listener.try_wait_one() {
            match event_id {
                Some(event_id) => {
                    received_ids.insert(event_id);
                }
                None => break,
            }
        }

        // Notify all ids once
        for event_id in received_ids {
            trace!(
                from self,
                "Received {}({})",
                self.static_config.messaging_pattern(),
                self.static_config.name()
            );
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
