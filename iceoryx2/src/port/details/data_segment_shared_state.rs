// Copyright (c) 2026 Contributors to the Eclipse Foundation
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

use core::fmt::Debug;

use iceoryx2_bb_elementary_traits::allocator::Grow;
use iceoryx2_bb_elementary_traits::testing::abandonable::Abandonable;
use iceoryx2_bb_flatbuffers::AllocationStrategy;
use iceoryx2_cal::shared_memory::ShmPointer;
use iceoryx2_cal::shm_allocator::PointerOffset;

use crate::service::static_config::message_type_details::MessageTypeDetails;

pub trait DataSegmentSharedState: Abandonable + Send + Debug + Grow<ShmPointer> {
    fn return_loan(&self, offset: PointerOffset);
    fn header_len(&self) -> usize;
    fn message_type_details(&self) -> MessageTypeDetails;
    fn allocation_strategy(&self) -> AllocationStrategy;
    fn payload_size(&self) -> usize;
}
