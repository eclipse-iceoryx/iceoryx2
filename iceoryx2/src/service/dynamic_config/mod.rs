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

/// The dynamic service configuration of an
/// [`MessagingPattern::RequestResponse`](crate::service::messaging_pattern::MessagingPattern::RequestResponse)
/// based service.
pub mod request_response;

/// The dynamic service configuration of an
/// [`MessagingPattern::Blackboard`](crate::service::messaging_pattern::MessagingPattern::Blackboard)
/// based service.
pub mod blackboard;

use core::fmt::Display;
use iceoryx2_bb_container::queue::RelocatableContainer;
use iceoryx2_bb_elementary::CallbackProgression;
use iceoryx2_bb_lock_free::mpmc::{
    container::{Container, ContainerAddFailure, ContainerHandle},
    unique_index_set_enums::{ReleaseMode, ReleaseState},
};
use iceoryx2_bb_memory::bump_allocator::BumpAllocator;
use iceoryx2_log::{fail, fatal_panic};

use crate::identifiers::{UniqueNodeId, UniquePortId};

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum PortCleanupAction {
    RemovePort,
    SkipPort,
}

#[derive(Debug)]
pub(crate) enum RegisterNodeResult {
    MarkedForDestruction,
    ExceedsMaxNumberOfNodes,
}

pub(crate) enum DeregisterNodeState {
    HasOwners,
    NoMoreOwners,
}

#[derive(Debug)]
pub(crate) enum MessagingPatternSettings {
    RequestResponse(request_response::DynamicConfigSettings),
    PublishSubscribe(publish_subscribe::DynamicConfigSettings),
    Event(event::DynamicConfigSettings),
    Blackboard(blackboard::DynamicConfigSettings),
}

#[derive(Debug)]
pub(crate) enum MessagingPattern {
    RequestResponse(request_response::DynamicConfig),
    PublishSubscribe(publish_subscribe::DynamicConfig),
    Event(event::DynamicConfig),
    Blackboard(blackboard::DynamicConfig),
}

impl MessagingPattern {
    pub(crate) fn new(settings: &MessagingPatternSettings) -> Self {
        match settings {
            MessagingPatternSettings::RequestResponse(v) => {
                MessagingPattern::RequestResponse(request_response::DynamicConfig::new(v))
            }
            MessagingPatternSettings::PublishSubscribe(v) => {
                MessagingPattern::PublishSubscribe(publish_subscribe::DynamicConfig::new(v))
            }
            MessagingPatternSettings::Event(v) => {
                MessagingPattern::Event(event::DynamicConfig::new(v))
            }
            MessagingPatternSettings::Blackboard(v) => {
                MessagingPattern::Blackboard(blackboard::DynamicConfig::new(v))
            }
        }
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct DynamicConfig {
    messaging_pattern: MessagingPattern,
    nodes: Container<UniqueNodeId>,
}

impl Display for DynamicConfig {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
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
        Container::<UniqueNodeId>::memory_size(max_number_of_nodes)
    }

    pub(crate) unsafe fn init(&mut self, allocator: &BumpAllocator) {
        unsafe {
            fatal_panic!(from self, when self.nodes.init(allocator),
            "This should never happen! Unable to initialize NodeId container.");
            match &mut self.messaging_pattern {
                MessagingPattern::PublishSubscribe(v) => v.init(allocator),
                MessagingPattern::Event(v) => v.init(allocator),
                MessagingPattern::RequestResponse(v) => v.init(allocator),
                MessagingPattern::Blackboard(v) => v.init(allocator),
            }
        }
    }

    pub(crate) unsafe fn remove_dead_node_id<
        PortCleanup: FnMut(UniquePortId) -> PortCleanupAction,
    >(
        &self,
        node_id: &UniqueNodeId,
        port_cleanup_callback: PortCleanup,
    ) -> DeregisterNodeState {
        unsafe {
            match self.messaging_pattern {
                MessagingPattern::PublishSubscribe(ref v) => {
                    v.remove_dead_node_id(node_id, port_cleanup_callback)
                }
                MessagingPattern::Event(ref v) => {
                    v.remove_dead_node_id(node_id, port_cleanup_callback)
                }
                MessagingPattern::RequestResponse(ref v) => {
                    v.remove_dead_node_id(node_id, port_cleanup_callback)
                }
                MessagingPattern::Blackboard(ref v) => {
                    v.remove_dead_node_id(node_id, port_cleanup_callback)
                }
            };

            match self.nodes.recover(
                |owner_id, _| owner_id == node_id.owner_id(),
                ReleaseMode::LockIfLastIndex,
            ) {
                ReleaseState::Locked => DeregisterNodeState::NoMoreOwners,
                ReleaseState::Unlocked => DeregisterNodeState::HasOwners,
            }
        }
    }

    pub(crate) fn register_node_id(
        &self,
        node_id: UniqueNodeId,
    ) -> Result<ContainerHandle, RegisterNodeResult> {
        let msg = "Unable to register NodeId in service";
        match unsafe { self.nodes.add(node_id, node_id.owner_id()) } {
            Ok(handle) => Ok(handle),
            Err(ContainerAddFailure::IsLocked) => {
                fail!(from self, with RegisterNodeResult::MarkedForDestruction,
                    "{msg} since the service is already marked for destruction.");
            }
            Err(ContainerAddFailure::OutOfSpace) => {
                fail!(from self, with RegisterNodeResult::ExceedsMaxNumberOfNodes,
                    "{msg} since it would exceed the maximum supported nodes of {}.", self.nodes.capacity());
            }
        }
    }

    pub(crate) fn list_node_ids<F: FnMut(&UniqueNodeId) -> CallbackProgression>(
        &self,
        mut callback: F,
    ) {
        let state = unsafe { self.nodes.get_state() };
        state.for_each(|_, node_id| callback(node_id));
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

    pub(crate) fn request_response(&self) -> &request_response::DynamicConfig {
        match &self.messaging_pattern {
            MessagingPattern::RequestResponse(v) => v,
            m => {
                fatal_panic!(from self, "This should never happen! Trying to access request_response::DynamicConfig when the messaging pattern is actually {:?}.", m);
            }
        }
    }

    pub(crate) fn publish_subscribe(&self) -> &publish_subscribe::DynamicConfig {
        match &self.messaging_pattern {
            MessagingPattern::PublishSubscribe(v) => v,
            m => {
                fatal_panic!(from self, "This should never happen! Trying to access publish_subscribe::DynamicConfig when the messaging pattern is actually {:?}.", m);
            }
        }
    }

    pub(crate) fn event(&self) -> &event::DynamicConfig {
        match &self.messaging_pattern {
            MessagingPattern::Event(v) => v,
            m => {
                fatal_panic!(from self, "This should never happen! Trying to access event::DynamicConfig when the messaging pattern is actually {:?}.", m);
            }
        }
    }

    pub(crate) fn blackboard(&self) -> &blackboard::DynamicConfig {
        match &self.messaging_pattern {
            MessagingPattern::Blackboard(v) => v,
            m => {
                fatal_panic!(from self, "This should never happen! Trying to access blackboard::DynamicConfig when the messaging pattern is actually {:?}.", m);
            }
        }
    }
}
