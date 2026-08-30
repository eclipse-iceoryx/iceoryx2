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

use iceoryx2::unique_id_generator::UniqueIdBuilder;
use iceoryx2::unique_id_generator::blub::UniqueSystemId;
use iceoryx2_bb_concurrency::atomic::AtomicU32;
use iceoryx2_bb_container::string::StaticString;
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing_macros::test;

#[test]
fn generated_id_contains_counter_hint() {
    let counter_hint = 432780;
    let id = UniqueIdBuilder::<UniqueSystemId>::new(&StaticString::new())
        .id_hint(AtomicU32::new(counter_hint))
        .create(0)
        .unwrap();

    let sut = UniqueSystemId::from(id);
    assert_that!(sut.counter(), eq counter_hint);
}
