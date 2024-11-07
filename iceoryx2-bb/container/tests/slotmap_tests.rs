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

    use iceoryx2_bb_container::slotmap::{FixedSizeSlotMap, SlotMapKey};

    use super::*;

    const SUT_CAPACITY: usize = 128;
    type Sut = SlotMap<usize>;
    type FixedSizeSut = FixedSizeSlotMap<usize, SUT_CAPACITY>;

    #[test]
    fn new_slotmap_is_empty() {
        let sut = Sut::new(SUT_CAPACITY);

        assert_that!(sut, len 0);
        assert_that!(sut, is_empty);
        assert_that!(sut.is_full(), eq false);
        assert_that!(sut.capacity(), eq SUT_CAPACITY);
    }

    #[test]
    fn new_fixed_size_slotmap_is_empty() {
        let sut = FixedSizeSut::new();

        assert_that!(sut, len 0);
        assert_that!(sut, is_empty);
        assert_that!(sut.is_full(), eq false);
        assert_that!(sut.capacity(), eq SUT_CAPACITY);
    }

    #[test]
    fn inserting_elements_works() {
        let mut sut = FixedSizeSut::new();

        for i in 0..SUT_CAPACITY {
            assert_that!(sut.is_full(), eq false);
            let key = sut.insert(i).unwrap();
            *sut.get_mut(key).unwrap() += i;
            assert_that!(*sut.get(key).unwrap(), eq 2 * i);
            assert_that!(sut, len i + 1);
            assert_that!(sut.is_empty(), eq false);
        }

        assert_that!(sut.is_full(), eq true);
        assert_that!(sut.insert(123), is_none);
    }

    #[test]
    fn insert_when_full_fails() {
        let mut sut = FixedSizeSut::new();

        for i in 0..SUT_CAPACITY {
            assert_that!(sut.insert(i), is_some);
        }

        assert_that!(sut.insert(34), is_none);
    }

    #[test]
    fn removing_elements_works() {
        let mut sut = FixedSizeSut::new();
        let mut keys = vec![];

        for i in 0..SUT_CAPACITY {
            keys.push(sut.insert(i).unwrap());
        }

        for (n, key) in keys.iter().enumerate() {
            assert_that!(sut.len(), eq sut.capacity() - n);
            assert_that!(sut.is_empty(), eq false);
            assert_that!(sut.contains(*key), eq true);
            assert_that!(sut.remove(*key), eq true);
            assert_that!(sut.remove(*key), eq false);
            assert_that!(sut.contains(*key), eq false);
            assert_that!(sut.is_full(), eq false);

            assert_that!(sut.get(*key), is_none);
            assert_that!(sut.get_mut(*key), is_none);
        }

        assert_that!(sut.is_empty(), eq true);
    }

    #[test]
    fn removing_out_of_bounds_key_returns_false() {
        let mut sut = FixedSizeSut::new();

        assert_that!(sut.remove(SlotMapKey::new(SUT_CAPACITY + 1)), eq false);
    }

    #[test]
    fn insert_at_works() {
        let mut sut = FixedSizeSut::new();

        let key = SlotMapKey::new(5);
        let value = 71823;
        assert_that!(sut.insert_at(key, 781), eq true);
        assert_that!(sut.insert_at(key, value), eq true);

        assert_that!(*sut.get(key).unwrap(), eq value);
    }

    #[test]
    fn insert_at_out_of_bounds_key_returns_false() {
        let mut sut = FixedSizeSut::new();
        let key = SlotMapKey::new(SUT_CAPACITY + 1);
        assert_that!(sut.insert_at(key, 781), eq false);
    }

    #[test]
    fn iterating_works() {
        let mut sut = FixedSizeSut::new();
        let mut keys = vec![];

        for i in 0..SUT_CAPACITY {
            keys.push(sut.insert(5 * i + 3).unwrap());
        }

        for (key, value) in sut.iter() {
            assert_that!(*value, eq 5 * key.value() + 3);
        }
    }
}
