// Copyright (c) 2023 Contributors to the Eclipse Foundation
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

//! # Examples
//!
//! ```
//! use iceoryx2::prelude::*;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let node = NodeBuilder::new().create::<ipc::Service>()?;
//! let event = node.service_builder(&"MyEventName".try_into()?)
//!     .event()
//!     .open_or_create()?;
//!
//! println!("number of active listeners:   {:?}", event.dynamic_config().number_of_listeners());
//! println!("number of active notifiers:   {:?}", event.dynamic_config().number_of_notifiers());
//! # Ok(())
//! # }
//! ```
use iceoryx2_bb_elementary::relocatable_container::RelocatableContainer;
use iceoryx2_bb_lock_free::mpmc::{container::*, unique_index_set::ReleaseMode};
use iceoryx2_bb_log::fatal_panic;
use iceoryx2_bb_memory::bump_allocator::BumpAllocator;

use crate::{
    node::NodeId,
    port::port_identifiers::{UniqueListenerId, UniqueNotifierId, UniquePortId},
};

use super::PortCleanupAction;

#[derive(Debug, Clone, Copy)]
pub(crate) struct DynamicConfigSettings {
    pub number_of_listeners: usize,
    pub number_of_notifiers: usize,
}

/// The dynamic configuration of an [`crate::service::messaging_pattern::MessagingPattern::Event`]
/// based service. Contains dynamic parameters like the connected endpoints etc..
#[derive(Debug)]
pub struct DynamicConfig {
    pub(crate) listeners: Container<ListenerDetails>,
    pub(crate) notifiers: Container<NotifierDetails>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ListenerDetails {
    pub(crate) listener_id: UniqueListenerId,
    pub(crate) node_id: NodeId,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct NotifierDetails {
    pub(crate) notifier_id: UniqueNotifierId,
    pub(crate) node_id: NodeId,
}

impl DynamicConfig {
    pub(crate) fn new(config: &DynamicConfigSettings) -> Self {
        Self {
            listeners: unsafe { Container::new_uninit(config.number_of_listeners) },
            notifiers: unsafe { Container::new_uninit(config.number_of_notifiers) },
        }
    }

    pub(crate) unsafe fn init(&self, allocator: &BumpAllocator) {
        fatal_panic!(from "event::DynamicConfig::init",
            when self.listeners.init(allocator),
            "This should never happen! Unable to initialize listener port id container.");
        fatal_panic!(from "event::DynamicConfig::init",
            when self.notifiers.init(allocator),
            "This should never happen! Unable to initialize notifier port id container.");
    }

    pub(crate) fn memory_size(config: &DynamicConfigSettings) -> usize {
        Container::<ListenerDetails>::memory_size(config.number_of_listeners)
            + Container::<NotifierDetails>::memory_size(config.number_of_notifiers)
    }

    /// Returns the how many [`crate::port::listener::Listener`] ports are currently connected.
    pub fn number_of_listeners(&self) -> usize {
        self.listeners.len()
    }

    /// Returns the how many [`crate::port::notifier::Notifier`] ports are currently connected.
    pub fn number_of_notifiers(&self) -> usize {
        self.notifiers.len()
    }

    pub(crate) unsafe fn remove_dead_node_id<
        PortCleanup: FnMut(UniquePortId) -> PortCleanupAction,
    >(
        &self,
        node_id: &NodeId,
        mut port_cleanup_callback: PortCleanup,
    ) {
        self.listeners
            .get_state()
            .for_each(|handle: ContainerHandle, registered_listener| {
                if registered_listener.node_id == *node_id
                    && port_cleanup_callback(UniquePortId::Listener(
                        registered_listener.listener_id,
                    )) == PortCleanupAction::RemovePort
                {
                    self.release_listener_handle(handle);
                }
                CallbackProgression::Continue
            });

        self.notifiers
            .get_state()
            .for_each(|handle: ContainerHandle, registered_notifier| {
                if registered_notifier.node_id == *node_id
                    && port_cleanup_callback(UniquePortId::Notifier(
                        registered_notifier.notifier_id,
                    )) == PortCleanupAction::RemovePort
                {
                    self.release_notifier_handle(handle);
                }
                CallbackProgression::Continue
            });
    }

    pub(crate) fn add_listener_id(&self, id: ListenerDetails) -> Option<ContainerHandle> {
        unsafe { self.listeners.add(id).ok() }
    }

    pub(crate) fn release_listener_handle(&self, handle: ContainerHandle) {
        unsafe { self.listeners.remove(handle, ReleaseMode::Default) };
    }

    pub(crate) fn add_notifier_id(&self, id: NotifierDetails) -> Option<ContainerHandle> {
        unsafe { self.notifiers.add(id).ok() }
    }

    pub(crate) fn release_notifier_handle(&self, handle: ContainerHandle) {
        unsafe { self.notifiers.remove(handle, ReleaseMode::Default) };
    }
}
