// Copyright (c) 2024 Contributors to the Eclipse Foundation
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

use iceoryx2_bb_container::slotmap::SlotMap;
use iceoryx2_bb_testing::assert_that;

mod slot_map {

    use super::*;

    const SUT_CAPACITY: usize = 128;
    type Sut = SlotMap<usize>;

    #[test]
    fn new_slotmap_is_empty() {
        let sut = Sut::new(SUT_CAPACITY);

        assert_that!(sut, is_empty);
        assert_that!(sut.is_full(), eq false);
        assert_that!(sut, len 0);
    }
}