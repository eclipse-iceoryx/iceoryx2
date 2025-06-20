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

use crate::node::NodeId;
use crate::port::port_identifiers::{UniquePortId, UniqueReaderId};
use iceoryx2_bb_container::queue::RelocatableContainer;
use iceoryx2_bb_lock_free::mpmc::container::Container;
use iceoryx2_bb_log::fatal_panic;
use iceoryx2_bb_memory::bump_allocator::BumpAllocator;

use super::PortCleanupAction;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub(crate) struct DynamicConfigSettings {
    pub number_of_readers: usize,
}

/// Contains the communication settings of the connected
/// [`Reader`](crate::port::reader::Reader).
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ReaderDetails {
    /// The [`UniqueReaderId`] of the [`Reader`](crate::port::reader::Reader).
    pub reader_id: UniqueReaderId,
    /// The [`NodeId`] of the [`Node`](crate::node::Node) under which the
    /// [`Reader`](crate::port::reader::Reader) was created.
    pub node_id: NodeId,
}

/// The dynamic configuration of an
/// [`crate::service::messaging_pattern::MessagingPattern::Blackboard`]
/// based service. Contains dynamic parameters like the connected endpoints etc..
#[repr(C)]
#[derive(Debug)]
pub struct DynamicConfig {
    pub(crate) readers: Container<ReaderDetails>,
}

impl DynamicConfig {
    pub(crate) fn new(config: &DynamicConfigSettings) -> Self {
        Self {
            readers: unsafe { Container::new_uninit(config.number_of_readers) },
        }
    }

    pub(crate) unsafe fn init(&mut self, allocator: &BumpAllocator) {
        fatal_panic!(from self,
            when self.readers.init(allocator),
            "This should never happen! Unable to initialize reader port id container.");
    }

    pub(crate) fn memory_size(config: &DynamicConfigSettings) -> usize {
        Container::<ReaderDetails>::memory_size(config.number_of_readers)
    }

    pub(crate) unsafe fn remove_dead_node_id<
        PortCleanup: FnMut(UniquePortId) -> PortCleanupAction,
    >(
        &self,
        _node_id: &NodeId,
        mut _port_cleanup_callback: PortCleanup,
    ) {
        todo!();
    }
}
