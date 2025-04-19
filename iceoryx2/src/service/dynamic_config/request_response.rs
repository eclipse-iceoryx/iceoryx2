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

use iceoryx2_bb_container::queue::RelocatableContainer;
use iceoryx2_bb_elementary::CallbackProgression;
use iceoryx2_bb_lock_free::mpmc::container::{Container, ContainerHandle, ReleaseMode};
use iceoryx2_bb_log::fatal_panic;
use iceoryx2_bb_memory::bump_allocator::BumpAllocator;

use crate::{
    node::NodeId,
    port::port_identifiers::{UniqueClientId, UniquePortId, UniqueServerId},
};

use super::PortCleanupAction;

#[doc(hidden)]
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ServerDetails {
    pub server_port_id: UniqueServerId,
    pub request_buffer_size: usize,
    pub number_of_responses: usize,
}

#[doc(hidden)]
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ClientDetails {
    pub client_port_id: UniqueClientId,
    pub node_id: NodeId,
    pub number_of_requests: usize,
    pub response_buffer_size: usize,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub(crate) struct DynamicConfigSettings {
    pub number_of_servers: usize,
    pub number_of_clients: usize,
}

/// The dynamic configuration of an
/// [`crate::service::messaging_pattern::MessagingPattern::RequestResponse`]
/// based service. Contains dynamic parameters like the connected endpoints etc..
#[repr(C)]
#[derive(Debug)]
pub struct DynamicConfig {
    pub(crate) servers: Container<ServerDetails>,
    pub(crate) clients: Container<ClientDetails>,
}

impl DynamicConfig {
    pub(crate) fn new(config: &DynamicConfigSettings) -> Self {
        Self {
            servers: unsafe { Container::new_uninit(config.number_of_servers) },
            clients: unsafe { Container::new_uninit(config.number_of_clients) },
        }
    }

    pub(crate) unsafe fn init(&mut self, allocator: &BumpAllocator) {
        fatal_panic!(from self,
            when self.servers.init(allocator),
            "This should never happen! Unable to initialize servers port id container.");
        fatal_panic!(from self,
            when self.clients.init(allocator),
            "This should never happen! Unable to initialize clients port id container.");
    }

    pub(crate) fn memory_size(config: &DynamicConfigSettings) -> usize {
        Container::<ServerDetails>::memory_size(config.number_of_servers)
            + Container::<ClientDetails>::memory_size(config.number_of_clients)
    }

    /// Returns how many [`crate::port::client::Client`] ports are currently connected.
    pub fn number_of_clients(&self) -> usize {
        self.clients.len()
    }

    /// Returns how many [`crate::port::server::Server`] ports are currently connected.
    pub fn number_of_servers(&self) -> usize {
        self.servers.len()
    }

    pub(crate) unsafe fn remove_dead_node_id<
        PortCleanup: FnMut(UniquePortId) -> PortCleanupAction,
    >(
        &self,
        _node_id: &NodeId,
        mut _port_cleanup_callback: PortCleanup,
    ) {
        todo!()
    }

    pub(crate) fn add_client_id(&self, details: ClientDetails) -> Option<ContainerHandle> {
        unsafe { self.clients.add(details).ok() }
    }

    pub(crate) fn release_client_handle(&self, handle: ContainerHandle) {
        unsafe { self.clients.remove(handle, ReleaseMode::Default) };
    }

    pub(crate) fn add_server_id(&self, details: ServerDetails) -> Option<ContainerHandle> {
        unsafe { self.servers.add(details).ok() }
    }

    pub(crate) fn release_server_handle(&self, handle: ContainerHandle) {
        unsafe { self.servers.remove(handle, ReleaseMode::Default) };
    }

    #[doc(hidden)]
    pub fn __internal_list_servers<F: FnMut(&ServerDetails)>(&self, mut callback: F) {
        let state = unsafe { self.servers.get_state() };

        state.for_each(|_, details| {
            callback(details);
            CallbackProgression::Continue
        });
    }

    #[doc(hidden)]
    pub fn __internal_list_publishers<F: FnMut(&ClientDetails)>(&self, mut callback: F) {
        let state = unsafe { self.clients.get_state() };

        state.for_each(|_, details| {
            callback(details);
            CallbackProgression::Continue
        });
    }
}
