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

use iceoryx2::unique_id_generator::UniqueId;
use iceoryx2_bb_posix::unique_system_id::UniqueSystemId;
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing_macros::test;

#[test]
fn unique_system_id_is_correctly_converted() {
    let unique_system_id = UniqueSystemId::new().unwrap();
    let unique_id = unsafe { UniqueId::from_raw_id(unique_system_id.value()) };
    assert_that!(unique_id.unique_value() >> 32, eq unique_system_id.counter() as u64);
    assert_that!(unique_id.unique_value() as u32, eq unique_system_id.pid().value() as u32);
    assert_that!(unique_id.payload_value() >> 32, eq unique_system_id.creation_time().nanoseconds() as u64);
    assert_that!(unique_id.payload_value() as u32, eq unique_system_id.creation_time().seconds() as u32);
}
