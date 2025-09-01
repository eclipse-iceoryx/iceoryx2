// Copyright (c) 2023 - 2024 Contributors to the Eclipse Foundation
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

use iceoryx2_bb_container::vec::*;
use iceoryx2_bb_elementary::bump_allocator::BumpAllocator;
use iceoryx2_bb_elementary_traits::placement_default::PlacementDefault;
use iceoryx2_bb_elementary_traits::relocatable_container::RelocatableContainer;
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing::lifetime_tracker::LifetimeTracker;
use iceoryx2_bb_testing::memory::RawMemory;
use serde_test::{assert_tokens, Token};

mod fixed_size_vec {
    use super::*;

    const SUT_CAPACITY: usize = 128;
    type Sut = FixedSizeVec<usize, SUT_CAPACITY>;

    #[test]
    fn new_vector_is_empty() {
        let mut sut = Sut::new();

        assert_that!(sut, is_empty);
        assert_that!(sut.is_full(), eq false);
        assert_that!(sut, len 0);
        assert_that!(sut.pop(), is_none);
    }

    #[test]
    fn capacity_is_correct() {
        assert_that!(Sut::capacity(), eq SUT_CAPACITY);
    }

    #[test]
    fn push_pop_works() {
        let mut sut = Sut::new();

        for i in 0..Sut::capacity() {
            let element = i * 2 + 3;
            assert_that!(sut.is_full(), eq false);
            assert_that!(sut.push(element), eq true);
            assert_that!(sut, is_not_empty);
            assert_that!(sut, len i + 1);
        }
        assert_that!(sut.is_full(), eq true);

        for i in 0..Sut::capacity() {
            assert_that!(*sut.get(i).unwrap(), eq i * 2 + 3);
            assert_that!(*sut.get_mut(i).unwrap(), eq i * 2 + 3);
            assert_that!(unsafe { *sut.get_unchecked(i) }, eq i * 2 + 3);
            assert_that!(unsafe { *sut.get_unchecked_mut(i) }, eq i * 2 + 3);
        }

        for i in (0..Sut::capacity()).rev() {
            assert_that!(sut, is_not_empty);
            assert_that!(sut, len i + 1);
            let result = sut.pop();
            assert_that!(sut.is_full(), eq false);
            assert_that!(result, eq Some(i * 2 + 3));
        }

        assert_that!(sut, is_empty);
        assert_that!(sut, len 0);
    }

    #[test]
    fn vec_push_pop_works_with_uninitialized_memory() {
        let mut memory = [0u8; 1024];
        let allocator = BumpAllocator::new(memory.as_mut_ptr());
        let mut sut = unsafe { RelocatableVec::<usize>::new_uninit(SUT_CAPACITY) };
        unsafe { assert_that!(sut.init(&allocator), is_ok) };

        for i in 0..sut.capacity() {
            let element = i * 2 + 3;
            assert_that!(sut.is_full(), eq false);
            assert_that!(unsafe { sut.push(element) }, eq true);
            assert_that!(sut, is_not_empty);
            assert_that!(sut, len i + 1);
        }
        assert_that!(sut.is_full(), eq true);

        for i in 0..sut.capacity() {
            assert_that!(*sut.get(i).unwrap(), eq i * 2 + 3);
            assert_that!(*sut.get_mut(i).unwrap(), eq i * 2 + 3);
            assert_that!(unsafe { *sut.get_unchecked(i) }, eq i * 2 + 3);
            assert_that!(unsafe { *sut.get_unchecked_mut(i) }, eq i * 2 + 3);
        }

        for i in 0..sut.capacity() {
            assert_that!(sut, is_not_empty);
            assert_that!(sut, len sut.capacity() - i);
            let result = unsafe { sut.pop() };
            assert_that!(sut.is_full(), eq false);
            assert_that!(result, eq Some((sut.capacity() - i - 1) * 2 + 3));
        }

        assert_that!(sut, is_empty);
        assert_that!(sut, len 0);
    }

    #[test]
    fn clear_works() {
        let mut sut = Sut::new();

        for i in 0..Sut::capacity() {
            assert_that!(sut.push(i), eq true);
        }

        sut.clear();
        assert_that!(sut, is_empty);
        assert_that!(sut, len 0);
        assert_that!(sut.pop(), is_none);
    }

    #[test]
    fn push_pop_alteration_works() {
        let mut sut = Sut::new();

        let mut push_counter: usize = 0;
        for _ in 0..Sut::capacity() / 3 {
            for _ in 0..5 {
                let element = push_counter * 4 + 1;
                push_counter += 1;
                assert_that!(sut.push(element), eq true);
            }

            for i in 0..3 {
                let result = sut.pop();
                assert_that!(result, eq Some((push_counter - i - 1) * 4 + 1));
            }
        }
    }

    #[test]
    fn valid_after_move() {
        let mut sut = Sut::new();

        for i in 0..Sut::capacity() {
            let element = i * 2 + 3;
            assert_that!(sut.push(element), eq true);
        }

        let mut sut2 = sut;

        for i in 0..Sut::capacity() {
            let result = sut2.pop();
            assert_that!(result, eq Some((Sut::capacity() - i - 1) * 2 + 3));
        }
    }

    #[test]
    fn eq_works() {
        let create_vec = |n| {
            let mut sut = Sut::new();
            for i in 0..n {
                sut.push(4 * i + 3);
            }
            sut
        };

        let vec1 = create_vec(SUT_CAPACITY - 2);
        let vec2 = create_vec(SUT_CAPACITY - 1);
        let vec3 = create_vec(SUT_CAPACITY);

        assert_that!(Sut::new() == Sut::new(), eq true);

        assert_that!(vec1 == vec1, eq true);
        assert_that!(vec1 == vec2, eq false);
        assert_that!(vec1 == vec3, eq false);
        assert_that!(vec1 == Sut::new(), eq false);

        assert_that!(vec2 == vec1, eq false);
        assert_that!(vec2 == vec2, eq true);
        assert_that!(vec2 == vec3, eq false);
        assert_that!(vec2 == Sut::new(), eq false);

        assert_that!(vec3 == vec1, eq false);
        assert_that!(vec3 == vec2, eq false);
        assert_that!(vec3 == vec3, eq true);
        assert_that!(vec3 == Sut::new(), eq false);
    }

    #[test]
    fn clone_works() {
        let mut sut = Sut::new();
        let sut1 = sut.clone();
        for i in 0..SUT_CAPACITY {
            sut.push(8 * i + 6);
        }

        let sut2 = sut.clone();

        assert_that!(Sut::new() == sut1, eq true);
        assert_that!(sut == sut2, eq true);
    }

    #[test]
    fn drops_all_objects_when_out_of_scope() {
        let state = LifetimeTracker::start_tracking();
        let mut sut = FixedSizeVec::<LifetimeTracker, SUT_CAPACITY>::new();

        for _ in 0..SUT_CAPACITY {
            assert_that!(sut.push(LifetimeTracker::new()), eq true);
        }

        assert_that!(state.number_of_living_instances(), eq SUT_CAPACITY);
        drop(sut);
        assert_that!(state.number_of_living_instances(), eq 0);
    }

    #[test]
    fn drops_all_objects_with_clear() {
        let state = LifetimeTracker::start_tracking();
        let mut sut = FixedSizeVec::<LifetimeTracker, SUT_CAPACITY>::new();

        for _ in 0..SUT_CAPACITY {
            assert_that!(sut.push(LifetimeTracker::new()), eq true);
        }

        assert_that!(state.number_of_living_instances(), eq SUT_CAPACITY);
        sut.clear();
        assert_that!(state.number_of_living_instances(), eq 0);
    }

    #[test]
    fn pop_releases_ownership() {
        let state = LifetimeTracker::start_tracking();
        let mut sut = FixedSizeVec::<LifetimeTracker, SUT_CAPACITY>::new();

        for _ in 0..SUT_CAPACITY {
            assert_that!(sut.push(LifetimeTracker::new()), eq true);
        }

        for i in (0..SUT_CAPACITY).rev() {
            let result = sut.pop();
            assert_that!(result, is_some);
            drop(result);
            assert_that!(state.number_of_living_instances(), eq i);
        }
    }

    #[test]
    fn placement_default_works() {
        type Sut = FixedSizeVec<usize, SUT_CAPACITY>;
        let mut sut = RawMemory::<Sut>::new_filled(0xff);
        unsafe { Sut::placement_default(sut.as_mut_ptr()) };

        assert_that!(unsafe { sut.assume_init()}, len 0);
        assert_that!(unsafe { sut.assume_init_mut()}.push(123), eq true);
        assert_that!(unsafe { sut.assume_init_mut()}.push(456), eq true);

        assert_that!(unsafe { sut.assume_init_mut()}.pop(), eq Some(456));
        assert_that!(unsafe { sut.assume_init_mut()}.pop(), eq Some(123));
    }

    #[test]
    fn as_slice_works() {
        let mut sut = Sut::new();
        for i in 0..12 {
            sut.push(2 * i * i + 3);
        }

        for (i, element) in sut.as_slice().iter().enumerate() {
            assert_that!(*element, eq 2 * i * i + 3);
        }
    }

    #[test]
    fn as_mut_slice_works() {
        let mut sut = Sut::new();
        for i in 0..12 {
            sut.push(3 + 2 * i);
        }

        for (i, element) in sut.as_mut_slice().iter_mut().enumerate() {
            assert_that!(*element, eq 3 + 2 * i);
            *element = 3 * i;
        }

        for (i, element) in sut.iter().enumerate() {
            assert_that!(*element, eq 3 * i);
        }
    }

    #[test]
    fn serialization_works() {
        let mut sut = Sut::new();
        sut.push(44617);
        sut.push(123123);
        sut.push(89712);
        sut.push(99101);

        assert_tokens(
            &sut,
            &[
                Token::Seq { len: Some(4) },
                Token::U64(44617),
                Token::U64(123123),
                Token::U64(89712),
                Token::U64(99101),
                Token::SeqEnd,
            ],
        );
    }
}

mod vec {
    use super::*;

    #[test]
    fn push_and_pop_element_works() {
        const CAPACITY: usize = 12;
        const TEST_VALUE: usize = 89123;
        let mut sut = Vec::<usize>::new(CAPACITY);
        assert_that!(sut.capacity(), eq CAPACITY);
        assert_that!(sut, len 0);

        sut.push(TEST_VALUE);

        assert_that!(sut, len 1);
        assert_that!(sut[0], eq TEST_VALUE);
        assert_that!(sut.pop(), eq Some(TEST_VALUE));
        assert_that!(sut, len 0);
    }

    #[test]
    fn remove_reverse_order_works() {
        const CAPACITY: usize = 12;
        let mut sut = Vec::<usize>::new(CAPACITY);

        for n in 0..CAPACITY {
            sut.push(n);
        }

        for n in (0..CAPACITY).rev() {
            assert_that!(sut.remove(n), eq n);

            for (idx, v) in sut.iter().enumerate() {
                assert_that!(*v, eq idx);
            }
        }
    }

    #[test]
    fn remove_works() {
        const CAPACITY: usize = 12;
        let mut sut = Vec::<usize>::new(CAPACITY);

        for n in 0..CAPACITY {
            sut.push(n);
        }

        for n in 0..CAPACITY {
            assert_that!(sut.remove(0), eq n);

            for (idx, v) in sut.iter().enumerate() {
                assert_that!(*v, eq idx + n + 1);
            }
        }
    }
}

mod relocatable_vec {
    use super::*;

    #[test]
    #[should_panic]
    fn double_init_call_causes_panic() {
        const CAPACITY: usize = 12;
        const MEM_SIZE: usize = RelocatableVec::<u128>::const_memory_size(CAPACITY);
        let mut memory = [0u8; MEM_SIZE];

        let bump_allocator = BumpAllocator::new(memory.as_mut_ptr());

        let mut sut = unsafe { RelocatableVec::<u128>::new_uninit(CAPACITY) };
        unsafe { sut.init(&bump_allocator).expect("sut init failed") };

        unsafe { sut.init(&bump_allocator).expect("sut init failed") };
    }

    #[cfg(debug_assertions)]
    #[test]
    #[should_panic]
    fn panic_is_called_in_debug_mode_if_vec_is_not_initialized() {
        const CAPACITY: usize = 12;
        let mut sut = unsafe { RelocatableVec::<u8>::new_uninit(CAPACITY) };
        unsafe { sut.remove(0) };
    }
}
