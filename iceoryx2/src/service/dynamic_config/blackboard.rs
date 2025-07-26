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

//! # Example
//!
//! ```
//! use iceoryx2::prelude::*;
//!
//! # fn main() -> Result<(), Box<dyn core::error::Error>> {
//! let node = NodeBuilder::new().create::<ipc::Service>()?;
//! type KeyType = u64;
//! let blackboard = node.service_builder(&"My/Funk/ServiceName".try_into()?)
//!     .blackboard_creator::<KeyType>()
//!     .add::<i32>(0,0)
//!     .create()?;
//!
//! println!("number of active readers:      {:?}", blackboard.dynamic_config().number_of_readers());
//! # Ok(())
//! # }
//! ```

use crate::node::NodeId;
use crate::port::port_identifiers::{UniquePortId, UniqueReaderId, UniqueWriterId};
use iceoryx2_bb_container::queue::RelocatableContainer;
use iceoryx2_bb_lock_free::mpmc::{container::*, unique_index_set::ReleaseMode};
use iceoryx2_bb_log::fatal_panic;
use iceoryx2_bb_memory::bump_allocator::BumpAllocator;

use super::PortCleanupAction;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub(crate) struct DynamicConfigSettings {
    pub number_of_readers: usize,
    pub number_of_writers: usize,
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

/// Contains the communication settings of the connected
/// [`Writer`](crate::port::writer::Writer).
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct WriterDetails {
    /// The [`UniqueWriterId`] of the [`Writer`](crate::port::writer::Writer).
    pub writer_id: UniqueWriterId,
    /// The [`NodeId`] of the [`Node`](crate::node::Node) under which the
    /// [`Writer`](crate::port::writer::Writer) was created.
    pub node_id: NodeId,
}

/// The dynamic configuration of an
/// [`crate::service::messaging_pattern::MessagingPattern::Blackboard`]
/// based service. Contains dynamic parameters like the connected endpoints etc..
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

    /// Returns how many [`Reader`](crate::port::reader::Reader) ports are currently connected.
    pub fn number_of_readers(&self) -> usize {
        self.readers.len()
    }

    /// Returns how many [`Writer`](crate::port::writer::Writer) ports are currently connected.
    pub fn number_of_writers(&self) -> usize {
        self.writers.len()
    }

    /// Iterates over all [`Reader`](crate::port::reader::Reader)s and calls the
    /// callback with the corresponding [`ReaderDetails`].
    /// The callback shall return [`CallbackProgression::Continue`] when the iteration shall
    /// continue otherwise [`CallbackProgression::Stop`].
    pub fn list_readers<F: FnMut(&ReaderDetails) -> CallbackProgression>(&self, mut callback: F) {
        let state = unsafe { self.readers.get_state() };

        state.for_each(|_, details| callback(details));
    }

    /// Iterates over all [`Writer`](crate::port::writer::Writer)s and calls the
    /// callback with the corresponding [`WriterDetails`].
    /// The callback shall return [`CallbackProgression::Continue`] when the iteration shall
    /// continue otherwise [`CallbackProgression::Stop`].
    pub fn list_writers<F: FnMut(&WriterDetails) -> CallbackProgression>(&self, mut callback: F) {
        let state = unsafe { self.writers.get_state() };

        state.for_each(|_, details| callback(details));
    }

    pub(crate) unsafe fn remove_dead_node_id<
        PortCleanup: FnMut(UniquePortId) -> PortCleanupAction,
    >(
        &self,
        node_id: &NodeId,
        mut port_cleanup_callback: PortCleanup,
    ) {
        self.readers
            .get_state()
            .for_each(|handle: ContainerHandle, registered_reader| {
                if registered_reader.node_id == *node_id
                    && port_cleanup_callback(UniquePortId::Reader(registered_reader.reader_id))
                        == PortCleanupAction::RemovePort
                {
                    self.release_reader_handle(handle);
                }
                CallbackProgression::Continue
            });

        self.writers
            .get_state()
            .for_each(|handle: ContainerHandle, registered_writer| {
                if registered_writer.node_id == *node_id
                    && port_cleanup_callback(UniquePortId::Writer(registered_writer.writer_id))
                        == PortCleanupAction::RemovePort
                {
                    self.release_writer_handle(handle);
                }
                CallbackProgression::Continue
            });
    }

    pub(crate) fn add_reader_id(&self, id: ReaderDetails) -> Option<ContainerHandle> {
        unsafe { self.readers.add(id).ok() }
    }

    pub(crate) fn release_reader_handle(&self, handle: ContainerHandle) {
        unsafe { self.readers.remove(handle, ReleaseMode::Default) };
    }

    pub(crate) fn add_writer_id(&self, id: WriterDetails) -> Option<ContainerHandle> {
        unsafe { self.writers.add(id).ok() }
    }

    pub(crate) fn release_writer_handle(&self, handle: ContainerHandle) {
        unsafe { self.writers.remove(handle, ReleaseMode::Default) };
    }
}
