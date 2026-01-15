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

#[generic_tests::define]
mod vector {

    extern crate iceoryx2_bb_loggers;

    use iceoryx2_bb_concurrency::cell::UnsafeCell;
    use iceoryx2_bb_container::vector::*;
    use iceoryx2_bb_elementary::bump_allocator::BumpAllocator;
    use iceoryx2_bb_testing::{assert_that, lifetime_tracker::LifetimeTracker};

    const SUT_CAPACITY: usize = 10;

    trait VectorTestFactory {
        type Sut: Vector<LifetimeTracker>;

        fn new() -> Self;
        fn create_sut(&self) -> Box<Self::Sut>;
    }

    struct StaticVecFactory {}

    impl VectorTestFactory for StaticVecFactory {
        type Sut = StaticVec<LifetimeTracker, SUT_CAPACITY>;

        fn new() -> Self {
            Self {}
        }

        fn create_sut(&self) -> Box<Self::Sut> {
            Box::new(Self::Sut::new())
        }
    }

    struct RelocatableVecFactory {
        raw_memory: UnsafeCell<
            Box<[u8; RelocatableVec::<LifetimeTracker>::const_memory_size(SUT_CAPACITY)]>,
        >,
    }

    impl VectorTestFactory for RelocatableVecFactory {
        type Sut = RelocatableVec<LifetimeTracker>;

        fn new() -> Self {
            Self {
                raw_memory: UnsafeCell::new(Box::new(
                    [0u8; RelocatableVec::<LifetimeTracker>::const_memory_size(SUT_CAPACITY)],
                )),
            }
        }

        fn create_sut(&self) -> Box<Self::Sut> {
            let mut sut = Box::new(unsafe { Self::Sut::new_uninit(SUT_CAPACITY) });
            let bump_allocator =
                BumpAllocator::new(unsafe { &mut *self.raw_memory.get() }.as_mut_ptr());
            unsafe { sut.init(&bump_allocator).unwrap() };

            sut
        }
    }

    struct PolymorphicVecFactory {
        raw_memory: UnsafeCell<Box<[u8; core::mem::size_of::<LifetimeTracker>() * SUT_CAPACITY]>>,
        allocator: UnsafeCell<Option<Box<BumpAllocator>>>,
    }

    impl VectorTestFactory for PolymorphicVecFactory {
        type Sut = PolymorphicVec<'static, LifetimeTracker, BumpAllocator>;

        fn new() -> Self {
            Self {
                raw_memory: UnsafeCell::new(Box::new(
                    [0u8; core::mem::size_of::<LifetimeTracker>() * SUT_CAPACITY],
                )),
                allocator: UnsafeCell::new(None),
            }
        }

        fn create_sut(&self) -> Box<Self::Sut> {
            unsafe {
                if (*self.allocator.get()).is_none() {
                    *self.allocator.get() = Some(Box::new(BumpAllocator::new(
                        (*self.raw_memory.get()).as_mut_ptr(),
                    )))
                }
            };

            Box::new(
                Self::Sut::new(
                    unsafe { (*self.allocator.get()).as_ref().unwrap() },
                    SUT_CAPACITY,
                )
                .unwrap(),
            )
        }
    }

    #[test]
    fn new_created_vec_is_empty<Factory: VectorTestFactory>() {
        let factory = Factory::new();
        let sut = factory.create_sut();

        assert_that!(sut.is_empty(), eq true);
        assert_that!(sut.len(), eq 0);
        assert_that!(sut.is_full(), eq false);
    }

    #[test]
    fn push_adds_element_at_the_end<Factory: VectorTestFactory>() {
        let factory = Factory::new();
        let mut sut = factory.create_sut();

        for number in 0..SUT_CAPACITY {
            assert_that!(sut.push(LifetimeTracker::new_with_value(number)), is_ok);

            for idx in 0..number + 1 {
                assert_that!(sut[idx].value, eq idx);
            }
        }
    }

    #[test]
    fn push_until_full_works<Factory: VectorTestFactory>() {
        let factory = Factory::new();
        let mut sut = factory.create_sut();

        assert_that!(sut.is_empty(), eq true);
        assert_that!(sut.is_full(), eq false);
        for n in 0..SUT_CAPACITY {
            assert_that!(sut.len(), eq n);
            assert_that!(sut.is_full(), eq false);

            assert_that!(sut.push(LifetimeTracker::new()), is_ok);

            assert_that!(sut.is_empty(), eq false);
        }

        assert_that!(sut.is_full(), eq true);
    }

    #[test]
    fn push_more_elements_than_capacity_fails<Factory: VectorTestFactory>() {
        let factory = Factory::new();
        let mut sut = factory.create_sut();

        for number in 0..SUT_CAPACITY {
            assert_that!(sut.push(LifetimeTracker::new_with_value(2 * number)), is_ok);
        }

        for number in 0..SUT_CAPACITY {
            assert_that!(sut.push(LifetimeTracker::new_with_value(number)), eq Err(VectorModificationError::InsertWouldExceedCapacity));

            for idx in 0..SUT_CAPACITY {
                assert_that!(sut[idx].value, eq 2 * idx);
            }
        }
    }

    #[test]
    fn push_pop_alteration_works<Factory: VectorTestFactory>() {
        let factory = Factory::new();
        let mut sut = factory.create_sut();

        let mut push_counter: usize = 0;
        for _ in 0..sut.capacity() / 3 {
            for _ in 0..5 {
                let element = push_counter * 4 + 1;
                push_counter += 1;
                assert_that!(sut.push(LifetimeTracker::new_with_value(element)), is_ok);
            }

            for i in 0..3 {
                let result = sut.pop();
                assert_that!(result.unwrap().value, eq(push_counter - i - 1) * 4 + 1);
            }
        }
    }

    #[test]
    fn pop_returns_none_when_empty<Factory: VectorTestFactory>() {
        let factory = Factory::new();
        let mut sut = factory.create_sut();

        assert_that!(sut.pop(), is_none);
    }

    #[test]
    fn pop_removes_last_element<Factory: VectorTestFactory>() {
        let factory = Factory::new();
        let mut sut = factory.create_sut();

        for number in 0..SUT_CAPACITY {
            assert_that!(
                sut.push(LifetimeTracker::new_with_value(4 * number + 1)),
                is_ok
            );
        }

        for number in (0..SUT_CAPACITY).rev() {
            let value = sut.pop();
            assert_that!(value, is_some);
            assert_that!(value.unwrap().value, eq 4 * number + 1);
            assert_that!(sut.len(), eq number);
        }

        assert_that!(sut.pop(), is_none);
    }

    #[test]
    fn truncate_does_nothing_when_new_len_is_larger_than_current_len<Factory: VectorTestFactory>() {
        let half_capacity = SUT_CAPACITY / 2;
        let factory = Factory::new();
        let mut sut = factory.create_sut();

        for number in 0..half_capacity {
            assert_that!(
                sut.push(LifetimeTracker::new_with_value(4 * number + 3)),
                is_ok
            );
        }

        sut.truncate(SUT_CAPACITY);

        for idx in 0..half_capacity {
            assert_that!(sut[idx].value, eq 4 * idx + 3);
        }
    }

    #[test]
    fn truncate_drops_all_elements_right_of_new_len<Factory: VectorTestFactory>() {
        let half_capacity = SUT_CAPACITY / 2;
        let tracker = LifetimeTracker::start_tracking();
        let factory = Factory::new();
        let mut sut = factory.create_sut();

        for number in 0..SUT_CAPACITY {
            assert_that!(
                sut.push(LifetimeTracker::new_with_value(5 * number + 7)),
                is_ok
            );
        }
        assert_that!(tracker.number_of_living_instances(), eq SUT_CAPACITY);

        sut.truncate(half_capacity);
        assert_that!(sut.len(), eq half_capacity);
        assert_that!(tracker.number_of_living_instances(), eq half_capacity);

        for idx in 0..half_capacity {
            assert_that!(sut[idx].value, eq 5 * idx + 7);
        }
    }

    #[test]
    fn truncate_drops_elements_in_reverse_order<Factory: VectorTestFactory>() {
        let half_capacity = SUT_CAPACITY / 2;
        let tracker = LifetimeTracker::start_tracking();
        let factory = Factory::new();
        let mut sut = factory.create_sut();

        for number in 0..SUT_CAPACITY {
            assert_that!(sut.push(LifetimeTracker::new_with_value(number)), is_ok);
        }

        sut.truncate(half_capacity);

        for (n, drop_value) in tracker.drop_order().iter().enumerate() {
            assert_that!(*drop_value, eq SUT_CAPACITY - n - 1);
        }
    }

    #[test]
    fn resize_increases_len_with_provided_value<Factory: VectorTestFactory>() {
        let half_capacity = SUT_CAPACITY / 2;
        const TEST_VALUE: usize = 871828;
        let factory = Factory::new();
        let mut sut = factory.create_sut();

        assert_that!(
            sut.resize(half_capacity, LifetimeTracker::new_with_value(TEST_VALUE)),
            is_ok
        );

        assert_that!(sut.len(), eq half_capacity);

        for element in sut.iter() {
            assert_that!(element.value, eq TEST_VALUE);
        }
    }

    #[test]
    fn resize_reduces_len_and_drops_element_in_reverse_order<Factory: VectorTestFactory>() {
        let half_capacity = SUT_CAPACITY / 2;
        let tracker = LifetimeTracker::start_tracking();
        let factory = Factory::new();
        let mut sut = factory.create_sut();

        for number in 0..SUT_CAPACITY {
            assert_that!(sut.push(LifetimeTracker::new_with_value(number)), is_ok);
        }

        let stub_element = LifetimeTracker::new_with_value(0);
        assert_that!(sut.resize(half_capacity, stub_element), is_ok);
        assert_that!(tracker.number_of_living_instances(), eq half_capacity);

        assert_that!(sut.len(), eq half_capacity);

        for (n, element) in sut.iter().enumerate() {
            assert_that!(element.value, eq n);
        }

        let mut drop_order = tracker.drop_order();
        // first element is the dropped stub_element, skip it
        drop_order.pop();

        for (n, dropped_element) in drop_order.iter().enumerate() {
            assert_that!(*dropped_element, eq SUT_CAPACITY - n - 1);
        }
    }

    #[test]
    fn resize_fails_if_len_greater_than_capacity<Factory: VectorTestFactory>() {
        let factory = Factory::new();
        let mut sut = factory.create_sut();
        assert_that!(sut.resize(SUT_CAPACITY + 1, LifetimeTracker::new()), eq Err(VectorModificationError::InsertWouldExceedCapacity));
    }

    #[test]
    fn resize_with_increases_len_with_provided_value<Factory: VectorTestFactory>() {
        let half_capacity = SUT_CAPACITY / 2;
        const TEST_VALUE: usize = 918293;
        let factory = Factory::new();
        let mut sut = factory.create_sut();

        assert_that!(
            sut.resize_with(half_capacity, || {
                LifetimeTracker::new_with_value(TEST_VALUE)
            }),
            is_ok
        );

        assert_that!(sut.len(), eq half_capacity);

        for element in sut.iter() {
            assert_that!(element.value, eq TEST_VALUE);
        }
    }

    #[test]
    fn resize_with_reduces_len_and_drops_element_in_reverse_order<Factory: VectorTestFactory>() {
        let half_capacity = SUT_CAPACITY / 2;
        let tracker = LifetimeTracker::start_tracking();
        let factory = Factory::new();
        let mut sut = factory.create_sut();

        for number in 0..SUT_CAPACITY {
            assert_that!(sut.push(LifetimeTracker::new_with_value(number)), is_ok);
        }

        assert_that!(
            sut.resize_with(half_capacity, || LifetimeTracker::new()),
            is_ok
        );
        assert_that!(tracker.number_of_living_instances(), eq half_capacity);

        assert_that!(sut.len(), eq half_capacity);

        for (n, element) in sut.iter().enumerate() {
            assert_that!(element.value, eq n);
        }

        for (n, dropped_element) in tracker.drop_order().iter().enumerate() {
            assert_that!(*dropped_element, eq SUT_CAPACITY - n - 1);
        }
    }

    #[test]
    fn resize_with_calls_callback_only_for_the_newly_inserted_elements<
        Factory: VectorTestFactory,
    >() {
        let half_capacity = SUT_CAPACITY / 2;
        let factory = Factory::new();
        let mut sut = factory.create_sut();

        let mut counter = 0;
        assert_that!(
            sut.resize_with(half_capacity, || {
                counter += 1;
                LifetimeTracker::new()
            }),
            is_ok
        );

        assert_that!(counter, eq half_capacity);
    }

    #[test]
    fn resize_with_fails_if_len_greater_than_capacity<Factory: VectorTestFactory>() {
        let factory = Factory::new();
        let mut sut = factory.create_sut();
        assert_that!(sut.resize_with(SUT_CAPACITY + 1, || LifetimeTracker::new()), eq Err(VectorModificationError::InsertWouldExceedCapacity));
    }

    #[test]
    fn remove_first_element_of_empty_vec_returns_none<Factory: VectorTestFactory>() {
        let factory = Factory::new();
        let mut sut = factory.create_sut();
        assert_that!(sut.remove(0), is_none);
    }

    #[test]
    fn remove_element_out_of_bounds_returns_none<Factory: VectorTestFactory>() {
        let half_capacity = SUT_CAPACITY / 2;
        let factory = Factory::new();
        let mut sut = factory.create_sut();

        for _ in 0..half_capacity {
            assert_that!(sut.push(LifetimeTracker::new()), is_ok);
        }

        assert_that!(sut.remove(half_capacity), is_none);
    }

    #[test]
    fn remove_first_element_until_empty_works<Factory: VectorTestFactory>() {
        let tracker = LifetimeTracker::start_tracking();
        let factory = Factory::new();
        let mut sut = factory.create_sut();

        for number in 0..SUT_CAPACITY {
            assert_that!(
                sut.push(LifetimeTracker::new_with_value(number * 13)),
                is_ok
            );
        }

        for number in 0..SUT_CAPACITY {
            for k in 0..SUT_CAPACITY - number {
                assert_that!(sut[k].value, eq((k + number) * 13));
            }
            assert_that!(sut.remove(0).unwrap().value, eq number * 13);
            assert_that!(sut.len(), eq SUT_CAPACITY - number - 1);
        }

        assert_that!(sut.is_empty(), eq true);
        assert_that!(tracker.number_of_living_instances(), eq 0);
    }

    #[test]
    fn remove_middle_element_works<Factory: VectorTestFactory>() {
        let half_capacity = SUT_CAPACITY / 2;
        let tracker = LifetimeTracker::start_tracking();
        let factory = Factory::new();
        let mut sut = factory.create_sut();

        for number in 0..SUT_CAPACITY {
            assert_that!(
                sut.push(LifetimeTracker::new_with_value(number * 17)),
                is_ok
            );
        }

        assert_that!(sut.remove(half_capacity).unwrap().value, eq half_capacity * 17 );
        assert_that!(sut.len(), eq SUT_CAPACITY - 1);
        assert_that!(tracker.number_of_living_instances(), eq SUT_CAPACITY - 1);

        for number in 0..half_capacity - 1 {
            assert_that!(sut[number].value, eq number * 17);
        }

        for number in half_capacity..SUT_CAPACITY - 1 {
            assert_that!(sut[number].value, eq(number + 1) * 17);
        }
    }

    #[test]
    fn insert_first_element_of_empty_vec_works<Factory: VectorTestFactory>() {
        const TEST_VALUE: usize = 91782389;
        let factory = Factory::new();
        let mut sut = factory.create_sut();
        assert_that!(
            sut.insert(0, LifetimeTracker::new_with_value(TEST_VALUE)),
            is_ok
        );

        assert_that!(sut[0].value, eq TEST_VALUE);
    }

    #[test]
    fn insert_second_element_of_empty_vec_fails<Factory: VectorTestFactory>() {
        let factory = Factory::new();
        let mut sut = factory.create_sut();
        assert_that!(sut.insert(1, LifetimeTracker::new()), eq Err(VectorModificationError::OutOfBounds));
    }

    #[test]
    fn insert_at_position_zero_fills_vector_in_reverse_order<Factory: VectorTestFactory>() {
        let factory = Factory::new();
        let mut sut = factory.create_sut();

        for number in 0..SUT_CAPACITY {
            assert_that!(
                sut.insert(0, LifetimeTracker::new_with_value(number)),
                is_ok
            );
            assert_that!(sut.len(), eq number + 1);
        }

        for (n, element) in sut.iter().enumerate() {
            assert_that!(element.value, eq SUT_CAPACITY - n - 1);
        }
    }

    #[test]
    fn insert_at_end_fills_vector_in_order<Factory: VectorTestFactory>() {
        let factory = Factory::new();
        let mut sut = factory.create_sut();

        for number in 0..SUT_CAPACITY {
            assert_that!(
                sut.insert(number, LifetimeTracker::new_with_value(number)),
                is_ok
            );
            assert_that!(sut.len(), eq number + 1);
        }

        for (n, element) in sut.iter().enumerate() {
            assert_that!(element.value, eq n);
        }
    }

    #[test]
    fn insert_at_center_move_elements_to_the_rights<Factory: VectorTestFactory>() {
        let half_capacity = SUT_CAPACITY / 2;
        const TEST_VALUE: usize = 565612334;
        let factory = Factory::new();
        let mut sut = factory.create_sut();

        for number in 0..SUT_CAPACITY - 1 {
            assert_that!(sut.push(LifetimeTracker::new_with_value(number)), is_ok);
        }

        assert_that!(
            sut.insert(half_capacity, LifetimeTracker::new_with_value(TEST_VALUE)),
            is_ok
        );
        assert_that!(sut.len(), eq SUT_CAPACITY);

        for number in 0..half_capacity {
            assert_that!(sut[number].value, eq number);
        }

        assert_that!(sut[half_capacity].value, eq TEST_VALUE);

        for number in half_capacity + 1..SUT_CAPACITY {
            assert_that!(sut[number].value, eq number - 1);
        }
    }

    #[test]
    fn insert_into_full_vec_fails<Factory: VectorTestFactory>() {
        let factory = Factory::new();
        let mut sut = factory.create_sut();

        for _ in 0..SUT_CAPACITY {
            assert_that!(sut.push(LifetimeTracker::new()), is_ok);
        }

        assert_that!(sut.insert(0, LifetimeTracker::new()), eq Err(VectorModificationError::InsertWouldExceedCapacity));
    }

    #[test]
    fn clearing_empty_vector_does_nothing<Factory: VectorTestFactory>() {
        let factory = Factory::new();
        let mut sut = factory.create_sut();

        sut.clear();
        assert_that!(sut.is_empty(), eq true);
    }

    #[test]
    fn clear_drops_elements_in_reverse_order<Factory: VectorTestFactory>() {
        let tracker = LifetimeTracker::start_tracking();
        let factory = Factory::new();
        let mut sut = factory.create_sut();

        for number in 0..SUT_CAPACITY {
            assert_that!(sut.push(LifetimeTracker::new_with_value(number)), is_ok);
        }

        sut.clear();

        assert_that!(sut.is_empty(), eq true);
        assert_that!(sut.len(), eq 0);
        assert_that!(sut.is_full(), eq false);

        for (n, element) in tracker.drop_order().iter().rev().enumerate() {
            assert_that!(*element, eq n);
        }
    }

    #[test]
    fn as_slice_contains_elements<Factory: VectorTestFactory>() {
        let factory = Factory::new();
        let mut sut = factory.create_sut();

        for number in 0..SUT_CAPACITY {
            assert_that!(sut.push(LifetimeTracker::new_with_value(number)), is_ok);
        }

        for (n, element) in sut.as_slice().iter().enumerate() {
            assert_that!(element.value, eq n);
        }
    }

    #[test]
    fn as_mut_slice_contains_mutable_elements<Factory: VectorTestFactory>() {
        let factory = Factory::new();
        let mut sut = factory.create_sut();

        for number in 0..SUT_CAPACITY {
            assert_that!(sut.push(LifetimeTracker::new_with_value(number)), is_ok);
        }

        for element in sut.as_mut_slice() {
            element.value *= 2
        }

        for number in 0..SUT_CAPACITY {
            assert_that!(sut[number].value, eq number * 2);
        }
    }

    #[test]
    fn adding_a_slice_to_empty_vec_that_exceeds_the_capacity_fails<Factory: VectorTestFactory>() {
        let factory = Factory::new();
        let mut sut = factory.create_sut();

        assert_that!(sut.extend_from_slice(&core::array::from_fn::<LifetimeTracker, {SUT_CAPACITY + 1}, _>(|_| LifetimeTracker::new())), eq Err(VectorModificationError::InsertWouldExceedCapacity));
    }

    #[test]
    fn adding_a_slice_to_empty_vec_that_has_the_same_capacity_works<Factory: VectorTestFactory>() {
        const TEST_VALUE: usize = 819212;
        let factory = Factory::new();
        let mut sut = factory.create_sut();

        assert_that!(
            sut.extend_from_slice(&core::array::from_fn::<LifetimeTracker, SUT_CAPACITY, _>(
                |_| LifetimeTracker::new_with_value(TEST_VALUE)
            )),
            is_ok
        );
        for value in sut.iter() {
            assert_that!(value.value, eq TEST_VALUE);
        }
    }

    #[test]
    fn adding_a_slice_to_filled_vec_appends_elements<Factory: VectorTestFactory>() {
        const HALF_CAPACITY: usize = SUT_CAPACITY / 2;
        const TEST_VALUE: usize = 9102;
        let factory = Factory::new();
        let mut sut = factory.create_sut();

        for n in 0..HALF_CAPACITY {
            assert_that!(sut.push(LifetimeTracker::new_with_value(n)), is_ok);
        }

        assert_that!(
            sut.extend_from_slice(&core::array::from_fn::<LifetimeTracker, HALF_CAPACITY, _>(
                |_| LifetimeTracker::new_with_value(TEST_VALUE)
            )),
            is_ok
        );
        for n in 0..HALF_CAPACITY {
            assert_that!(sut[n].value, eq n);
        }

        for n in HALF_CAPACITY..SUT_CAPACITY {
            assert_that!(sut[n].value, eq TEST_VALUE);
        }
    }

    #[test]
    fn when_vec_is_dropped_all_elements_are_dropped_in_reverse_order<Factory: VectorTestFactory>() {
        let tracker = LifetimeTracker::start_tracking();
        let factory = Factory::new();
        let mut sut = factory.create_sut();

        for number in 0..SUT_CAPACITY {
            assert_that!(sut.push(LifetimeTracker::new_with_value(number)), is_ok);
        }

        drop(sut);

        assert_that!(tracker.number_of_living_instances(), eq 0);
        for (n, element) in tracker.drop_order().iter().rev().enumerate() {
            assert_that!(*element, eq n);
        }
    }

    #[instantiate_tests(<PolymorphicVecFactory>)]
    mod polymorphic_vec {}

    #[instantiate_tests(<RelocatableVecFactory>)]
    mod relocatable_vec {}

    #[instantiate_tests(<StaticVecFactory>)]
    mod static_vec {}
}
