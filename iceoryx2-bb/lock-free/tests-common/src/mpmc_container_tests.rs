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

use iceoryx2_bb_testing_macros::tests;

#[tests(usize, TestType)]
pub mod generic {
    use alloc::collections::btree_map::BTreeMap;
    use alloc::collections::btree_set::BTreeSet;
    use alloc::vec;
    use alloc::vec::Vec;
    use core::fmt::Debug;
    use iceoryx2_bb_lock_free::mpmc::robust_unique_index_set::OwnerId;
    use iceoryx2_bb_lock_free::mpmc::unique_index_set_enums::{ReleaseMode, ReleaseState};
    use iceoryx2_bb_testing::watchdog::Watchdog;

    use iceoryx2_bb_concurrency::atomic::{AtomicU32, AtomicU64, Ordering};
    use iceoryx2_bb_elementary::CallbackProgression;
    use iceoryx2_bb_elementary::bump_allocator::BumpAllocator;
    use iceoryx2_bb_elementary_traits::relocatable_container::RelocatableContainer;
    use iceoryx2_bb_lock_free::mpmc::container::ContainerAddFailure;
    use iceoryx2_bb_lock_free::mpmc::container::*;
    use iceoryx2_bb_posix::barrier::{BarrierBuilder, BarrierHandle, Handle};
    use iceoryx2_bb_posix::mutex::{MutexBuilder, MutexHandle};
    use iceoryx2_bb_posix::system_configuration::SystemInfo;
    use iceoryx2_bb_posix::thread::thread_scope;
    use iceoryx2_bb_testing::assert_that;

    #[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Eq, Ord)]
    pub struct TestType {
        some_numbers: [usize; 32],
    }

    impl From<usize> for TestType {
        fn from(value: usize) -> Self {
            TestType {
                some_numbers: {
                    let mut n = [0usize; 32];
                    n.iter_mut().enumerate().for_each(|(i, elem)| {
                        *elem = value + i;
                    });
                    n
                },
            }
        }
    }

    impl From<TestType> for usize {
        fn from(value: TestType) -> Self {
            for (i, &num) in value.some_numbers.iter().enumerate() {
                assert_that!(num, eq value.some_numbers[0] + i);
            }
            value.some_numbers[0]
        }
    }

    unsafe impl Send for TestType {}

    const CAPACITY: usize = 129;

    #[test]
    pub fn add_elements_until_full_works<T: Debug + Copy + From<usize> + Into<usize>>() {
        let sut = FixedSizeContainer::<T, CAPACITY>::new();
        assert_that!(sut.capacity(), eq CAPACITY);
        let owner_id = OwnerId::new(2).unwrap();
        for i in 0..CAPACITY {
            let index = sut.add((i * 5 + 2).into(), owner_id);
            assert_that!(index, is_ok);
        }
        let index = sut.add(0.into(), owner_id);
        assert_that!(index, is_err);
        assert_that!(index.err().unwrap(), eq ContainerAddFailure::OutOfSpace);

        let state = sut.get_state();
        let mut contained_values: Vec<usize> = vec![];
        state.for_each(|_, value: &T| {
            contained_values.push((*value).into());
            CallbackProgression::Continue
        });

        contained_values.iter().enumerate().for_each(|(i, &value)| {
            assert_that!(value, eq i * 5 + 2);
        });
    }

    #[test]
    pub fn add_and_remove_elements_works<T: Debug + Copy + From<usize> + Into<usize>>() {
        let sut = FixedSizeContainer::<T, CAPACITY>::new();
        assert_that!(sut.is_empty(), eq true);
        let mut stored_indices = vec![];
        let owner_id = OwnerId::new(2).unwrap();
        for i in 0..CAPACITY - 1 {
            let index = sut.add((i * 3 + 1).into(), owner_id);
            assert_that!(index, is_ok);
            stored_indices.push(index.unwrap());

            let index = sut.add((i * 7 + 5).into(), owner_id);
            assert_that!(index, is_ok);
            stored_indices.push(index.unwrap());

            unsafe {
                sut.remove(
                    stored_indices.remove(stored_indices.len() - 2),
                    ReleaseMode::Default,
                )
                .unwrap()
            };
        }
        assert_that!(sut.is_empty(), eq false);

        let state = sut.get_state();
        let mut contained_values = vec![];
        state.for_each(|_, value: &T| {
            contained_values.push((*value).into());
            CallbackProgression::Continue
        });

        contained_values.iter().enumerate().for_each(|(i, &value)| {
            assert_that!(value, eq i * 7 + 5);
        });
    }

    #[test]
    pub fn double_remove_is_detected<T: Debug + Copy + From<usize> + Into<usize>>() {
        let sut = FixedSizeContainer::<T, CAPACITY>::new();
        let mut stored_indices = vec![];
        let owner_id = OwnerId::new(2).unwrap();
        for i in 0..CAPACITY - 1 {
            let index = sut.add((i * 3 + 1).into(), owner_id);
            assert_that!(index, is_ok);
            stored_indices.push(index.unwrap());

            let index = sut.add((i * 7 + 5).into(), owner_id);
            assert_that!(index, is_ok);
            stored_indices.push(index.unwrap());

            let to_be_removed_handle = stored_indices.remove(stored_indices.len() - 2);
            assert_that!(
                unsafe { sut.remove(to_be_removed_handle, ReleaseMode::Default) },
                is_ok
            );

            assert_that!(unsafe {
                sut.remove(
                    to_be_removed_handle,
                    ReleaseMode::Default,
                ).err()
            }, eq Some(ContainerRemoveError::ContainerHandleNotOwnedByContainer));
        }
        assert_that!(sut.is_empty(), eq false);

        let state = sut.get_state();
        let mut contained_values = vec![];
        state.for_each(|_, value: &T| {
            contained_values.push((*value).into());
            CallbackProgression::Continue
        });

        contained_values.iter().enumerate().for_each(|(i, &value)| {
            assert_that!(value, eq i * 7 + 5);
        });
    }

    #[test]
    pub fn add_and_remove_elements_works_with_uninitialized_memory<
        T: Debug + Copy + From<usize> + Into<usize>,
    >() {
        // TestType is the largest test type so it is safe to acquire this memory for every test
        // case - hack required since `T` cannot be used in const operations
        let mut memory = [0u8; Container::<TestType>::const_memory_size(129_usize)];
        let allocator = BumpAllocator::new(memory.as_mut_ptr());
        let mut sut = unsafe { Container::<T>::new_uninit(CAPACITY) };
        unsafe { assert_that!(sut.init(&allocator), is_ok) };

        let mut stored_indices = vec![];
        let owner_id = OwnerId::new(2).unwrap();
        for i in 0..CAPACITY - 1 {
            let index = unsafe { sut.add((i * 3 + 1).into(), owner_id) };
            assert_that!(index, is_ok);
            stored_indices.push(index.unwrap());

            let index = unsafe { sut.add((i * 7 + 5).into(), owner_id) };
            assert_that!(index, is_ok);
            stored_indices.push(index.unwrap());

            unsafe {
                sut.remove(
                    stored_indices.remove(stored_indices.len() - 2),
                    ReleaseMode::Default,
                )
                .unwrap()
            };
        }

        let state = unsafe { sut.get_state() };
        let mut contained_values = vec![];
        state.for_each(|_, value: &T| {
            contained_values.push((*value).into());
            CallbackProgression::Continue
        });

        contained_values.iter().enumerate().for_each(|(i, &value)| {
            assert_that!(value, eq i * 7 + 5);
        });
    }

    #[test]
    pub fn add_and_unsafe_remove_with_handle_works<T: Debug + Copy + From<usize> + Into<usize>>() {
        let sut = FixedSizeContainer::<T, CAPACITY>::new();
        let mut stored_handles: Vec<ContainerHandle> = vec![];

        let owner_id = OwnerId::new(2).unwrap();
        for i in 0..CAPACITY - 1 {
            let handle = sut.add((i * 3 + 1).into(), owner_id);
            assert_that!(handle, is_ok);
            stored_handles.push(handle.unwrap());

            let handle = sut.add((i * 7 + 5).into(), owner_id);
            assert_that!(handle, is_ok);
            stored_handles.push(handle.unwrap());

            unsafe {
                sut.remove(
                    stored_handles[stored_handles.len() - 2],
                    ReleaseMode::Default,
                )
                .unwrap()
            };
        }

        let state = sut.get_state();
        let mut contained_values = vec![];
        state.for_each(|_, value: &T| {
            contained_values.push((*value).into());
            CallbackProgression::Continue
        });

        contained_values.iter().enumerate().for_each(|(i, &value)| {
            assert_that!(value, eq i * 7 + 5);
        });
    }

    #[test]
    pub fn state_of_empty_container_is_empty<T: Debug + Copy + From<usize> + Into<usize>>() {
        let sut = FixedSizeContainer::<T, CAPACITY>::new();
        let mut counter = 0;

        let mut state = sut.get_state();
        state.for_each(|_, _| {
            counter += 1;
            CallbackProgression::Continue
        });
        assert_that!(counter, eq 0);

        unsafe { sut.update_state(&mut state) };
        state.for_each(|_, _| {
            counter += 1;
            CallbackProgression::Continue
        });
        assert_that!(counter, eq 0);
    }

    #[test]
    pub fn state_not_updated_when_contents_do_not_change<
        T: Debug + Copy + From<usize> + Into<usize>,
    >() {
        let sut = FixedSizeContainer::<T, CAPACITY>::new();
        let owner_id = OwnerId::new(4).unwrap();

        for i in 0..CAPACITY - 1 {
            let index = sut.add((i * 3 + 1).into(), owner_id);
            assert_that!(index, is_ok);
        }

        let mut state = sut.get_state();
        let mut contained_values1 = vec![];
        state.for_each(|_, value: &T| {
            contained_values1.push((*value).into());
            CallbackProgression::Continue
        });

        assert_that!(unsafe { sut.update_state(&mut state) }, eq false);
        let mut contained_values2 = vec![];
        state.for_each(|_, value: &T| {
            contained_values2.push((*value).into());
            CallbackProgression::Continue
        });

        for i in 0..CAPACITY - 1 {
            assert_that!(contained_values1[i], eq i * 3 + 1);
            assert_that!(contained_values2[i], eq i * 3 + 1);
        }
    }

    #[test]
    pub fn state_updated_when_contents_are_removed<T: Debug + Copy + From<usize> + Into<usize>>() {
        let sut = FixedSizeContainer::<T, CAPACITY>::new();
        let mut stored_indices: Vec<ContainerHandle> = vec![];
        let owner_id = OwnerId::new(12903).unwrap();

        for i in 0..CAPACITY - 1 {
            let index = sut.add((i * 3 + 1).into(), owner_id);
            assert_that!(index, is_ok);
            stored_indices.push(index.unwrap());
        }

        let mut state = sut.get_state();
        for i in &stored_indices {
            assert_that!(unsafe { sut.remove(*i, ReleaseMode::Default) }.unwrap(), eq ReleaseState::Unlocked);
        }
        stored_indices.clear();

        assert_that!(unsafe { sut.update_state(&mut state) }, eq true);
        let mut contained_values = vec![];
        state.for_each(|_, value: &T| {
            contained_values.push((*value).into());
            CallbackProgression::Continue
        });

        assert_that!(contained_values, is_empty);
    }

    #[test]
    pub fn state_updated_when_contents_are_changed<T: Debug + Copy + From<usize> + Into<usize>>() {
        let sut = FixedSizeContainer::<T, CAPACITY>::new();
        let mut stored_indices: Vec<ContainerHandle> = vec![];
        let owner_id = OwnerId::new(12903).unwrap();

        for i in 0..CAPACITY - 1 {
            let index = sut.add((i * 3 + 1).into(), owner_id);
            assert_that!(index, is_ok);
            stored_indices.push(index.unwrap());
        }

        let mut state = sut.get_state();
        for i in stored_indices {
            assert_that!(unsafe { sut.remove(i, ReleaseMode::Default).unwrap() }, eq ReleaseState::Unlocked);
        }

        let mut results = BTreeMap::<usize, usize>::new();
        for i in 0..CAPACITY - 1 {
            let index = sut.add((i * 81 + 56).into(), owner_id);
            assert_that!(index, is_ok);
            results.insert(index.as_ref().unwrap().index(), i * 81 + 56);
        }

        assert_that!(unsafe { sut.update_state(&mut state) }, eq true);
        let mut contained_values = vec![];
        state.for_each(|_, value: &T| {
            contained_values.push((*value).into());
            CallbackProgression::Continue
        });

        for (i, value) in contained_values.iter().enumerate() {
            assert_that!(*value, eq * results.get(&i).unwrap());
        }
    }

    #[test]
    pub fn state_updated_works_for_new_and_removed_elements<
        T: Debug + Copy + From<usize> + Into<usize>,
    >() {
        let sut = FixedSizeContainer::<T, CAPACITY>::new();
        let mut state = sut.get_state();
        let mut stored_indices: Vec<ContainerHandle> = vec![];
        let mut stored_values: Vec<usize> = vec![];
        let owner_id = OwnerId::new(12903).unwrap();

        for i in 0..CAPACITY {
            let v = i * 3 + 1;
            let index = sut.add(v.into(), owner_id);
            assert_that!(index, is_ok);
            stored_values.push(v);
            stored_indices.push(index.unwrap());

            unsafe { sut.update_state(&mut state) };
            let mut contained_values = vec![];
            state.for_each(|_, value: &T| {
                contained_values.push((*value).into());
                CallbackProgression::Continue
            });

            assert_that!(contained_values, len stored_values.len());
            for e in &stored_values {
                assert_that!(contained_values, contains * e);
            }
        }

        for _ in 0..CAPACITY {
            unsafe {
                sut.remove(stored_indices.pop().unwrap(), ReleaseMode::Default)
                    .unwrap()
            };
            stored_values.pop();

            unsafe { sut.update_state(&mut state) };
            let mut contained_values = vec![];
            state.for_each(|_, value: &T| {
                contained_values.push((*value).into());
                CallbackProgression::Continue
            });

            assert_that!(contained_values, len stored_values.len());
            for e in &stored_values {
                assert_that!(contained_values, contains * e);
            }
        }
    }

    #[test]
    pub fn state_updated_works_when_same_element_is_added_and_removed<
        T: Debug + Copy + From<usize> + Into<usize>,
    >() {
        let sut = FixedSizeContainer::<T, CAPACITY>::new();
        let mut state = sut.get_state();
        let owner_id = OwnerId::new(123).unwrap();

        let index = sut.add(123.into(), owner_id).unwrap();
        unsafe { sut.remove(index, ReleaseMode::Default).unwrap() };
        assert_that!(unsafe { sut.update_state(&mut state) }, eq true);
    }

    #[test]
    pub fn concurrent_add_release_for_each<T: Debug + Copy + From<usize> + Into<usize> + Send>() {
        const REPETITIONS: i64 = 1000;
        let number_of_threads_per_op =
            (SystemInfo::NumberOfCpuCores.value() / 2).clamp(2, usize::MAX);
        let owner_id = OwnerId::new(123).unwrap();

        let sut = FixedSizeContainer::<T, CAPACITY>::new();
        let barrier_handle = BarrierHandle::new();
        let barrier = BarrierBuilder::new((number_of_threads_per_op * 2) as u32)
            .create(&barrier_handle)
            .unwrap();

        let added_handle = MutexHandle::<Vec<usize>>::new();
        let added = MutexBuilder::new()
            .create(vec![], &added_handle)
            .expect("failed to create mutex");
        let extracted_handle = MutexHandle::<Vec<usize>>::new();
        let extracted = MutexBuilder::new()
            .create(vec![], &extracted_handle)
            .expect("failed to create mutex");

        let finished_threads_counter = AtomicU32::new(0);

        thread_scope(|s| {
            for thread_number in 0..number_of_threads_per_op {
                s.thread_builder()
                    .spawn(|| {
                        let mut repetition = 0;
                        let mut ids = vec![];
                        let mut counter = 0;

                        barrier.wait();
                        while repetition < REPETITIONS {
                            counter += 1;
                            let value = counter * number_of_threads_per_op + thread_number;

                            match sut.add(value.into(), owner_id) {
                                Ok(index) => {
                                    ids.push(index);
                                    added.lock().expect("failed to lock mutex").push(value);
                                }
                                Err(ContainerAddFailure::OutOfSpace) => {
                                    repetition += 1;
                                    for id in &ids {
                                        unsafe { sut.remove(*id, ReleaseMode::Default).unwrap() };
                                    }
                                    ids.clear();
                                }
                                Err(ContainerAddFailure::IsLocked) => {
                                    assert_that!(true, eq false);
                                }
                            }
                        }

                        finished_threads_counter.fetch_add(1, Ordering::Relaxed);
                    })
                    .expect("failed to spawn thread");
            }

            for _ in 0..number_of_threads_per_op {
                s.thread_builder()
                    .spawn(|| {
                        barrier.wait();

                        let mut state = sut.get_state();
                        while finished_threads_counter.load(Ordering::Relaxed)
                            != number_of_threads_per_op as u32
                        {
                            if unsafe { sut.update_state(&mut state) } {
                                state.for_each(|_, value: &T| {
                                    extracted
                                        .lock()
                                        .expect("failed to lock mutex")
                                        .push((*value).into());
                                    CallbackProgression::Continue
                                })
                            }
                        }
                    })
                    .expect("failed to spawn thread");
            }

            Ok(())
        })
        .expect("failed to run thread scope");

        let added_set: BTreeSet<usize> = added
            .lock()
            .expect("failed to lock mutex")
            .iter()
            .copied()
            .collect();

        for entry in extracted.lock().expect("failed to lock mutex").iter() {
            assert_that!(added_set.get(entry), is_some);
        }

        // check if it is still in a consistent state
        add_and_remove_elements_works::<T>();
    }

    #[test]
    pub fn recover_cleans_up_dead_entries<T: Debug + Copy + From<usize> + Into<usize>>() {
        let sut = FixedSizeContainer::<T, { CAPACITY * 2 }>::new();
        let mut stored_handles: Vec<ContainerHandle> = vec![];

        let good_owner_id = OwnerId::new(2).unwrap();
        let bad_owner_id = OwnerId::new(3).unwrap();
        for i in 0..CAPACITY {
            let handle = sut.add(i.into(), good_owner_id).unwrap();
            stored_handles.push(handle);
        }

        for i in 0..CAPACITY {
            let handle = sut.add((i + CAPACITY).into(), bad_owner_id).unwrap();
            stored_handles.push(handle);
        }

        unsafe { sut.recover(bad_owner_id, |_| true, ReleaseMode::Default) };

        let state = sut.get_state();
        let mut contained_values = vec![];
        state.for_each(|_, value: &T| {
            contained_values.push((*value).into());
            CallbackProgression::Continue
        });

        contained_values.iter().enumerate().for_each(|(i, &value)| {
            assert_that!(value, eq i);
        });
        assert_that!(contained_values, len CAPACITY);
    }

    #[test]
    pub fn recover_cleans_up_nothing_when_owner_id_is_not_present<
        T: Debug + Copy + From<usize> + Into<usize>,
    >() {
        let sut = FixedSizeContainer::<T, CAPACITY>::new();
        let mut stored_handles: Vec<ContainerHandle> = vec![];

        let good_owner_id = OwnerId::new(2).unwrap();
        let bad_owner_id = OwnerId::new(3).unwrap();
        for i in 0..CAPACITY {
            let handle = sut.add(i.into(), good_owner_id).unwrap();
            stored_handles.push(handle);
        }

        unsafe { sut.recover(bad_owner_id, |_| true, ReleaseMode::Default) };

        let state = sut.get_state();
        let mut contained_values = vec![];
        state.for_each(|_, value: &T| {
            contained_values.push((*value).into());
            CallbackProgression::Continue
        });

        contained_values.iter().enumerate().for_each(|(i, &value)| {
            assert_that!(value, eq i);
        });
        assert_that!(contained_values, len CAPACITY);
    }

    #[test]
    pub fn recover_cleans_up_nothing_when_predicate_returns_false<
        T: Debug + Copy + From<usize> + Into<usize>,
    >() {
        let sut = FixedSizeContainer::<T, { CAPACITY * 2 }>::new();
        let mut stored_handles: Vec<ContainerHandle> = vec![];

        let good_owner_id = OwnerId::new(2).unwrap();
        let bad_owner_id = OwnerId::new(3).unwrap();
        for i in 0..CAPACITY {
            let handle = sut.add(i.into(), good_owner_id).unwrap();
            stored_handles.push(handle);
        }

        for i in 0..CAPACITY {
            let handle = sut.add((i + CAPACITY).into(), bad_owner_id).unwrap();
            stored_handles.push(handle);
        }

        unsafe { sut.recover(bad_owner_id, |_| false, ReleaseMode::Default) };

        let state = sut.get_state();
        let mut contained_values = vec![];
        state.for_each(|_, value: &T| {
            contained_values.push((*value).into());
            CallbackProgression::Continue
        });

        contained_values.iter().enumerate().for_each(|(i, &value)| {
            assert_that!(value, eq i);
        });
        assert_that!(contained_values, len CAPACITY * 2);
    }

    #[test]
    pub fn recover_locks_container_with_release_mode_lock_if_last_index<
        T: Debug + Copy + From<usize> + Into<usize>,
    >() {
        let sut = FixedSizeContainer::<T, CAPACITY>::new();
        let mut stored_handles: Vec<ContainerHandle> = vec![];

        let bad_owner_id = OwnerId::new(3).unwrap();
        for i in 0..CAPACITY {
            let handle = sut.add((i + CAPACITY).into(), bad_owner_id).unwrap();
            stored_handles.push(handle);
        }

        unsafe { sut.recover(bad_owner_id, |_| true, ReleaseMode::LockIfLastIndex) };

        let state = sut.get_state();
        let mut contained_values = vec![];
        state.for_each(|_, value: &T| {
            contained_values.push((*value).into());
            CallbackProgression::Continue
        });

        assert_that!(contained_values, is_empty);
        assert_that!(sut.is_empty(), eq true);
        assert_that!(sut.is_locked(), eq true);
    }

    #[test]
    pub fn recover_provides_content_in_predicate_argument<
        T: Debug + Copy + From<usize> + Into<usize>,
    >() {
        let sut = FixedSizeContainer::<T, CAPACITY>::new();
        let mut stored_handles: Vec<ContainerHandle> = vec![];

        let bad_owner_id = OwnerId::new(3).unwrap();
        for i in 0..CAPACITY {
            let handle = sut.add(i.into(), bad_owner_id).unwrap();
            stored_handles.push(handle);
        }

        let mut predicate_values = vec![];
        unsafe {
            sut.recover(
                bad_owner_id,
                |c| {
                    predicate_values.push(c.into());
                    true
                },
                ReleaseMode::LockIfLastIndex,
            )
        };

        let state = sut.get_state();
        let mut contained_values = vec![];
        state.for_each(|_, value: &T| {
            contained_values.push((*value).into());
            CallbackProgression::Continue
        });

        assert_that!(contained_values, is_empty);
        for i in 0..CAPACITY {
            assert_that!(predicate_values, contains i);
        }
    }

    #[test]
    pub fn concurrent_add_and_recover<T: Debug + Copy + From<usize> + Into<usize> + Send + Ord>() {
        let _watchdog = Watchdog::new();
        const REPETITIONS: i64 = 1000;
        let number_of_threads_per_op = (SystemInfo::NumberOfCpuCores.value()).clamp(2, usize::MAX);

        let sut = FixedSizeContainer::<T, CAPACITY>::new();
        let barrier_handle = BarrierHandle::new();
        let barrier = BarrierBuilder::new((number_of_threads_per_op) as u32)
            .create(&barrier_handle)
            .unwrap();

        let add_thread_counter = AtomicU64::new(0);
        thread_scope(|s| {
            for _ in 0..number_of_threads_per_op {
                s.thread_builder()
                    .spawn(|| {
                        let thread_number = add_thread_counter.fetch_add(1, Ordering::Relaxed);
                        let owner =
                            OwnerId::new(thread_number + (2 * number_of_threads_per_op + 2) as u64)
                                .unwrap();

                        let mut added_indices = Vec::<T>::new();
                        barrier.wait();

                        for _ in 0..REPETITIONS {
                            for i in 0..CAPACITY {
                                match sut.add(i.into(), owner) {
                                    Ok(_) => added_indices.push(i.into()),
                                    Err(ContainerAddFailure::OutOfSpace) => break,
                                    Err(ContainerAddFailure::IsLocked) => {
                                        assert_that!(true, eq false)
                                    }
                                }
                            }

                            unsafe {
                                sut.recover(
                                    owner,
                                    |i| {
                                        assert_that!(added_indices, contains i);
                                        true
                                    },
                                    ReleaseMode::Default,
                                )
                            };
                        }
                    })
                    .expect("failed to spawn thread");
            }

            Ok(())
        })
        .expect("failed to run thread scope");
    }
}
