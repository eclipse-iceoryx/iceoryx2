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

use iceoryx2_bb_container::flatmap::*;
use iceoryx2_bb_elementary::bump_allocator::BumpAllocator;
use iceoryx2_bb_elementary_traits::placement_default::PlacementDefault;
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing::lifetime_tracker::LifetimeTracker;
use iceoryx2_bb_testing::memory::RawMemory;

mod flat_map {
    use super::*;

    const CAPACITY: usize = 100;

    #[test]
    fn new_creates_empty_flat_map() {
        let map_diff_key = FlatMap::<u8, i32>::new(CAPACITY);
        assert_that!(map_diff_key, is_empty);
        assert_that!(map_diff_key.is_full(), eq false);
        assert_that!(map_diff_key, len 0);
        let map_same_key = FlatMap::<u16, u16>::new(CAPACITY);
        assert_that!(map_same_key, is_empty);
        assert_that!(map_diff_key.is_full(), eq false);
        assert_that!(map_same_key, len 0);
    }

    #[test]
    fn new_creates_empty_fixed_size_flat_map() {
        let map_diff_key = FixedSizeFlatMap::<u8, i32, CAPACITY>::new();
        assert_that!(map_diff_key, is_empty);
        assert_that!(map_diff_key, len 0);
        let map_same_key = FixedSizeFlatMap::<u16, u16, CAPACITY>::new();
        assert_that!(map_same_key, is_empty);
        assert_that!(map_same_key, len 0);
    }

    #[test]
    fn default_creates_empty_flat_map() {
        let map = FixedSizeFlatMap::<u8, u8, CAPACITY>::default();
        assert_that!(map, is_empty);
        assert_that!(map, len 0);
    }

    #[test]
    fn placement_default_works() {
        type Sut = FixedSizeFlatMap<u8, u8, CAPACITY>;
        let mut sut = RawMemory::<Sut>::new_zeroed();
        unsafe { Sut::placement_default(sut.as_mut_ptr()) };

        let res = unsafe { sut.assume_init_mut() }.insert(4, 6);
        assert_that!(res, is_ok);
    }

    #[test]
    fn drop_called_for_keys_and_values() {
        let state = LifetimeTracker::start_tracking();
        let mut map = FixedSizeFlatMap::<LifetimeTracker, LifetimeTracker, CAPACITY>::new();
        for _ in 0..CAPACITY {
            assert_that!(
                map.insert(LifetimeTracker::default(), LifetimeTracker::default()),
                is_ok
            );
        }
        assert_that!(state.number_of_living_instances(), eq CAPACITY*2);

        drop(map);
        assert_that!(state.number_of_living_instances(), eq 0);
    }

    #[test]
    fn insert_into_empty_flat_map_works() {
        let mut map = FixedSizeFlatMap::<u8, i8, CAPACITY>::new();

        let res = map.insert(3, -4);
        assert_that!(res, is_ok);
        assert_that!(map, is_not_empty);
        assert_that!(map, len 1);
        assert_that!(map.contains(&3), eq true);
    }

    #[test]
    fn insert_the_same_key_fails() {
        let mut map = FixedSizeFlatMap::<i16, i16, CAPACITY>::new();
        let key = -2023;

        let res = map.insert(key, -9);
        assert_that!(res, is_ok);
        assert_that!(map, len 1);
        assert_that!(map.contains(&key), eq true);

        let res = map.insert(key, 19);
        assert_that!(res, is_err);
        assert_that!(map, len 1);
        assert_that!(map.contains(&key), eq true);
    }

    #[test]
    fn insert_until_full_works() {
        let mut map = FixedSizeFlatMap::<u32, u32, CAPACITY>::new();
        for i in 0..CAPACITY as u32 {
            assert_that!(map.insert(i, i), is_ok);
            assert_that!(map.contains(&i), eq true);
        }
        assert_that!(map.is_full(), eq true);
        let res = map.insert(CAPACITY as u32, CAPACITY as u32);
        assert_that!(res, is_err);
        assert_that!(res.unwrap_err(), eq FlatMapError::IsFull);
        assert_that!(map.contains(&(CAPACITY as u32)), eq false);
        assert_that!(map, len CAPACITY);
    }

    #[test]
    fn get_value_from_flat_map_works() {
        let mut map = FixedSizeFlatMap::<u8, u8, CAPACITY>::new();
        let key = 34;
        assert_that!(map.insert(key, 40), is_ok);

        let res = map.get(&key);
        assert_that!(res, is_some);
        assert_that!(res.unwrap(), eq 40);

        let res = map.get(&35);
        assert_that!(res, is_none);
    }

    #[test]
    fn get_ref_value_from_flat_map_works() {
        let mut map = FixedSizeFlatMap::<u8, u8, CAPACITY>::new();
        let key = 34;
        assert_that!(map.insert(key, 40), is_ok);

        let res = map.get_ref(&key);
        assert_that!(res, is_some);
        assert_that!(*res.unwrap(), eq 40);

        let res = map.get_ref(&35);
        assert_that!(res, is_none);
    }

    #[test]
    fn get_mut_ref_value_from_flat_map_works() {
        let mut map = FixedSizeFlatMap::<u8, u8, CAPACITY>::new();
        let key = 34;
        assert_that!(map.insert(key, 40), is_ok);

        let res = map.get_mut_ref(&key);
        assert_that!(res, is_some);

        *res.unwrap() = 41;
        let res = map.get_ref(&key);
        assert_that!(res, is_some);
        assert_that!(*res.unwrap(), eq 41);

        let res = map.get_mut_ref(&35);
        assert_that!(res, is_none);
    }

    #[test]
    fn remove_keys_from_flat_map_works() {
        let mut map = FixedSizeFlatMap::<u8, u8, CAPACITY>::new();
        assert_that!(map, is_empty);

        assert_eq!(map.remove(&0), None);
        assert_that!(map, is_empty);

        assert_that!(map.insert(1, 1), is_ok);
        assert_that!(map.contains(&1), eq true);

        assert_eq!(map.remove(&0), None);
        assert_that!(map, is_not_empty);
        assert_eq!(map.remove(&1), Some(1));
        assert_that!(map, is_empty);
        assert_that!(map.contains(&1), eq false);
    }

    #[test]
    fn remove_until_empty_and_reinsert_works() {
        let mut map = FixedSizeFlatMap::<u32, u32, CAPACITY>::new();
        // insert until full
        for i in 0..CAPACITY as u32 {
            assert_that!(map.insert(i, i), is_ok);
        }
        assert_that!(map.is_full(), eq true);

        // remove until empty
        for i in 0..CAPACITY as u32 {
            assert_eq!(map.remove(&i), Some(i));
        }
        assert_that!(map, is_empty);

        // reinsert until full
        for i in 0..CAPACITY as u32 {
            assert_that!(map.insert(i, i), is_ok);
        }
        assert_that!(map.is_full(), eq true);
    }

    #[test]
    #[should_panic]
    fn double_init_call_causes_panic() {
        const MEM_SIZE: usize = RelocatableFlatMap::<u8, u8>::const_memory_size(CAPACITY);
        let mut memory = [0u8; MEM_SIZE];
        let bump_allocator = BumpAllocator::new(memory.as_mut_ptr());

        let mut sut = unsafe { RelocatableFlatMap::<u8, u8>::new_uninit(CAPACITY) };
        unsafe { sut.init(&bump_allocator).expect("sut init failed") };

        unsafe { sut.init(&bump_allocator).expect("sut init failed") };
    }

    #[cfg(debug_assertions)]
    #[test]
    #[should_panic]
    fn panic_is_called_in_debug_mode_if_map_is_not_initialized() {
        let mut sut = unsafe { RelocatableFlatMap::<u8, u8>::new_uninit(CAPACITY) };
        unsafe { sut.remove(&1) };
    }
}
