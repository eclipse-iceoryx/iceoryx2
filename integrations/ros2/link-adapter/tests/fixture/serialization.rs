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
    RMW_RET_OK, rcutils_allocator_t, rcutils_get_default_allocator, rmw_deserialize, rmw_serialize,
    rmw_serialized_message_t, rosidl_message_type_support_t,
};
use rosidl_runtime_rs::RmwMessage;

/// The serialization of `message` by the rmw.
/// The serialized form of `message` as the rmw reports it. The rmw leaves
/// padding uninitialized and reports its size estimate as the length, so
/// the bytes come from a zeroing allocator to make them comparable.
pub fn serialize<M: RmwMessage>(message: &M) -> Vec<u8> {
    let mut buffer = Vec::new();
    let mut serialized = rmw_serialized_message_t {
        buffer: core::ptr::null_mut(),
        buffer_length: 0,
        buffer_capacity: 0,
        allocator: zeroing_allocator(&mut buffer),
    };
    let ret = unsafe {
        rmw_serialize(
            (message as *const M).cast::<c_void>(),
            type_support::<M>(),
            &mut serialized,
        )
    };
    assert_that!(ret, eq RMW_RET_OK as i32);

    buffer.truncate(serialized.buffer_length);
    buffer
}

/// An `rcutils` allocator over `buffer` that zero-fills whatever it hands
/// out, so the bytes the rmw does not write are deterministic.
fn zeroing_allocator(buffer: &mut Vec<u8>) -> rcutils_allocator_t {
    unsafe extern "C" fn allocate(size: usize, state: *mut c_void) -> *mut c_void {
        let buffer = unsafe { &mut *state.cast::<Vec<u8>>() };
        buffer.clear();
        buffer.resize(size, 0);
        buffer.as_mut_ptr().cast::<c_void>()
    }

    unsafe extern "C" fn reallocate(
        _pointer: *mut c_void,
        size: usize,
        state: *mut c_void,
    ) -> *mut c_void {
        let buffer = unsafe { &mut *state.cast::<Vec<u8>>() };
        buffer.resize(size, 0);
        buffer.as_mut_ptr().cast::<c_void>()
    }

    unsafe extern "C" fn deallocate(_pointer: *mut c_void, _state: *mut c_void) {}

    unsafe extern "C" fn zero_allocate(
        number_of_elements: usize,
        size_of_element: usize,
        state: *mut c_void,
    ) -> *mut c_void {
        unsafe { allocate(number_of_elements * size_of_element, state) }
    }

    rcutils_allocator_t {
        allocate: Some(allocate),
        deallocate: Some(deallocate),
        reallocate: Some(reallocate),
        zero_allocate: Some(zero_allocate),
        state: (buffer as *mut Vec<u8>).cast::<c_void>(),
    }
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
