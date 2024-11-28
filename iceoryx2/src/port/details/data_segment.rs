// Copyright (c) 2023 - 2024 Contributors to the Eclipse Foundation
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

use std::alloc::Layout;

use iceoryx2_bb_log::fail;
use iceoryx2_cal::{
    event::NamedConceptBuilder,
    shared_memory::{
        SharedMemory, SharedMemoryBuilder, SharedMemoryCreateError, SharedMemoryOpenError,
        ShmPointer,
    },
    shm_allocator::{
        self, pool_allocator::PoolAllocator, AllocationStrategy, PointerOffset, ShmAllocationError,
    },
};

use crate::{
    config,
    service::{
        self, config_scheme::data_segment_config,
        dynamic_config::publish_subscribe::PublisherDetails, naming_scheme::data_segment_name,
    },
};

#[repr(C)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum DataSegmentType {
    Dynamic,
    Static,
}

impl DataSegmentType {
    pub(crate) fn new_from_allocation_strategy(v: AllocationStrategy) -> Self {
        match v {
            AllocationStrategy::Static => DataSegmentType::Static,
            _ => DataSegmentType::Dynamic,
        }
    }
}

#[derive(Debug)]
pub(crate) struct DataSegment<Service: service::Service> {
    memory: Service::SharedMemory,
}

impl<Service: service::Service> DataSegment<Service> {
    pub(crate) fn create(
        details: &PublisherDetails,
        global_config: &config::Config,
        sample_layout: Layout,
    ) -> Result<Self, SharedMemoryCreateError> {
        let allocator_config = shm_allocator::pool_allocator::Config {
            bucket_layout: sample_layout,
        };

        let memory = fail!(from "DataSegment::create()",
            when <<Service::SharedMemory as SharedMemory<PoolAllocator>>::Builder as NamedConceptBuilder<
            Service::SharedMemory,
                >>::new(&data_segment_name(&details.publisher_id))
                .config(&data_segment_config::<Service>(global_config))
                .size(sample_layout.size() * details.number_of_samples + sample_layout.align() - 1)
                .create(&allocator_config),
            "Unable to create the data segment since the underlying shared memory could not be created.");

        Ok(Self { memory })
    }

    pub(crate) fn allocate(&self, layout: Layout) -> Result<ShmPointer, ShmAllocationError> {
        Ok(fail!(from self, when self.memory.allocate(layout),
            "Unable to allocate memory from the data segment."))
    }

    pub(crate) unsafe fn deallocate(&self, offset: PointerOffset, layout: Layout) {
        self.memory.deallocate(offset, layout)
    }
}

#[derive(Debug)]
pub(crate) struct DataSegmentView<Service: service::Service> {
    memory: Service::SharedMemory,
}

impl<Service: service::Service> DataSegmentView<Service> {
    pub(crate) fn open(
        details: &PublisherDetails,
        global_config: &config::Config,
    ) -> Result<Self, SharedMemoryOpenError> {
        let memory = fail!(from "DataSegment::open()",
                            when <Service::SharedMemory as SharedMemory<PoolAllocator>>::
                                Builder::new(&data_segment_name(&details.publisher_id))
                                .config(&data_segment_config::<Service>(global_config))
                                .timeout(global_config.global.service.creation_timeout)
                                .open(),
                            "Unable to open data segment since the underlying shared memory could not be opened.");

        Ok(Self { memory })
    }

    pub(crate) fn register_and_translate_offset(&self, offset: PointerOffset) -> usize {
        offset.offset() + self.memory.payload_start_address()
    }

    pub(crate) unsafe fn unregister_offset(&self, _offset: PointerOffset) {}
}
