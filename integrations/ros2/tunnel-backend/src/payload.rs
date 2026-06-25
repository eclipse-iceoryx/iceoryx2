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

//! Reinterpretation between iceoryx2's untyped payload marker and raw
//! bytes.

use iceoryx2::service::builder::CustomPayloadMarker;

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
