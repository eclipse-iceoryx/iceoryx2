// Copyright (c) 2023 Contributors to the Eclipse Foundation
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

mod queue {
    use iceoryx2_bb_container::queue::*;
    use iceoryx2_bb_elementary::bump_allocator::BumpAllocator;
    use iceoryx2_bb_elementary_traits::placement_default::PlacementDefault;
    use iceoryx2_bb_testing::{assert_that, lifetime_tracker::LifetimeTracker, memory::RawMemory};

    const SUT_CAPACITY: usize = 128;
    type Sut = FixedSizeQueue<usize, SUT_CAPACITY>;

    #[test]
    fn relocatable_push_pop_works_with_uninitialized_memory() {
        let mut memory = [0u8; 1024];
        let allocator = BumpAllocator::new(memory.as_mut_ptr());

        let mut sut = unsafe { RelocatableQueue::<usize>::new_uninit(SUT_CAPACITY) };
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
            assert_that!(sut, is_not_empty);
            assert_that!(sut, len sut.capacity() - i);
            let result = unsafe { sut.pop() };
            assert_that!(sut.is_full(), eq false);
            assert_that!(result, eq Some(i * 2 + 3));
        }

        assert_that!(sut, is_empty);
        assert_that!(sut, len 0);
    }

    #[test]
    fn relocatable_clear_empties_queue() {
        let mut memory = [0u8; 1024];
        let allocator = BumpAllocator::new(memory.as_mut_ptr());

        let mut sut = unsafe { RelocatableQueue::<usize>::new_uninit(SUT_CAPACITY) };
        unsafe { assert_that!(sut.init(&allocator), is_ok) };

        for i in 0..sut.capacity() {
            assert_that!(sut.is_full(), eq false);
            assert_that!(unsafe { sut.push_with_overflow(i) }, eq None);
            assert_that!(sut, is_not_empty);
            assert_that!(sut, len i + 1);
        }
        assert_that!(sut.is_full(), eq true);
        assert_that!(unsafe { sut.push_with_overflow(123) }, eq Some(0));

        unsafe { sut.clear() };
        assert_that!(sut.is_empty(), eq true);
    }

    #[test]
    fn capacity_is_correct() {
        let sut = Sut::new();
        assert_that!(sut.capacity(), eq SUT_CAPACITY);
    }

    #[test]
    fn newly_created_buffer_is_empty() {
        let mut sut = Sut::new();
        assert_that!(sut, is_empty);
        assert_that!(sut, len 0);
        assert_that!(sut.pop(), is_none);
        assert_that!(sut.is_full(), eq false);
    }

    #[test]
    fn push_pop_works() {
        let mut sut = Sut::new();

        for i in 0..sut.capacity() {
            let element = i * 2 + 3;
            assert_that!(sut.is_full(), eq false);
            assert_that!(sut.push(element), eq true);
            assert_that!(sut, is_not_empty);
            assert_that!(sut, len i + 1);
        }
        assert_that!(sut.is_full(), eq true);

        for i in 0..sut.capacity() {
            assert_that!(sut, is_not_empty);
            assert_that!(sut, len sut.capacity() - i);
            let result = sut.pop();
            assert_that!(sut.is_full(), eq false);
            assert_that!(result, eq Some(i * 2 + 3));
        }

        assert_that!(sut, is_empty);
        assert_that!(sut, len 0);
    }

    #[test]
    fn valid_after_move() {
        let mut sut = Sut::new();

        for i in 0..sut.capacity() {
            let element = i * 2 + 3;
            assert_that!(sut.push(element), eq true);
        }

        let mut sut2 = sut;

        for i in 0..sut2.capacity() {
            let result = sut2.pop();
            assert_that!(result, eq Some(i * 2 + 3));
        }
    }

    #[test]
    fn push_pop_alteration_works() {
        let mut sut = Sut::new();

        let mut push_counter: usize = 0;
        let mut pop_counter: usize = 0;
        for _ in 0..sut.capacity() / 3 {
            for _ in 0..5 {
                let element = push_counter * 4 + 1;
                push_counter += 1;
                assert_that!(sut.push(element), eq true);
            }

            for _ in 0..3 {
                let result = sut.pop();
                assert_that!(result, eq Some(pop_counter * 4 + 1));
                pop_counter += 1;
            }
        }
    }

    #[test]
    fn clear_works() {
        let mut sut = Sut::new();

        for i in 0..sut.capacity() {
            assert_that!(sut.push(i), eq true);
        }

        sut.clear();
        assert_that!(sut, is_empty);
        assert_that!(sut, len 0);
        assert_that!(sut.pop(), is_none);
    }

    #[test]
    fn overflow_works() {
        let mut sut = Sut::new();

        for i in 0..sut.capacity() {
            let element = i;
            assert_that!(sut.push_with_overflow(element), is_none);
        }

        for i in 0..sut.capacity() {
            let element = (i + 5) * sut.capacity();
            let result = sut.push_with_overflow(element);
            assert_that!(result, eq Some(i));
        }

        for i in 0..sut.capacity() {
            let element = (i + 5) * sut.capacity();
            let result = sut.pop();
            assert_that!(result, eq Some(element));
        }
    }

    #[test]
    fn iterate_with_get() {
        let mut sut = Sut::new();

        for i in 0..sut.capacity() / 2 {
            sut.push_with_overflow(i);
        }

        for i in 0..sut.capacity() {
            sut.push_with_overflow(2 * i + 25);
        }

        for i in 0..sut.len() {
            assert_that!(sut.get(i), eq 2 * i + 25);
        }
    }

    #[test]
    fn drops_all_objects_when_out_of_scope() {
        let state = LifetimeTracker::start_tracking();
        let mut sut = FixedSizeQueue::<LifetimeTracker, SUT_CAPACITY>::new();

        for _ in 0..sut.capacity() {
            sut.push(LifetimeTracker::new());
        }

        assert_that!(state.number_of_living_instances(), eq SUT_CAPACITY);
        drop(sut);
        assert_that!(state.number_of_living_instances(), eq 0);
    }

    #[test]
    fn drops_all_objects_with_clear() {
        let state = LifetimeTracker::start_tracking();
        let mut sut = FixedSizeQueue::<LifetimeTracker, SUT_CAPACITY>::new();

        for _ in 0..sut.capacity() {
            sut.push(LifetimeTracker::new());
        }

        assert_that!(state.number_of_living_instances(), eq SUT_CAPACITY);
        sut.clear();
        assert_that!(state.number_of_living_instances(), eq 0);
    }

    #[test]
    fn pop_releases_object() {
        let state = LifetimeTracker::start_tracking();
        let mut sut = FixedSizeQueue::<LifetimeTracker, SUT_CAPACITY>::new();

        for _ in 0..sut.capacity() {
            sut.push(LifetimeTracker::new());
        }

        for i in (0..sut.capacity()).rev() {
            let result = sut.pop();
            assert_that!(result, is_some);
            drop(result);
            assert_that!(state.number_of_living_instances(), eq i);
        }
    }

    #[test]
    fn queue_clear_drops_all_objects() {
        let state = LifetimeTracker::start_tracking();
        let mut sut = Queue::<LifetimeTracker>::new(SUT_CAPACITY);

        for _ in 0..sut.capacity() {
            sut.push(LifetimeTracker::new());
        }

        sut.clear();
        assert_that!(state.number_of_living_instances(), eq 0);
    }

    #[test]
    fn fixed_size_queue_clear_drops_all_objects() {
        let state = LifetimeTracker::start_tracking();
        let mut sut = FixedSizeQueue::<LifetimeTracker, SUT_CAPACITY>::new();

        for _ in 0..sut.capacity() {
            sut.push(LifetimeTracker::new());
        }

        sut.clear();
        assert_that!(state.number_of_living_instances(), eq 0);
    }

    #[test]
    #[should_panic]
    fn get_invalid_index_panics() {
        let mut sut = FixedSizeQueue::<u64, SUT_CAPACITY>::new();
        sut.push(123);

        sut.get(2);
    }

    #[test]
    fn get_unchecked_works() {
        let mut sut = FixedSizeQueue::<usize, SUT_CAPACITY>::new();

        for i in 0..SUT_CAPACITY {
            assert_that!(sut.push(i * 3 + 2), eq true);
        }

        for i in 0..SUT_CAPACITY {
            assert_that!(unsafe { sut.get_unchecked(i) }, eq i * 3 + 2);
        }
    }

    #[test]
    fn placement_default_works() {
        type Sut = FixedSizeQueue<usize, SUT_CAPACITY>;
        let mut sut = RawMemory::<Sut>::new_filled(0xff);
        unsafe { Sut::placement_default(sut.as_mut_ptr()) };

        assert_that!(unsafe {sut.assume_init()}, len 0);
        assert_that!(unsafe {sut.assume_init_mut()}.push(123), eq true);
        assert_that!(unsafe {sut.assume_init_mut()}.push(456), eq true);

        assert_that!(unsafe {sut.assume_init_mut()}.pop(), eq Some(123));
        assert_that!(unsafe {sut.assume_init_mut()}.pop(), eq Some(456));
    }

    #[test]
    fn peek_works() {
        let mut sut = Sut::new();

        assert_that!(sut.peek(), is_none);
        assert_that!(sut.peek_mut(), is_none);

        sut.push(8781);

        assert_that!(*sut.peek().unwrap(), eq 8781);
        assert_that!(*sut.peek_mut().unwrap(), eq 8781);

        *sut.peek_mut().unwrap() = 99182;

        assert_that!(*sut.peek().unwrap(), eq 99182);
        assert_that!(*sut.peek_mut().unwrap(), eq 99182);
    }

    #[test]
    #[should_panic]
    fn double_init_call_causes_panic() {
        const MEM_SIZE: usize = RelocatableQueue::<usize>::const_memory_size(SUT_CAPACITY);
        let mut memory = [0u8; MEM_SIZE];
        let bump_allocator = BumpAllocator::new(memory.as_mut_ptr());

        let mut sut = unsafe { RelocatableQueue::<usize>::new_uninit(SUT_CAPACITY) };
        unsafe { sut.init(&bump_allocator).expect("sut init failed") };

        unsafe { sut.init(&bump_allocator).expect("sut init failed") };
    }

    #[cfg(debug_assertions)]
    #[test]
    #[should_panic]
    fn panic_is_called_in_debug_mode_if_queue_is_not_initialized() {
        let mut sut = unsafe { RelocatableQueue::<u8>::new_uninit(SUT_CAPACITY) };
        unsafe { sut.pop() };
    }
}
