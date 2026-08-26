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

use core::mem::MaybeUninit;

use iceoryx2::service::marker::{CustomHeaderMarker, CustomPayloadMarker};

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

/// Copies user header and payload bytes into a loan.
///
/// # Safety
///
/// `destination_user_header` must point to at least
/// `source_user_header.len()` writable bytes.
pub unsafe fn write_message(
    user_header_bytes: &[u8],
    payload_bytes: &[u8],
    user_header_loan: *mut CustomHeaderMarker,
    payload_loan: &mut [MaybeUninit<CustomPayloadMarker>],
) {
    debug_assert!(
        payload_loan.len() >= payload_bytes.len(),
        "Loaned payload size ({}) is too small for received payload ({})",
        payload_loan.len(),
        payload_bytes.len()
    );

    unsafe {
        core::ptr::copy_nonoverlapping(
            user_header_bytes.as_ptr(),
            user_header_loan as *mut u8,
            user_header_bytes.len(),
        );
        core::ptr::copy_nonoverlapping(
            payload_bytes.as_ptr(),
            payload_loan.as_mut_ptr().cast::<u8>(),
            payload_bytes.len(),
        );
    }
}
