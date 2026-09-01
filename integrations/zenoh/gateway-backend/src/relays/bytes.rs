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

//! Byte views of untyped iceoryx2 messages for moving them through zenoh.

use iceoryx2::service::Service;
use iceoryx2::service::marker::{CustomHeaderMarker, CustomPayloadMarker};
use iceoryx2_gateway_backend::types::publish_subscribe::{SampleMut, SampleMutUninit};

/// Views the user header of an untyped message as bytes.
pub fn user_header_bytes(user_header: &CustomHeaderMarker, size: usize) -> &[u8] {
    unsafe {
        core::slice::from_raw_parts(user_header as *const CustomHeaderMarker as *const u8, size)
    }
}

/// Views the payload of an untyped message as bytes.
pub fn payload_bytes(payload: &[CustomPayloadMarker]) -> &[u8] {
    unsafe { core::slice::from_raw_parts(payload.as_ptr() as *const u8, payload.len()) }
}

/// Initializes a loaned sample with received user header and payload
/// bytes.
///
/// # Safety
///
/// `user_header` must be exactly the size of the service's user header
/// type and `payload` must be exactly the size of the loan.
pub unsafe fn initialize_sample<S: Service>(
    user_header: &[u8],
    payload: &[u8],
    mut sample: SampleMutUninit<S>,
) -> SampleMut<S> {
    debug_assert!(
        sample.payload_mut().len() == payload.len(),
        "Loaned payload size ({}) does not match received payload ({})",
        sample.payload_mut().len(),
        payload.len()
    );

    unsafe {
        core::ptr::copy_nonoverlapping(
            user_header.as_ptr(),
            sample.user_header_mut() as *mut CustomHeaderMarker as *mut u8,
            user_header.len(),
        );
        core::ptr::copy_nonoverlapping(
            payload.as_ptr(),
            sample.payload_mut().as_mut_ptr().cast::<u8>(),
            payload.len(),
        );
        sample.assume_init()
    }
}
