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

#[generic_tests::define]
mod used_chunk_list {
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_cal::zero_copy_connection::used_chunk_list::FixedSizeUsedChunkList;

    #[test]
    fn used_chunk_list_insert_pop_works<const CAPACITY: usize>() {
        let mut sut = FixedSizeUsedChunkList::<CAPACITY>::new();

        for i in 0..sut.capacity() {
            assert_that!(sut.insert(i), eq true);
            assert_that!(sut.insert(i), eq false);
        }

        for i in 0..sut.capacity() {
            assert_that!(sut.insert(i), eq false);
            assert_that!(sut.pop(), eq Some(i));
            assert_that!(sut.insert(i), eq true);
            assert_that!(sut.pop(), eq Some(i));
        }

        assert_that!(sut.pop(), eq None);
    }

    #[test]
    fn used_chunk_list_insert_remove_works<const CAPACITY: usize>() {
        let mut sut = FixedSizeUsedChunkList::<CAPACITY>::new();

        for i in 0..sut.capacity() {
            assert_that!(sut.remove(i), eq false);
            assert_that!(sut.insert(i), eq true);
            assert_that!(sut.remove(i), eq true);
            assert_that!(sut.remove(i), eq false);

            assert_that!(sut.insert(i), eq true);
        }

        for i in (0..sut.capacity()).rev() {
            assert_that!(sut.remove(i), eq true);
            assert_that!(sut.remove(i), eq false);
        }

        assert_that!(sut.pop(), eq None);
    }

    #[instantiate_tests(<1>)]
    mod capacity_1 {}

    #[instantiate_tests(<2>)]
    mod capacity_2 {}

    #[instantiate_tests(<3>)]
    mod capacity_3 {}

    #[instantiate_tests(<128>)]
    mod capacity_128 {}
}
