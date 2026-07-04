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

//! Reinterpretation between iceoryx2's untyped sample markers and raw
//! bytes.

use core::mem::MaybeUninit;

use iceoryx2::service::Service;
use iceoryx2::service::marker::{CustomHeaderMarker, CustomPayloadMarker};
use iceoryx2_services_tunnel_backend::types::publish_subscribe::{SampleMut, SampleMutUninit};

// The byte views rely on the marker being exactly one byte.
const _: () = assert!(core::mem::size_of::<CustomPayloadMarker>() == 1);
const _: () = assert!(core::mem::align_of::<CustomPayloadMarker>() == 1);

/// A byte view of an untyped payload.
pub fn as_bytes(payload: &[CustomPayloadMarker]) -> &[u8] {
    // SAFETY: the marker is a single byte (asserted above), so length and
    // alignment carry over.
    // The memory is initialized by the publishing application.
    #[allow(unsafe_code)]
    unsafe {
        core::slice::from_raw_parts(payload.as_ptr().cast::<u8>(), payload.len())
    }
}

/// A raw byte pointer to an uninitialized untyped payload, as the
/// destination for an external writer.
pub fn uninit_bytes_ptr(payload: &mut [MaybeUninit<CustomPayloadMarker>]) -> *mut u8 {
    payload.as_mut_ptr().cast()
}

/// Writes `header` into an untyped user header location.
///
/// The caller must ensure that the service's user-header type detail
/// matches `T` — the location is only as large as that detail states.
pub fn write_user_header<T>(location: &mut CustomHeaderMarker, header: T) {
    // SAFETY: size and alignment of the location are guaranteed by the
    // caller (see above).
    #[allow(unsafe_code)]
    unsafe {
        (location as *mut CustomHeaderMarker)
            .cast::<T>()
            .write(header)
    }
}

/// Marks a sample whose payload was fully initialized by an external
/// writer as initialized.
pub fn assume_init<S: Service>(sample: SampleMutUninit<S>) -> SampleMut<S> {
    // SAFETY: the caller guarantees the payload was fully written.
    #[allow(unsafe_code)]
    unsafe {
        sample.assume_init()
    }
}
