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

use iceoryx2_cal::shm_allocator::AllocationStrategy;

use crate::service;

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

pub(crate) struct DataSegment<Service: service::Service> {
    memory: Service::SharedMemory,
}
