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

use iceoryx2_bb_elementary::static_assert_size_of;

use crate::service::{header::payload_header::PayloadHeader, marker::CustomPayloadMarker};

pub fn number_of_elements<Payload: Sized, H: PayloadHeader>(
    header: &H,
    payload_size: usize,
) -> usize {
    static_assert_size_of!(CustomPayloadMarker, 1);
    // We need to handle the custom payload marker her, that has always a size of 1
    // and the ability to set custom payload type size/alignment. Therefore, we need
    // to calculate number of elements * payload_size divided again by the payload size.
    // If the generic argument and payload size is equal it will return the actual
    // number of elements.
    //
    // But in the special case of the CustomPayloadMarker, it will divide by 1 and
    // return a slice of bytes with the correct size.
    header.number_of_elements() as usize * payload_size / core::mem::size_of::<Payload>()
}
