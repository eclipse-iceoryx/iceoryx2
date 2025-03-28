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

use iceoryx2_cal::{shared_memory::ShmPointer, shm_allocator::PointerOffset};

use crate::service::static_config::message_type_details::MessageTypeDetails;

#[derive(Debug)]
pub(crate) struct Chunk {
    pub(crate) header: *const u8,
    pub(crate) user_header: *const u8,
    pub(crate) payload: *const u8,
}

impl Chunk {
    pub(crate) fn new(message_type_details: &MessageTypeDetails, offset: usize) -> Self {
        let header = offset as *const u8;
        Self {
            user_header: message_type_details.user_header_ptr_from_header(header),
            payload: message_type_details.payload_ptr_from_header(header),
            header: offset as *const u8,
        }
    }
}

#[derive(Debug)]
pub(crate) struct ChunkMut {
    pub(crate) offset: PointerOffset,
    pub(crate) header: *mut u8,
    pub(crate) user_header: *mut u8,
    pub(crate) payload: *mut u8,
    pub(crate) size: usize,
}

impl ChunkMut {
    pub(crate) fn new(
        message_type_details: &MessageTypeDetails,
        shm_pointer: ShmPointer,
        size: usize,
    ) -> Self {
        Self {
            user_header: message_type_details
                .user_header_ptr_from_header(shm_pointer.data_ptr)
                .cast_mut(),
            payload: message_type_details
                .payload_ptr_from_header(shm_pointer.data_ptr)
                .cast_mut(),
            header: shm_pointer.data_ptr,
            offset: shm_pointer.offset,
            size,
        }
    }
}
