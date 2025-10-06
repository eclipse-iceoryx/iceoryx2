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

use std::collections::HashSet;

use iceoryx2::{
    node::{Node, NodeId},
    port::{
        listener::{Listener, ListenerCreateError},
        notifier::{Notifier, NotifierCreateError},
    },
    prelude::{EventId, PortFactory},
    service::{builder::event::EventOpenOrCreateError, static_config::StaticConfig, Service},
};
use iceoryx2_bb_log::fail;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum CreationError {
    Error,
}

impl From<EventOpenOrCreateError> for CreationError {
    fn from(_: EventOpenOrCreateError) -> Self {
        CreationError::Error
    }
}

impl From<NotifierCreateError> for CreationError {
    fn from(_: NotifierCreateError) -> Self {
        CreationError::Error
    }
}

impl From<ListenerCreateError> for CreationError {
    fn from(_: ListenerCreateError) -> Self {
        CreationError::Error
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum WaitError {
    Error,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum NotifyError {
    Error,
}

#[derive(Debug)]
pub(crate) struct Ports<S: Service> {
    pub(crate) static_config: StaticConfig,
    pub(crate) notifier: Notifier<S>,
    pub(crate) listener: Listener<S>,
}

impl<S: Service> Ports<S> {
    pub(crate) fn new(static_config: &StaticConfig, node: &Node<S>) -> Result<Self, CreationError> {
        let event_config = static_config.event();
        let service = fail!(
            from "Ports::new",
            when node
                .service_builder(static_config.name())
                .event()
                .max_nodes(event_config.max_nodes())
                .max_listeners(event_config.max_listeners())
                .max_notifiers(event_config.max_notifiers())
                .event_id_max_value(event_config.event_id_max_value())
                .open_or_create(),
            "Failed to open or create event service"
        );

        let notifier = fail!(
            from "create_notifier()",
            when service.notifier_builder().create(),
            "{}",
            &format!("Failed to create Notifier for '{}'", service.name())
        );

        let listener = fail!(
            from "create_listener()",
            when service.listener_builder().create(),
            "{}",
            &format!("Failed to create Listener for '{}'", service.name())
        );

        Ok(Ports {
            static_config: static_config.clone(),
            notifier,
            listener,
        })
    }

    pub(crate) fn try_wait_all<PropagateFn>(
        &self,
        mut propagate: PropagateFn,
    ) -> Result<(), WaitError>
    where
        // TODO: Handle failed propagation
        PropagateFn: FnMut(EventId),
    {
        // let mut notified_ids: HashSet<usize> = HashSet::new();
        while let Ok(event_id) = self.listener.try_wait_one() {
            match event_id {
                Some(event_id) => {
                    propagate(event_id);
                }
                None => break,
            }
        }

        Ok(())
    }

    pub(crate) fn notify(&self, event_id: EventId) -> Result<(), NotifyError> {
        fail!(
            from "Ports::notify",
            when self.notifier.__internal_notify(event_id, true),
            with NotifyError::Error,
            "Failed to propagate remote notification"
        );
        Ok(())
    }
}
