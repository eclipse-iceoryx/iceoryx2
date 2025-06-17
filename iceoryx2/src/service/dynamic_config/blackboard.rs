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
use crate::port::port_identifiers::{UniquePortId, UniqueReaderId, UniqueWriterId};
use iceoryx2_bb_container::queue::RelocatableContainer;
use iceoryx2_bb_lock_free::mpmc::container::Container;
use iceoryx2_bb_log::fatal_panic;
use iceoryx2_bb_memory::bump_allocator::BumpAllocator;

use super::PortCleanupAction;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub(crate) struct DynamicConfigSettings {
    pub number_of_readers: usize,
    pub number_of_writers: usize,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ReaderDetails {
    pub reader_id: UniqueReaderId,
    pub node_id: NodeId,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct WriterDetails {
    pub writer_id: UniqueWriterId,
    pub node_id: NodeId,
}

#[repr(C)]
#[derive(Debug)]
pub struct DynamicConfig {
    pub(crate) readers: Container<ReaderDetails>,
    pub(crate) writers: Container<WriterDetails>,
}

impl DynamicConfig {
    pub(crate) fn new(config: &DynamicConfigSettings) -> Self {
        Self {
            readers: unsafe { Container::new_uninit(config.number_of_readers) },
            writers: unsafe { Container::new_uninit(config.number_of_writers) },
        }
    }

    pub(crate) unsafe fn init(&mut self, allocator: &BumpAllocator) {
        fatal_panic!(from self,
            when self.readers.init(allocator),
            "This should never happen! Unable to initialize reader port id container.");
        fatal_panic!(from self,
            when self.writers.init(allocator),
            "This should never happen! Unable to initialize writer port id container.");
    }

    pub(crate) fn memory_size(config: &DynamicConfigSettings) -> usize {
        Container::<ReaderDetails>::memory_size(config.number_of_readers)
            + Container::<WriterDetails>::memory_size(config.number_of_writers)
    }

    pub(crate) unsafe fn remove_dead_node_id<
        PortCleanup: FnMut(UniquePortId) -> PortCleanupAction,
    >(
        &self,
        node_id: &NodeId,
        mut port_cleanup_callback: PortCleanup,
    ) {
    }
}
