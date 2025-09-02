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

use iceoryx2_bb_container::slotmap::*;
use iceoryx2_bb_elementary::bump_allocator::BumpAllocator;
use iceoryx2_bb_elementary_traits::placement_default::PlacementDefault;
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing::memory::RawMemory;

mod slot_map {
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
            assert_that!(sut.remove(*key), eq Some(n));
            assert_that!(sut.remove(*key), eq None);
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

        assert_that!(sut.remove(SlotMapKey::new(SUT_CAPACITY + 1)), eq None);
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
    fn insert_at_and_remove_adjust_map_len_correctly() {
        let mut sut = FixedSizeSut::new();

        for n in 0..SUT_CAPACITY {
            let key = SlotMapKey::new(n);
            assert_that!(sut.len(), eq n);
            assert_that!(sut.insert_at(key, 0), eq true);
        }
        assert_that!(sut.len(), eq SUT_CAPACITY);

        for n in (0..SUT_CAPACITY).rev() {
            let key = SlotMapKey::new(n);
            assert_that!(sut.remove(key), eq Some(0));
            assert_that!(sut.remove(key), eq None);
            assert_that!(sut.len(), eq n);
        }
        assert_that!(sut.len(), eq 0);
    }

    #[test]
    fn insert_does_not_use_insert_at_indices() {
        let mut sut = FixedSizeSut::new();

        for n in 0..SUT_CAPACITY / 2 {
            let key = SlotMapKey::new(2 * n + 1);
            assert_that!(sut.insert_at(key, 0), eq true);
        }

        for _ in 0..SUT_CAPACITY / 2 {
            let key = sut.insert(0);
            assert_that!(key, is_some);
            assert_that!(key.unwrap().value() % 2, eq 0);
        }

        assert_that!(sut.insert(0), is_none);
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

    #[test]
    fn insert_remove_and_insert_works() {
        let mut sut = FixedSizeSut::new();

        for _ in 0..SUT_CAPACITY {
            assert_that!(sut.insert(3), is_some);
        }

        for n in 0..SUT_CAPACITY / 2 {
            assert_that!(sut.remove(SlotMapKey::new(2 * n)), eq Some(3));
        }

        for _ in 0..SUT_CAPACITY / 2 {
            let key = sut.insert(2);
            assert_that!(key, is_some);
        }

        for (key, value) in sut.iter() {
            if key.value() % 2 == 0 {
                assert_that!(*value, eq 2);
            } else {
                assert_that!(*value, eq 3);
            }
        }
    }

    #[test]
    fn next_free_key_returns_key_used_for_insert() {
        let mut sut = FixedSizeSut::new();
        let mut keys = vec![];

        for _ in 0..SUT_CAPACITY / 2 {
            keys.push(sut.insert(0).unwrap());
        }

        let next_key = sut.next_free_key();
        assert_that!(next_key, is_some);
        assert_that!(sut.insert(0), eq next_key);
    }

    #[test]
    fn next_free_key_returns_none_when_full() {
        let mut sut = FixedSizeSut::new();
        let mut keys = vec![];

        for _ in 0..SUT_CAPACITY {
            keys.push(sut.insert(0).unwrap());
        }

        let next_key = sut.next_free_key();
        assert_that!(next_key, is_none);
    }

    #[test]
    fn placement_default_works() {
        let mut sut = RawMemory::<FixedSizeSut>::new_zeroed();
        unsafe { FixedSizeSut::placement_default(sut.as_mut_ptr()) };

        let res = unsafe { sut.assume_init_mut() }.insert(4);
        assert_that!(res, is_some);
    }

    #[test]
    #[should_panic]
    fn double_init_call_causes_panic() {
        const MEM_SIZE: usize = RelocatableSlotMap::<usize>::const_memory_size(SUT_CAPACITY);
        let mut memory = [0u8; MEM_SIZE];
        let bump_allocator = BumpAllocator::new(memory.as_mut_ptr());

        let mut sut = unsafe { RelocatableSlotMap::<usize>::new_uninit(SUT_CAPACITY) };
        unsafe { sut.init(&bump_allocator).expect("sut init failed") };

        unsafe { sut.init(&bump_allocator).expect("sut init failed") };
    }

    #[cfg(debug_assertions)]
    #[test]
    #[should_panic]
    fn panic_is_called_in_debug_mode_if_map_is_not_initialized() {
        let mut sut = unsafe { RelocatableSlotMap::<u8>::new_uninit(SUT_CAPACITY) };
        unsafe { sut.insert(1) };
    }
}
