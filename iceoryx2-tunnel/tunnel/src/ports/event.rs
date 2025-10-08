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
use iceoryx2_bb_log::fail;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum CreationError {
    FailedToCreateService,
    FailedToCreateNotifier,
    FailedToCreateListener,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum SendError {
    FailedToSendNotification,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum ReceiveError {
    FailedToPropagateNotification,
}

#[derive(Debug)]
pub(crate) struct EventPorts<S: Service> {
    pub(crate) static_config: StaticConfig,
    pub(crate) notifier: Notifier<S>,
    pub(crate) listener: Listener<S>,
}

impl<S: Service> EventPorts<S> {
    pub(crate) fn new(static_config: &StaticConfig, node: &Node<S>) -> Result<Self, CreationError> {
        let event_config = static_config.event();
        let service = fail!(
            from "EventPorts::new",
            when node
                .service_builder(static_config.name())
                .event()
                .max_nodes(event_config.max_nodes())
                .max_listeners(event_config.max_listeners())
                .max_notifiers(event_config.max_notifiers())
                .event_id_max_value(event_config.event_id_max_value())
                .open_or_create(),
            with CreationError::FailedToCreateService,
            "{}", format!("Failed to open or create service {}({})", static_config.messaging_pattern(), static_config.name())
        );

        let notifier = fail!(
            from "create_notifier()",
            when service.notifier_builder().create(),
            with CreationError::FailedToCreateNotifier,
            "{}",
            &format!("Failed to create Notifier for {}({})", static_config.messaging_pattern(), static_config.name())
        );

        let listener = fail!(
            from "create_listener()",
            when service.listener_builder().create(),
            with CreationError::FailedToCreateListener,
            "{}",
            &format!("Failed to create Listener for {}({})", static_config.messaging_pattern(),static_config.name())
        );

        Ok(EventPorts {
            static_config: static_config.clone(),
            notifier,
            listener,
        })
    }

    pub(crate) fn send(&self, event_id: EventId) -> Result<(), SendError> {
        fail!(
            from "EventPorts::notify",
            when self.notifier.__internal_notify(event_id, true),
            with SendError::FailedToSendNotification,
            "Failed to send notification"
        );
        Ok(())
    }

    pub(crate) fn receive<PropagateFn, E>(
        &self,
        mut propagate: PropagateFn,
    ) -> Result<(), ReceiveError>
    where
        PropagateFn: FnMut(EventId) -> Result<(), E>,
    {
        // let mut notified_ids: HashSet<usize> = HashSet::new();
        while let Ok(event_id) = self.listener.try_wait_one() {
            match event_id {
                Some(event_id) => {
                    fail!(
                        from "EventPorts::receive",
                        when propagate(event_id),
                        with ReceiveError::FailedToPropagateNotification,
                        "Failed to propagate received event to backend"
                    );
                }
                None => break,
            }
        }

        Ok(())
    }
}
