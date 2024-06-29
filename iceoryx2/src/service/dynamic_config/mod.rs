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

/// The dynamic service configuration of an
/// [`MessagingPattern::Event`](crate::service::messaging_pattern::MessagingPattern::Event)
/// based service.
pub mod event;

/// The dynamic service configuration of an
/// [`MessagingPattern::PublishSubscribe`](crate::service::messaging_pattern::MessagingPattern::PublishSubscribe)
/// based service.
pub mod publish_subscribe;

use iceoryx2_bb_container::queue::RelocatableContainer;
use iceoryx2_bb_lock_free::mpmc::{
    container::{Container, ContainerAddFailure, ContainerHandle},
    unique_index_set::{ReleaseMode, ReleaseState},
};
use iceoryx2_bb_log::{fail, fatal_panic};
use iceoryx2_bb_memory::bump_allocator::BumpAllocator;
use std::fmt::Display;

use crate::node::NodeId;

pub(crate) enum RegisterNodeResult {
    MarkedForDestruction,
    MaximumNodesExceeded,
}

pub(crate) enum DeregisterNodeState {
    HasOwners,
    NoMoreOwners,
}

#[derive(Debug)]
pub(crate) enum MessagingPattern {
    PublishSubscribe(publish_subscribe::DynamicConfig),
    Event(event::DynamicConfig),
}

#[doc(hidden)]
#[derive(Debug)]
pub struct DynamicConfig {
    messaging_pattern: MessagingPattern,
    nodes: Container<NodeId>,
}

impl Display for DynamicConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "service::DynamicConfig {{ messaging_pattern: {:?} }}",
            self.messaging_pattern
        )
    }
}

impl DynamicConfig {
    pub(crate) fn new_uninit(
        messaging_pattern: MessagingPattern,
        max_number_of_nodes: usize,
    ) -> Self {
        Self {
            messaging_pattern,
            nodes: unsafe { Container::new_uninit(max_number_of_nodes) },
        }
    }

    pub(crate) fn memory_size(max_number_of_nodes: usize) -> usize {
        Container::<NodeId>::memory_size(max_number_of_nodes)
    }

    pub(crate) unsafe fn init(&self, allocator: &BumpAllocator) {
        fatal_panic!(from self, when self.nodes.init(allocator),
            "This should never happen! Unable to initialize NodeId container.");
        match &self.messaging_pattern {
            MessagingPattern::PublishSubscribe(ref v) => v.init(allocator),
            MessagingPattern::Event(ref v) => v.init(allocator),
        }
    }

    pub(crate) fn register_node_id(
        &self,
        node_id: NodeId,
    ) -> Result<ContainerHandle, RegisterNodeResult> {
        let msg = "Unable to register NodeId in service";
        match unsafe { self.nodes.add(node_id) } {
            Ok(handle) => Ok(handle),
            Err(ContainerAddFailure::IsLocked) => {
                fail!(from self, with RegisterNodeResult::MarkedForDestruction,
                    "{msg} since the service is already marked for destruction.");
            }
            Err(ContainerAddFailure::OutOfSpace) => {
                fail!(from self, with RegisterNodeResult::MaximumNodesExceeded,
                    "{msg} since it would exceed the maximum supported nodes of {}.", self.nodes.capacity());
            }
        }
    }

    pub(crate) fn list_node_ids(&self) -> Vec<NodeId> {
        let mut node_ids = vec![];
        let state = unsafe { self.nodes.get_state() };
        state.for_each(|_, node_id| node_ids.push(*node_id));
        node_ids
    }

    pub(crate) fn deregister_node_id(&self, handle: ContainerHandle) -> DeregisterNodeState {
        if unsafe { self.nodes.remove(handle, ReleaseMode::LockIfLastIndex) }
            == ReleaseState::Locked
        {
            DeregisterNodeState::NoMoreOwners
        } else {
            DeregisterNodeState::HasOwners
        }
    }

    pub(crate) fn publish_subscribe(&self) -> &publish_subscribe::DynamicConfig {
        match &self.messaging_pattern {
            MessagingPattern::PublishSubscribe(ref v) => v,
            m => {
                fatal_panic!(from self, "This should never happen! Try to access publish_subscribe::DynamicConfig when the messaging pattern is actually {:?}.", m);
            }
        }
    }

    pub(crate) fn event(&self) -> &event::DynamicConfig {
        match &self.messaging_pattern {
            MessagingPattern::Event(ref v) => v,
            m => {
                fatal_panic!(from self, "This should never happen! Try to access event::DynamicConfig when the messaging pattern is actually {:?}.", m);
            }
        }
    }
}
