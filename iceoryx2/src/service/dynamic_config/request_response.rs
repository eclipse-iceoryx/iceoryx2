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
    port::{
        details::data_segment::DataSegmentType,
        port_identifiers::{UniqueClientId, UniquePortId, UniqueServerId},
    },
};

use super::PortCleanupAction;

/// Contains the communication settings of the connected
/// [`Server`](crate::port::server::Server).
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ServerDetails {
    /// The [`UniqueServerId`] of the [`Server`](crate::port::server::Server).
    pub server_id: UniqueServerId,
    /// The [`NodeId`] of the [`Node`](crate::node::Node) under which the
    /// [`Server`](crate::port::server::Server) was created.
    pub node_id: NodeId,
    /// The receive buffer size for incoming requests.
    pub request_buffer_size: usize,
    /// The total number of responses available in the
    /// [`Server`](crate::port::server::Server)s data segment
    pub number_of_responses: usize,
    /// The current maximum length of a slice.
    pub max_slice_len: usize,
    /// The type of data segment the [`Server`](crate::port::server::Server)
    /// uses.
    pub data_segment_type: DataSegmentType,
    /// If the [`Server`](crate::port::server::Server) has the
    /// [`DataSegmentType::Dynamic`] it defines how many segment the
    /// [`Server`](crate::port::server::Server) can have at most.
    pub max_number_of_segments: u8,
}

/// Contains the communication settings of the connected
/// [`Client`](crate::port::client::Client).
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ClientDetails {
    /// The [`UniqueClientId`] of the [`Client`](crate::port::client::Client).
    pub client_id: UniqueClientId,
    /// The [`NodeId`] of the [`Node`](crate::node::Node) under which the
    /// [`Client`](crate::port::client::Client) was created.
    pub node_id: NodeId,
    /// The total number of requests available in the
    /// [`Client`](crate::port::client::Client)s data segment
    pub number_of_requests: usize,
    /// The receive buffer size for incoming responses.
    pub response_buffer_size: usize,
    /// The current maximum length of a slice.
    pub max_slice_len: usize,
    /// The type of data segment the [`Client`](crate::port::client::Client)
    /// uses.
    pub data_segment_type: DataSegmentType,
    /// If the [`Client`](crate::port::client::Client) has the
    /// [`DataSegmentType::Dynamic`] it defines how many segment the
    /// [`Client`](crate::port::client::Client) can have at most.
    pub max_number_of_segments: u8,
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
        node_id: &NodeId,
        mut port_cleanup_callback: PortCleanup,
    ) {
        self.servers
            .get_state()
            .for_each(|handle: ContainerHandle, registered_server| {
                if registered_server.node_id == *node_id
                    && port_cleanup_callback(UniquePortId::Server(registered_server.server_id))
                        == PortCleanupAction::RemovePort
                {
                    self.release_server_handle(handle);
                }
                CallbackProgression::Continue
            });

        self.clients
            .get_state()
            .for_each(|handle: ContainerHandle, registered_client| {
                if registered_client.node_id == *node_id
                    && port_cleanup_callback(UniquePortId::Client(registered_client.client_id))
                        == PortCleanupAction::RemovePort
                {
                    self.release_client_handle(handle);
                }
                CallbackProgression::Continue
            });
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

    /// Iterates over all [`Server`](crate::port::server::Server)s and calls the
    /// callback with the corresponding [`ServerDetails`].
    /// The callback shall return [`CallbackProgression::Continue`] when the iteration shall
    /// continue otherwise [`CallbackProgression::Stop`].
    pub fn list_servers<F: FnMut(&ServerDetails) -> CallbackProgression>(&self, mut callback: F) {
        let state = unsafe { self.servers.get_state() };

        state.for_each(|_, details| callback(details));
    }

    /// Iterates over all [`Client`](crate::port::client::Client)s and calls the
    /// callback with the corresponding [`ClientDetails`].
    /// The callback shall return [`CallbackProgression::Continue`] when the iteration shall
    /// continue otherwise [`CallbackProgression::Stop`].
    pub fn list_clients<F: FnMut(&ClientDetails) -> CallbackProgression>(&self, mut callback: F) {
        let state = unsafe { self.clients.get_state() };

        state.for_each(|_, details| callback(details));
    }
}
