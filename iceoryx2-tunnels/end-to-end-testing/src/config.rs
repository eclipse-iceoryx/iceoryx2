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

use iceoryx2::prelude::{PlacementDefault, ZeroCopySend};
use iceoryx2_bb_container::{
    byte_string::FixedSizeByteString, queue::FixedSizeQueue, vec::FixedSizeVec,
};
use std::time::Duration;

#[allow(dead_code)]
pub const TIMEOUT_DURATION: Duration = Duration::from_secs(5);
pub const HISTORY_SIZE: usize = 10;
pub const PING_SERVICE_NAME: &str = "tunnel-end-to-end-test/ping";
pub const PONG_SERVICE_NAME: &str = "tunnel-end-to-end-test/pong";

pub trait PayloadWriter {
    type PayloadType: Send
        + Sync
        + core::fmt::Debug
        + PartialEq
        + ZeroCopySend
        + PlacementDefault
        + 'static;

    #[allow(dead_code)]
    fn write_payload(ptr: *mut Self::PayloadType);
}

pub struct PrimitivePayload;

impl PayloadWriter for PrimitivePayload {
    type PayloadType = u64;

    fn write_payload(ptr: *mut Self::PayloadType) {
        unsafe {
            *ptr = 42;
        }
    }
}

#[derive(Debug, PartialEq, Default, PlacementDefault, ZeroCopySend)]
#[repr(C)]
pub struct ComplexData {
    name: FixedSizeByteString<4>,
    data: FixedSizeVec<u64, 4>,
}

#[derive(Debug, PartialEq, PlacementDefault, ZeroCopySend)]
#[repr(C)]
pub struct ComplexType {
    plain_old_data: u64,
    text: FixedSizeByteString<8>,
    vec_of_data: FixedSizeVec<u64, 4>,
    vec_of_complex_data: FixedSizeVec<ComplexData, 404857>,
    a_queue_of_things: FixedSizeQueue<FixedSizeByteString<4>, 2>,
}

pub struct ComplexPayload;

impl PayloadWriter for ComplexPayload {
    type PayloadType = ComplexType;

    fn write_payload(ptr: *mut Self::PayloadType) {
        unsafe {
            Self::PayloadType::placement_default(ptr);
            (*ptr).plain_old_data = 0;
            (*ptr).text = FixedSizeByteString::from_bytes(b"hello").unwrap();
            (*ptr).vec_of_data.push(42);
            (*ptr).vec_of_complex_data.push(ComplexData {
                name: FixedSizeByteString::from_bytes(b"bla").unwrap(),
                data: {
                    let mut v = FixedSizeVec::new();
                    v.fill(42);
                    v
                },
            });
            (*ptr)
                .a_queue_of_things
                .push(FixedSizeByteString::from_bytes(b"buh").unwrap());
        }
    }
}
