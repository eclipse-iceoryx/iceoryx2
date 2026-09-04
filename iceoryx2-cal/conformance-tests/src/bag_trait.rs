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

use iceoryx2_bb_testing_macros::conformance_tests;

#[allow(clippy::module_inception)]
#[conformance_tests]
pub mod bag_trait {
    use core::ptr::NonNull;
    use iceoryx2_bb_derive_macros::ZeroCopySend;
    use iceoryx2_bb_elementary::CallbackProgression;
    use iceoryx2_bb_elementary_traits::relocatable_container::RelocatableContainer;
    use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
    use iceoryx2_bb_lock_free::mpmc::unique_index_set_enums::ReleaseMode;
    use iceoryx2_bb_lock_free::mpmc::{container::OwnerId, unique_index_set_enums::ReleaseState};
    use iceoryx2_bb_memory::bump_allocator::BumpAllocator;
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_bb_testing_macros::conformance_test;
    use iceoryx2_cal::bag::{Bag, BagAddFailure, BagFamily};

    #[derive(Debug, Copy, Clone, PartialEq, ZeroCopySend)]
    #[repr(C)]
    pub struct Data {
        value: u64,
    }

    const BAG_CAPACITY: usize = 5;
    const BAG_ALLOCATOR_BUFFER_SIZE: usize = 4096;

    pub struct TestFixture<Sut: BagFamily> {
        buffer: [u8; BAG_ALLOCATOR_BUFFER_SIZE],
        sut: Sut::Bag<Data>,
    }

    impl<Sut: BagFamily> TestFixture<Sut> {
        fn create() -> TestFixture<Sut> {
            let mut fixture = Self {
                buffer: [0; BAG_ALLOCATOR_BUFFER_SIZE],
                sut: unsafe { Sut::Bag::<Data>::new_uninit(BAG_CAPACITY) },
            };
            let allocator = BumpAllocator::new(
                NonNull::new(fixture.buffer.as_ptr() as *mut u8).unwrap(),
                fixture.buffer.len(),
            );
            unsafe {
                fixture.sut.init(&allocator).unwrap();
            }
            fixture
        }
    }

    #[conformance_test]
    pub fn bag_reports_correct_capacity<Sut: BagFamily>() {
        let fixture = TestFixture::<Sut>::create();

        let sut = &fixture.sut;

        assert_that!(sut.capacity(), eq(BAG_CAPACITY));
    }

    #[conformance_test]
    pub fn newly_created_bag_is_empty<Sut: BagFamily>() {
        let fixture = TestFixture::<Sut>::create();

        let sut = &fixture.sut;

        assert_that!(sut.is_empty(), eq(true));
    }

    #[conformance_test]
    pub fn add_one_element_to_bag_leads_to_a_len_of_one<Sut: BagFamily>() {
        let fixture = TestFixture::<Sut>::create();

        let sut = &fixture.sut;

        unsafe {
            sut.add(Data { value: 42 }, OwnerId::new(13).unwrap())
                .unwrap();
        }

        assert_that!(sut.is_empty(), eq(false));
        assert_that!(sut.len(), eq(1));
    }

    #[conformance_test]
    pub fn exceeding_the_bag_capacity_results_in_an_error<Sut: BagFamily>() {
        let fixture = TestFixture::<Sut>::create();

        let sut = &fixture.sut;

        for i in 0..BAG_CAPACITY as _ {
            unsafe {
                sut.add(Data { value: 42 + i }, OwnerId::new(13 + i).unwrap())
                    .unwrap();
            }
        }

        let add_result = unsafe { sut.add(Data { value: 13 }, OwnerId::new(42).unwrap()) };

        assert_that!(add_result, eq(Err(BagAddFailure::OutOfSpace)));
    }

    #[conformance_test]
    pub fn remove_an_element_from_the_bag_works<Sut: BagFamily>() {
        let fixture = TestFixture::<Sut>::create();

        let sut = &fixture.sut;

        let (_, handle) = unsafe {
            sut.add(Data { value: 42 }, OwnerId::new(13).unwrap())
                .unwrap()
        };

        unsafe {
            sut.remove(handle, ReleaseMode::Default).unwrap();
        }

        assert_that!(sut.is_empty(), eq(true));
        assert_that!(sut.len(), eq(0));
    }

    #[conformance_test]
    pub fn get_state_works<Sut: BagFamily>() {
        let fixture = TestFixture::<Sut>::create();

        let sut = &fixture.sut;

        const DATA: u64 = 73;
        unsafe {
            sut.add(Data { value: DATA }, OwnerId::new(13).unwrap())
                .unwrap()
        };

        let state = unsafe { sut.get_state() };

        let mut count = 0;
        state.for_each(|_, element| {
            assert_that!(element.value, eq(DATA));
            count += 1;

            CallbackProgression::Continue
        });

        assert_that!(count, eq(1));
    }

    #[conformance_test]
    pub fn update_state_works<Sut: BagFamily>() {
        let fixture = TestFixture::<Sut>::create();

        let sut = &fixture.sut;

        const DATA1: u64 = 73;
        let (_, handle1) = unsafe {
            sut.add(Data { value: DATA1 }, OwnerId::new(13).unwrap())
                .unwrap()
        };

        let mut state = unsafe { sut.get_state() };

        const DATA2: u64 = 37;
        unsafe {
            sut.add(Data { value: DATA2 }, OwnerId::new(13).unwrap())
                .unwrap()
        };

        let state_updated = unsafe { sut.update_state(&mut state) };
        assert_that!(state_updated, eq(true));

        let mut count = 0;
        state.for_each(|index, element| {
            if index == handle1.index() {
                assert_that!(element.value, eq(DATA1));
            } else {
                assert_that!(element.value, eq(DATA2));
            }
            count += 1;

            CallbackProgression::Continue
        });

        assert_that!(count, eq(2));
    }

    #[conformance_test]
    pub fn recover_releases_all_affected_elements<Sut: BagFamily>() {
        let fixture = TestFixture::<Sut>::create();

        let sut = &fixture.sut;

        const DATA: u64 = 73;
        unsafe {
            sut.add(Data { value: DATA }, OwnerId::new(13).unwrap())
                .unwrap()
        };

        let id_to_recover = OwnerId::new(42).unwrap();
        for i in 0..2 {
            unsafe { sut.add(Data { value: i }, id_to_recover).unwrap() };
        }

        let release_state = unsafe { sut.recover(id_to_recover, |_| true, ReleaseMode::Default) };
        assert_that!(release_state, eq(ReleaseState::Unlocked));

        let state = unsafe { sut.get_state() };

        let mut count = 0;
        state.for_each(|_, element| {
            assert_that!(element.value, eq(DATA));
            count += 1;

            CallbackProgression::Continue
        });

        assert_that!(count, eq(1));
    }
}
