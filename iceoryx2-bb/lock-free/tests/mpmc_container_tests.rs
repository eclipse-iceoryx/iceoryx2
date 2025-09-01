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

use iceoryx2_bb_testing::assert_that;

#[derive(Clone, Copy, Debug)]
struct TestType {
    some_numbers: [usize; 32],
}

impl From<usize> for TestType {
    fn from(value: usize) -> Self {
        TestType {
            some_numbers: {
                let mut n = [0usize; 32];
                for i in 0..n.len() {
                    n[i] = value + i;
                }
                n
            },
        }
    }
}

impl From<TestType> for usize {
    fn from(value: TestType) -> Self {
        for i in 0..value.some_numbers.len() {
            assert_that!(value.some_numbers[i], eq value.some_numbers[0] + i);
        }
        value.some_numbers[0]
    }
}

unsafe impl Send for TestType {}

#[generic_tests::define]
mod mpmc_container {
    use core::fmt::Debug;
    use core::sync::atomic::AtomicU32;
    use core::sync::atomic::Ordering;
    use iceoryx2_bb_elementary::bump_allocator::BumpAllocator;
    use iceoryx2_bb_elementary::CallbackProgression;
    use iceoryx2_bb_elementary_traits::relocatable_container::RelocatableContainer;
    use iceoryx2_bb_lock_free::mpmc::container::ContainerAddFailure;
    use iceoryx2_bb_lock_free::mpmc::container::*;
    use iceoryx2_bb_lock_free::mpmc::unique_index_set::ReleaseMode;
    use iceoryx2_bb_lock_free::mpmc::unique_index_set::ReleaseState;
    use iceoryx2_bb_posix::system_configuration::SystemInfo;
    use iceoryx2_bb_testing::assert_that;
    use std::collections::HashMap;
    use std::collections::HashSet;
    use std::sync::{Barrier, Mutex};
    use std::thread;

    const CAPACITY: usize = 129;

    #[test]
    fn mpmc_container_add_elements_until_full_works<T: Debug + Copy + From<usize> + Into<usize>>() {
        let sut = FixedSizeContainer::<T, CAPACITY>::new();
        assert_that!(sut.capacity(), eq CAPACITY);
        for i in 0..CAPACITY {
            let index = unsafe { sut.add((i * 5 + 2).into()) };
            assert_that!(index, is_ok);
        }
        let index = unsafe { sut.add(0.into()) };
        assert_that!(index, is_err);
        assert_that!(index.err().unwrap(), eq ContainerAddFailure::OutOfSpace);

        let state = sut.get_state();
        let mut contained_values: Vec<(u32, usize)> = vec![];
        state.for_each(|h: ContainerHandle, value: &T| {
            contained_values.push((h.index(), (*value).into()));
            CallbackProgression::Continue
        });

        for i in 0..CAPACITY {
            assert_that!(contained_values[i].0, eq i as u32);
            assert_that!(contained_values[i].1, eq i * 5 + 2);
        }
    }

    #[test]
    fn mpmc_container_add_and_remove_elements_works<T: Debug + Copy + From<usize> + Into<usize>>() {
        let sut = FixedSizeContainer::<T, CAPACITY>::new();
        assert_that!(sut.is_empty(), eq true);
        let mut stored_indices = vec![];
        for i in 0..CAPACITY - 1 {
            let index = unsafe { sut.add((i * 3 + 1).into()) };
            assert_that!(index, is_ok);
            stored_indices.push(index.unwrap());

            let index = unsafe { sut.add((i * 7 + 5).into()) };
            assert_that!(index, is_ok);
            stored_indices.push(index.unwrap());

            unsafe {
                sut.remove(
                    stored_indices.remove(stored_indices.len() - 2),
                    ReleaseMode::Default,
                )
            };
        }
        assert_that!(sut.is_empty(), eq false);

        let state = sut.get_state();
        let mut contained_values = vec![];
        state.for_each(|_: ContainerHandle, value: &T| {
            contained_values.push((*value).into());
            CallbackProgression::Continue
        });

        for i in 0..CAPACITY - 1 {
            assert_that!(contained_values[i], eq i * 7 + 5);
        }
    }

    #[test]
    fn mpmc_container_add_and_remove_elements_works_with_uninitialized_memory<
        T: Debug + Copy + From<usize> + Into<usize>,
    >() {
        // TestType is the largest test type so it is safe to acquire this memory for every test
        // case - hack required since `T` cannot be used in const operations
        let mut memory = [0u8; Container::<crate::TestType>::const_memory_size(129_usize)];
        let allocator = BumpAllocator::new(memory.as_mut_ptr());
        let mut sut = unsafe { Container::<T>::new_uninit(CAPACITY) };
        unsafe { assert_that!(sut.init(&allocator), is_ok) };

        let mut stored_indices = vec![];
        for i in 0..CAPACITY - 1 {
            let index = unsafe { sut.add((i * 3 + 1).into()) };
            assert_that!(index, is_ok);
            stored_indices.push(index.unwrap());

            let index = unsafe { sut.add((i * 7 + 5).into()) };
            assert_that!(index, is_ok);
            stored_indices.push(index.unwrap());

            unsafe {
                sut.remove(
                    stored_indices.remove(stored_indices.len() - 2),
                    ReleaseMode::Default,
                )
            };
        }

        let state = unsafe { sut.get_state() };
        let mut contained_values = vec![];
        state.for_each(|_: ContainerHandle, value: &T| {
            contained_values.push((*value).into());
            CallbackProgression::Continue
        });

        for i in 0..CAPACITY - 1 {
            assert_that!(contained_values[i], eq i * 7 + 5);
        }
    }

    #[test]
    fn mpmc_container_add_and_unsafe_remove_with_handle_works<
        T: Debug + Copy + From<usize> + Into<usize>,
    >() {
        let sut = FixedSizeContainer::<T, CAPACITY>::new();
        let mut stored_handles: Vec<ContainerHandle> = vec![];

        for i in 0..CAPACITY - 1 {
            let handle = unsafe { sut.add((i * 3 + 1).into()) };
            assert_that!(handle, is_ok);
            stored_handles.push(handle.unwrap());

            let handle = unsafe { sut.add((i * 7 + 5).into()) };
            assert_that!(handle, is_ok);
            stored_handles.push(handle.unwrap());

            unsafe {
                sut.remove(
                    stored_handles[stored_handles.len() - 2],
                    ReleaseMode::Default,
                )
            };
        }

        let state = sut.get_state();
        let mut contained_values = vec![];
        state.for_each(|_: ContainerHandle, value: &T| {
            contained_values.push((*value).into());
            CallbackProgression::Continue
        });

        for i in 0..CAPACITY - 1 {
            assert_that!(contained_values[i], eq i * 7 + 5);
        }
    }

    #[test]
    fn mpmc_container_state_of_empty_container_is_empty<
        T: Debug + Copy + From<usize> + Into<usize>,
    >() {
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
    fn mpmc_container_state_not_updated_when_contents_do_not_change<
        T: Debug + Copy + From<usize> + Into<usize>,
    >() {
        let sut = FixedSizeContainer::<T, CAPACITY>::new();

        for i in 0..CAPACITY - 1 {
            let index = unsafe { sut.add((i * 3 + 1).into()) };
            assert_that!(index, is_ok);
        }

        let mut state = sut.get_state();
        let mut contained_values1 = vec![];
        state.for_each(|_: ContainerHandle, value: &T| {
            contained_values1.push((*value).into());
            CallbackProgression::Continue
        });

        assert_that!(unsafe { sut.update_state(&mut state) }, eq false);
        let mut contained_values2 = vec![];
        state.for_each(|_: ContainerHandle, value: &T| {
            contained_values2.push((*value).into());
            CallbackProgression::Continue
        });

        for i in 0..CAPACITY - 1 {
            assert_that!(contained_values1[i], eq i * 3 + 1);
            assert_that!(contained_values2[i], eq i * 3 + 1);
        }
    }

    #[test]
    fn mpmc_container_state_updated_when_contents_are_removed<
        T: Debug + Copy + From<usize> + Into<usize>,
    >() {
        let sut = FixedSizeContainer::<T, CAPACITY>::new();
        let mut stored_indices: Vec<ContainerHandle> = vec![];

        for i in 0..CAPACITY - 1 {
            let index = unsafe { sut.add((i * 3 + 1).into()) };
            assert_that!(index, is_ok);
            stored_indices.push(index.unwrap());
        }

        let mut state = sut.get_state();
        for i in &stored_indices {
            assert_that!(unsafe { sut.remove(*i, ReleaseMode::Default) }, eq ReleaseState::Unlocked);
        }
        stored_indices.clear();

        assert_that!(unsafe { sut.update_state(&mut state) }, eq true);
        let mut contained_values = vec![];
        state.for_each(|_: ContainerHandle, value: &T| {
            contained_values.push((*value).into());
            CallbackProgression::Continue
        });

        assert_that!(contained_values, is_empty);
    }

    #[test]
    fn mpmc_container_state_updated_when_contents_are_changed<
        T: Debug + Copy + From<usize> + Into<usize>,
    >() {
        let sut = FixedSizeContainer::<T, CAPACITY>::new();
        let mut stored_indices: Vec<ContainerHandle> = vec![];

        for i in 0..CAPACITY - 1 {
            let index = unsafe { sut.add((i * 3 + 1).into()) };
            assert_that!(index, is_ok);
            stored_indices.push(index.unwrap());
        }

        let mut state = sut.get_state();
        for i in stored_indices {
            assert_that!(unsafe { sut.remove(i, ReleaseMode::Default) }, eq ReleaseState::Unlocked);
        }

        let mut results = HashMap::<u32, usize>::new();
        for i in 0..CAPACITY - 1 {
            let index = unsafe { sut.add((i * 81 + 56).into()) };
            assert_that!(index, is_ok);
            results.insert(index.as_ref().unwrap().index(), i * 81 + 56);
        }

        assert_that!(unsafe { sut.update_state(&mut state) }, eq true);
        let mut contained_values = vec![];
        state.for_each(|_: ContainerHandle, value: &T| {
            contained_values.push((*value).into());
            CallbackProgression::Continue
        });

        for i in 0..CAPACITY - 1 {
            assert_that!(contained_values[i], eq * results.get(&(i as u32)).unwrap());
        }
    }

    #[test]
    fn mpmc_container_state_updated_works_for_new_and_removed_elements<
        T: Debug + Copy + From<usize> + Into<usize>,
    >() {
        let sut = FixedSizeContainer::<T, CAPACITY>::new();
        let mut state = sut.get_state();
        let mut stored_indices: Vec<ContainerHandle> = vec![];
        let mut stored_values: Vec<(u32, usize)> = vec![];

        for i in 0..CAPACITY {
            let v = i * 3 + 1;
            let index = unsafe { sut.add(v.into()) };
            assert_that!(index, is_ok);
            stored_values.push((index.as_ref().unwrap().index(), v));
            stored_indices.push(index.unwrap());

            unsafe { sut.update_state(&mut state) };
            let mut contained_values = vec![];
            state.for_each(|h: ContainerHandle, value: &T| {
                contained_values.push((h.index(), (*value).into()));
                CallbackProgression::Continue
            });

            assert_that!(contained_values, len stored_values.len());
            for e in &stored_values {
                assert_that!(contained_values, contains * e);
            }
        }

        for _ in 0..CAPACITY {
            unsafe { sut.remove(stored_indices.pop().unwrap(), ReleaseMode::Default) };
            stored_values.pop();

            unsafe { sut.update_state(&mut state) };
            let mut contained_values = vec![];
            state.for_each(|h: ContainerHandle, value: &T| {
                contained_values.push((h.index(), (*value).into()));
                CallbackProgression::Continue
            });

            assert_that!(contained_values, len stored_values.len());
            for e in &stored_values {
                assert_that!(contained_values, contains * e);
            }
        }
    }

    #[test]
    fn mpmc_container_state_updated_works_when_same_element_is_added_and_removed<
        T: Debug + Copy + From<usize> + Into<usize>,
    >() {
        let sut = FixedSizeContainer::<T, CAPACITY>::new();
        let mut state = sut.get_state();

        let index = unsafe { sut.add(123.into()) }.unwrap();
        unsafe { sut.remove(index, ReleaseMode::Default) };
        assert_that!(unsafe { sut.update_state(&mut state) }, eq true);
    }

    #[test]
    fn mpmc_container_concurrent_add_release_for_each<
        T: Debug + Copy + From<usize> + Into<usize> + Send,
    >() {
        const REPETITIONS: i64 = 1000;
        let number_of_threads_per_op =
            (SystemInfo::NumberOfCpuCores.value() / 2).clamp(2, usize::MAX);

        let sut = FixedSizeContainer::<T, CAPACITY>::new();
        let barrier = Barrier::new(number_of_threads_per_op * 2);
        let mut added_content: Vec<Mutex<Vec<(u32, T)>>> = vec![];
        let mut extracted_content: Vec<Mutex<Vec<(u32, T)>>> = vec![];

        for _ in 0..number_of_threads_per_op {
            added_content.push(Mutex::new(vec![]));
            extracted_content.push(Mutex::new(vec![]));
        }

        let finished_threads_counter = AtomicU32::new(0);
        thread::scope(|s| {
            for thread_number in 0..number_of_threads_per_op {
                let barrier = &barrier;
                let sut = &sut;
                let added_content = &added_content;
                let finished_threads_counter = &finished_threads_counter;
                s.spawn(move || {
                    let mut repetition = 0;
                    let mut ids = vec![];
                    let mut counter = 0;

                    barrier.wait();
                    while repetition < REPETITIONS {
                        counter += 1;
                        let value = counter * number_of_threads_per_op + thread_number;

                        match unsafe { sut.add(value.into()) } {
                            Ok(index) => {
                                let index_value = index.index();
                                ids.push(index);
                                added_content[thread_number]
                                    .lock()
                                    .unwrap()
                                    .push((index_value, value.into()));
                            }
                            Err(ContainerAddFailure::OutOfSpace) => {
                                repetition += 1;
                                for id in &ids {
                                    unsafe { sut.remove(*id, ReleaseMode::Default) };
                                }
                                ids.clear();
                            }
                            Err(ContainerAddFailure::IsLocked) => {
                                assert_that!(true, eq false);
                            }
                        }
                    }

                    finished_threads_counter.fetch_add(1, Ordering::Relaxed);
                });
            }

            for thread_number in 0..number_of_threads_per_op {
                let sut = &sut;
                let barrier = &barrier;
                let finished_threads_counter = &finished_threads_counter;
                let extracted_content = &extracted_content;
                s.spawn(move || {
                    barrier.wait();

                    let mut state = sut.get_state();
                    while finished_threads_counter.load(Ordering::Relaxed)
                        != number_of_threads_per_op as u32
                    {
                        if unsafe { sut.update_state(&mut state) } {
                            state.for_each(|h: ContainerHandle, value: &T| {
                                extracted_content[thread_number]
                                    .lock()
                                    .unwrap()
                                    .push((h.index(), *value));
                                CallbackProgression::Continue
                            })
                        }
                    }
                });
            }
        });

        let mut added_contents_set = HashSet::<(u32, usize)>::new();

        for thread_number in 0..number_of_threads_per_op {
            for entry in &*added_content[thread_number].lock().unwrap() {
                added_contents_set.insert((entry.0, entry.1.into()));
            }
        }

        for thread_number in 0..number_of_threads_per_op {
            for entry in &*extracted_content[thread_number].lock().unwrap() {
                assert_that!(added_contents_set.get(&(entry.0, entry.1.into())), is_some);
            }
        }

        // check if it is still in a consistent state
        mpmc_container_add_and_remove_elements_works::<T>();
    }

    #[instantiate_tests(<usize>)]
    mod usize {}

    #[instantiate_tests(<crate::TestType>)]
    mod test_type {}
}
