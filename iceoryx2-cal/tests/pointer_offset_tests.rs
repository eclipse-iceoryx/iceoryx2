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

mod pointer_offset {
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_cal::shm_allocator::{PointerOffset, SegmentId};

    #[test]
    fn new_works() {
        const TEST_OFFSET: usize = 123914;
        let sut = PointerOffset::new(TEST_OFFSET);

        assert_that!(sut.offset(), eq TEST_OFFSET);
        assert_that!(sut.segment_id(), eq SegmentId::new(0));
    }

    #[test]
    fn set_segment_id_works() {
        const TEST_OFFSET: usize = 123914;
        const SEGMENT_ID: SegmentId = SegmentId::new(7);
        let mut sut = PointerOffset::new(TEST_OFFSET);
        sut.set_segment_id(SEGMENT_ID);

        assert_that!(sut.offset(), eq TEST_OFFSET);
        assert_that!(sut.segment_id(), eq SEGMENT_ID);
    }

    #[test]
    fn set_segment_id_multiple_times_works() {
        const TEST_OFFSET: usize = 123914;
        const SEGMENT_ID_1: SegmentId = SegmentId::new(7);
        const SEGMENT_ID_2: SegmentId = SegmentId::new(8);
        let mut sut = PointerOffset::new(TEST_OFFSET);

        sut.set_segment_id(SEGMENT_ID_1);
        assert_that!(sut.segment_id(), eq SEGMENT_ID_1);

        sut.set_segment_id(SEGMENT_ID_2);
        assert_that!(sut.segment_id(), eq SEGMENT_ID_2);
    }
}
