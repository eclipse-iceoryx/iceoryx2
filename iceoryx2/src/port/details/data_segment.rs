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

use core::alloc::Layout;

use iceoryx2_bb_log::fail;
use iceoryx2_bb_system_types::file_name::FileName;
use iceoryx2_cal::{
    event::NamedConceptBuilder,
    resizable_shared_memory::*,
    shared_memory::{
        SharedMemory, SharedMemoryBuilder, SharedMemoryCreateError, SharedMemoryForPoolAllocator,
        SharedMemoryOpenError, ShmPointer,
    },
    shm_allocator::{
        self, pool_allocator::PoolAllocator, AllocationError, AllocationStrategy, PointerOffset,
        SegmentId, ShmAllocationError,
    },
};

use crate::{
    config,
    service::{
        self,
        config_scheme::{data_segment_config, resizable_data_segment_config},
    },
};

/// Defines the data segment type of a zero copy capable sender port.
#[repr(C)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum DataSegmentType {
    /// The data segment can be resized if no more memory is available.
    Dynamic,
    /// The data segment is allocated once. If it is out-of-memory no reallocation will occur.
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
enum MemoryType<Service: service::Service> {
    Static(Service::SharedMemory),
    Dynamic(Service::ResizableSharedMemory),
}

#[derive(Debug)]
pub(crate) struct DataSegment<Service: service::Service> {
    memory: MemoryType<Service>,
}

impl<Service: service::Service> DataSegment<Service> {
    pub(crate) fn create_static_segment(
        segment_name: &FileName,
        chunk_layout: Layout,
        global_config: &config::Config,
        number_of_chunks: usize,
    ) -> Result<Self, SharedMemoryCreateError> {
        let allocator_config = shm_allocator::pool_allocator::Config {
            bucket_layout: chunk_layout,
        };
        let msg = "Unable to create the static data segment since the underlying shared memory could not be created.";
        let origin = "DataSegment::create_static_segment()";

        let segment_config = data_segment_config::<Service>(global_config);
        let memory = fail!(from origin,
                                when <<Service::SharedMemory as SharedMemory<PoolAllocator>>::Builder as NamedConceptBuilder<
                                Service::SharedMemory,
                                    >>::new(segment_name)
                                    .config(&segment_config)
                                    .size(chunk_layout.size() * number_of_chunks + chunk_layout.align() - 1)
                                    .create(&allocator_config),
                                "{msg}");

        Ok(Self {
            memory: MemoryType::Static(memory),
        })
    }

    pub(crate) fn create_dynamic_segment(
        segment_name: &FileName,
        chunk_layout: Layout,
        global_config: &config::Config,
        number_of_chunks: usize,
        allocation_strategy: AllocationStrategy,
    ) -> Result<Self, SharedMemoryCreateError> {
        let msg = "Unable to create the dynamic data segment since the underlying shared memory could not be created.";
        let origin = "DataSegment::create_dynamic_segment()";

        let segment_config = resizable_data_segment_config::<Service>(global_config);
        let memory = fail!(from origin,
                    when <<Service::ResizableSharedMemory as ResizableSharedMemory<
                        PoolAllocator,
                        Service::SharedMemory,
                    >>::MemoryBuilder as NamedConceptBuilder<Service::ResizableSharedMemory>>::new(
                        segment_name,
                    )
                    .config(&segment_config)
                    .max_number_of_chunks_hint(number_of_chunks)
                    .max_chunk_layout_hint(chunk_layout)
                    .allocation_strategy(allocation_strategy)
                    .create(),
                    "{msg}");

        Ok(Self {
            memory: MemoryType::Dynamic(memory),
        })
    }

    pub(crate) fn allocate(&self, layout: Layout) -> Result<ShmPointer, ShmAllocationError> {
        let msg = "Unable to allocate memory from the data segment";
        match &self.memory {
            MemoryType::Static(memory) => Ok(fail!(from self, when memory.allocate(layout),
                                            "{msg}.")),
            MemoryType::Dynamic(memory) => match memory.allocate(layout) {
                Ok(ptr) => Ok(ptr),
                Err(ResizableShmAllocationError::ShmAllocationError(e)) => {
                    fail!(from self, with e,
                        "{msg} caused by {:?}.", e);
                }
                Err(ResizableShmAllocationError::MaxReallocationsReached) => {
                    fail!(from self,
                        with ShmAllocationError::AllocationError(AllocationError::OutOfMemory),
                        "{msg} since the maxmimum number of reallocations was reached. Try to provide initial_max_slice_len({}) as hint when creating the publisher to have a more fitting initial setup.", layout.size());
                }
                Err(ResizableShmAllocationError::SharedMemoryCreateError(e)) => {
                    fail!(from self,
                        with ShmAllocationError::AllocationError(AllocationError::InternalError),
                        "{msg} since the shared memory segment creation failed while resizing the memory due to ({:?}).", e);
                }
            },
        }
    }

    pub(crate) unsafe fn deallocate_bucket(&self, offset: PointerOffset) {
        match &self.memory {
            MemoryType::Static(memory) => memory.deallocate_bucket(offset),
            MemoryType::Dynamic(memory) => memory.deallocate_bucket(offset),
        }
    }

    pub(crate) fn bucket_size(&self, segment_id: SegmentId) -> usize {
        match &self.memory {
            MemoryType::Static(memory) => memory.bucket_size(),
            MemoryType::Dynamic(memory) => memory.bucket_size(segment_id),
        }
    }

    pub(crate) fn max_number_of_segments(data_segment_type: DataSegmentType) -> u8 {
        match data_segment_type {
            DataSegmentType::Static => 1,
            DataSegmentType::Dynamic => {
                (Service::ResizableSharedMemory::max_number_of_reallocations() - 1) as u8
            }
        }
    }
}

#[derive(Debug)]
enum MemoryViewType<Service: service::Service> {
    Static(Service::SharedMemory),
    Dynamic(
        <Service::ResizableSharedMemory as ResizableSharedMemory<
            PoolAllocator,
            Service::SharedMemory,
        >>::View,
    ),
}

#[derive(Debug)]
pub(crate) struct DataSegmentView<Service: service::Service> {
    memory: MemoryViewType<Service>,
}

impl<Service: service::Service> DataSegmentView<Service> {
    pub(crate) fn open_static_segment(
        segment_name: &FileName,
        global_config: &config::Config,
    ) -> Result<Self, SharedMemoryOpenError> {
        let origin = "DataSegment::open()";
        let msg =
            "Unable to open data segment since the underlying shared memory could not be opened.";

        let segment_config = data_segment_config::<Service>(global_config);
        let memory = fail!(from origin,
                            when <Service::SharedMemory as SharedMemory<PoolAllocator>>::
                                Builder::new(segment_name)
                                .config(&segment_config)
                                .timeout(global_config.global.service.creation_timeout)
                                .open(),
                            "{msg}");

        Ok(Self {
            memory: MemoryViewType::Static(memory),
        })
    }

    pub(crate) fn open_dynamic_segment(
        segment_name: &FileName,
        global_config: &config::Config,
    ) -> Result<Self, SharedMemoryOpenError> {
        let origin = "DataSegment::open()";
        let msg =
            "Unable to open data segment since the underlying shared memory could not be opened.";

        let segment_config = resizable_data_segment_config::<Service>(global_config);
        let memory = fail!(from origin,
                    when <<Service::ResizableSharedMemory as ResizableSharedMemory<
                        PoolAllocator,
                        Service::SharedMemory,
                    >>::ViewBuilder as NamedConceptBuilder<Service::ResizableSharedMemory>>::new(
                        segment_name,
                    )
                    .config(&segment_config)
                    .open(),
                    "{msg}");

        Ok(Self {
            memory: MemoryViewType::Dynamic(memory),
        })
    }

    pub(crate) fn register_and_translate_offset(
        &self,
        offset: PointerOffset,
    ) -> Result<usize, SharedMemoryOpenError> {
        match &self.memory {
            MemoryViewType::Static(memory) => Ok(offset.offset() + memory.payload_start_address()),
            MemoryViewType::Dynamic(memory) => unsafe {
                match memory.register_and_translate_offset(offset) {
                    Ok(ptr) => Ok(ptr as usize),
                    Err(e) => {
                        fail!(from self, with e,
                            "Failed to register and translate pointer due to a failure while opening the corresponding shared memory segment ({:?}).",
                            e);
                    }
                }
            },
        }
    }

    pub(crate) unsafe fn unregister_offset(&self, offset: PointerOffset) {
        if let MemoryViewType::Dynamic(memory) = &self.memory {
            memory.unregister_offset(offset);
        }
    }

    pub(crate) fn is_dynamic(&self) -> bool {
        matches!(&self.memory, MemoryViewType::Dynamic(_))
    }
}
