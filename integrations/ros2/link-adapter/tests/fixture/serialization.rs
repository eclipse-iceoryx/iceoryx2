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

use core::ffi::c_void;

use iceoryx2_bb_testing::assert_that;
use r2r_rcl::{
    RMW_RET_OK, rcutils_get_default_allocator, rmw_deserialize, rmw_serialize,
    rmw_serialized_message_t, rosidl_message_type_support_t,
};
use rosidl_runtime_rs::RmwMessage;

/// The serialization of `message` by the rmw.
pub fn serialize<M: RmwMessage>(message: &M) -> Vec<u8> {
    // Serialize the message.
    let mut serialized = rmw_serialized_message_t {
        buffer: core::ptr::null_mut(),
        buffer_length: 0,
        buffer_capacity: 0,
        allocator: unsafe { rcutils_get_default_allocator() },
    };
    let ret = unsafe {
        rmw_serialize(
            (message as *const M).cast::<c_void>(),
            type_support::<M>(),
            &mut serialized,
        )
    };
    assert_that!(ret, eq RMW_RET_OK as i32);

    // Copy the bytes out before releasing the buffer the rmw allocated.
    let bytes = unsafe { core::slice::from_raw_parts(serialized.buffer, serialized.buffer_length) }
        .to_vec();
    if let Some(deallocate) = serialized.allocator.deallocate {
        unsafe {
            deallocate(
                serialized.buffer.cast::<c_void>(),
                serialized.allocator.state,
            )
        };
    }

    bytes
}

/// The message the rmw deserializes from `bytes`.
pub fn deserialize<M: RmwMessage>(bytes: &[u8]) -> M {
    // Deserialize the message.
    let serialized = rmw_serialized_message_t {
        buffer: bytes.as_ptr().cast_mut(),
        buffer_length: bytes.len(),
        buffer_capacity: bytes.len(),
        allocator: unsafe { rcutils_get_default_allocator() },
    };
    let mut message = M::default();
    let ret = unsafe {
        rmw_deserialize(
            &serialized,
            type_support::<M>(),
            (&mut message as *mut M).cast::<c_void>(),
        )
    };
    assert_that!(ret, eq RMW_RET_OK as i32);

    message
}

fn type_support<M: RmwMessage>() -> *const rosidl_message_type_support_t {
    M::get_type_support().cast::<rosidl_message_type_support_t>()
}
