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
mod event_id_tracker {
    use std::collections::HashSet;

    use iceoryx2_bb_lock_free::mpmc::bit_set::RelocatableBitSet;
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_cal::event::{id_tracker::IdTracker, TriggerId};

    use iceoryx2_bb_memory::bump_allocator::*;
    use iceoryx2_bb_memory::memory::Memory;
    use pin_init::PtrPinWith;

    const MEMORY_SIZE: usize = 1024 * 1024;

    #[test]
    fn max_trigger_id_must_be_at_least_capacity<Sut: IdTracker>() {
        let memory = Box::pin_with(Memory::<MEMORY_SIZE, BumpAllocator>::new()).unwrap();
        const CAPACITY: usize = 5234;

        let sut = unsafe { Sut::new_uninit(CAPACITY) };
        assert_that!(unsafe { sut.init(memory.allocator()) }, is_ok);
        assert_that!(sut.trigger_id_max().as_value(), lt CAPACITY);
    }

    #[test]
    fn add_and_acquire_works<Sut: IdTracker>() {
        let memory = Box::pin_with(Memory::<MEMORY_SIZE, BumpAllocator>::new()).unwrap();
        const CAPACITY: usize = 1234;

        let sut = unsafe { Sut::new_uninit(CAPACITY) };
        assert_that!(unsafe { sut.init(memory.allocator()) }, is_ok);

        assert_that!(unsafe { sut.acquire() }, eq None);
        for i in 0..CAPACITY {
            let id = TriggerId::new(i);
            assert_that!(unsafe { sut.add(id) }, is_ok);
            assert_that!(unsafe { sut.acquire() }, eq Some(id));
            assert_that!(unsafe { sut.acquire() }, is_none);
        }
    }

    #[test]
    fn add_until_full_and_then_acquire_works<Sut: IdTracker>() {
        let memory = Box::pin_with(Memory::<MEMORY_SIZE, BumpAllocator>::new()).unwrap();
        const CAPACITY: usize = 1234;

        let sut = unsafe { Sut::new_uninit(CAPACITY) };
        assert_that!(unsafe { sut.init(memory.allocator()) }, is_ok);

        for i in 0..CAPACITY {
            let id = TriggerId::new((i).min(sut.trigger_id_max().as_value()));
            assert_that!(unsafe { sut.add(id) }, is_ok);
        }

        let mut ids = HashSet::new();
        for _ in 0..CAPACITY {
            let result = unsafe { sut.acquire().unwrap() };
            assert_that!(result, le sut.trigger_id_max());
            assert_that!(ids.insert(result), eq true);
        }

        assert_that!(unsafe { sut.acquire() }, is_none);
    }

    #[test]
    fn add_and_acquire_all_works<Sut: IdTracker>() {
        let memory = Box::pin_with(Memory::<MEMORY_SIZE, BumpAllocator>::new()).unwrap();
        const CAPACITY: usize = 3234;

        let sut = unsafe { Sut::new_uninit(CAPACITY) };
        assert_that!(unsafe { sut.init(memory.allocator()) }, is_ok);

        for i in 0..CAPACITY {
            let id = TriggerId::new((i).min(sut.trigger_id_max().as_value()));
            assert_that!(unsafe { sut.add(id) }, is_ok);
        }

        let mut ids = HashSet::new();
        unsafe {
            sut.acquire_all(|id| {
                assert_that!(id, le sut.trigger_id_max());
                assert_that!(ids.insert(id), eq true);
            })
        };

        let mut callback_called = false;
        unsafe { sut.acquire_all(|_| callback_called = true) };
        assert_that!(callback_called, eq false);

        assert_that!(ids, len CAPACITY);
    }

    #[test]
    fn add_acquire_and_acquire_all_works<Sut: IdTracker>() {
        let memory = Box::pin_with(Memory::<MEMORY_SIZE, BumpAllocator>::new()).unwrap();
        const CAPACITY: usize = 234;

        let sut = unsafe { Sut::new_uninit(CAPACITY) };
        assert_that!(unsafe { sut.init(memory.allocator()) }, is_ok);

        for i in 0..CAPACITY {
            let id = TriggerId::new(i.min(sut.trigger_id_max().as_value()));
            assert_that!(unsafe { sut.add(id) }, is_ok);
        }

        let mut ids = HashSet::new();
        for _ in 0..CAPACITY / 2 {
            let result = unsafe { sut.acquire().unwrap() };
            assert_that!(result, le sut.trigger_id_max());
            assert_that!(ids.insert(result), eq true);
        }

        unsafe {
            sut.acquire_all(|id| {
                assert_that!(id, le sut.trigger_id_max());
                assert_that!(ids.insert(id), eq true);
            })
        };

        assert_that!(ids, len CAPACITY);
    }

    #[test]
    fn add_max_trigger_id_and_acquire_works<Sut: IdTracker>() {
        let memory = Box::pin_with(Memory::<MEMORY_SIZE, BumpAllocator>::new()).unwrap();
        const CAPACITY: usize = 1234;

        let sut = unsafe { Sut::new_uninit(CAPACITY) };
        assert_that!(unsafe { sut.init(memory.allocator()) }, is_ok);

        assert_that!(unsafe { sut.acquire() }, eq None);
        let id = sut.trigger_id_max();
        assert_that!(unsafe { sut.add(id) }, is_ok);
        assert_that!(unsafe { sut.acquire() }, eq Some(id));
        assert_that!(unsafe { sut.acquire() }, is_none);
    }

    #[instantiate_tests(<RelocatableBitSet>)]
    mod bitset {}
}
