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

use iceoryx2_bb_testing_macros::tests;

#[tests(1, 2, 3, 63, 64, 65, 128)]
pub mod generic {
    use alloc::vec;

    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_cal::zero_copy_connection::used_chunk_list::FixedSizeUsedChunkList;

    #[test]
    fn insert_remove_all_works<const CAPACITY: usize>() {
        let mut sut = FixedSizeUsedChunkList::<CAPACITY>::new();

        for i in 0..sut.capacity() {
            assert_that!(sut.insert(i), eq true);
        }

        let mut removed_indices = vec![false; sut.capacity()];
        sut.remove_all(|index| {
            removed_indices[index] = true;
        });

        for index in removed_indices {
            assert_that!(index, eq true);
        }
    }

    #[test]
    fn insert_remove_works<const CAPACITY: usize>() {
        let sut = FixedSizeUsedChunkList::<CAPACITY>::new();

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
    }

    #[test]
    fn remove_all_returns_exactly_the_inserted_values<const CAPACITY: usize>() {
        const STEP: usize = 3;
        let mut sut = FixedSizeUsedChunkList::<CAPACITY>::new();
        let mut inserted = vec![false; sut.capacity()];

        for i in (0..sut.capacity())
            .step_by(STEP)
            .chain([sut.capacity() - 1])
        {
            sut.insert(i);
            inserted[i] = true;
        }

        let mut removed = vec![false; sut.capacity()];
        sut.remove_all(|index| {
            assert_that!(removed[index], eq false);
            removed[index] = true;
        });
        assert_that!(removed, eq inserted);

        let mut removed_twice = 0;
        sut.remove_all(|_| removed_twice += 1);
        assert_that!(removed_twice, eq 0);
    }
}
