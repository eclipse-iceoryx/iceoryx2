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

use iceoryx2_bb_elementary::relocatable_container::RelocatableContainer;
use iceoryx2_bb_lock_free::mpmc::unique_index_set::*;
use iceoryx2_bb_memory::bump_allocator::BumpAllocator;
use iceoryx2_bb_memory::memory::Memory;
use iceoryx2_bb_posix::system_configuration::SystemInfo;
use iceoryx2_bb_testing::assert_that;
use pin_init::init_stack;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Barrier, Mutex};
use std::thread;

const CAPACITY: usize = 128;

#[test]
fn mpmc_unique_index_set_capacity_is_set_correctly() {
    let sut = FixedSizeUniqueIndexSet::<CAPACITY>::new();
    assert_that!(sut.capacity(), eq CAPACITY as u32);

    let sut = FixedSizeUniqueIndexSet::<CAPACITY>::new_with_reduced_capacity(CAPACITY * 2);
    assert_that!(sut, is_err);

    let sut = FixedSizeUniqueIndexSet::<CAPACITY>::new_with_reduced_capacity(CAPACITY / 2);
    assert_that!(sut, is_ok);
    assert_that!(sut.unwrap().capacity(), eq(CAPACITY / 2) as u32);

    let sut = FixedSizeUniqueIndexSet::<CAPACITY>::new_with_reduced_capacity(0);
    assert_that!(sut, is_err);
}

#[test]
fn mpmc_unique_index_set_when_created_contains_indices() {
    let sut = FixedSizeUniqueIndexSet::<CAPACITY>::new();
    let mut ids = vec![];

    for i in 0..CAPACITY {
        let e = sut.acquire();
        assert_that!(e, is_some);
        assert_that!(e.as_ref().unwrap().value(), eq i as u32);
        ids.push(e.unwrap());
    }

    let e = sut.acquire();
    assert_that!(e, is_none);
}

#[test]
fn mpmc_unique_index_set_acquire_and_release_works() {
    let sut = FixedSizeUniqueIndexSet::<CAPACITY>::new();
    let mut ids = vec![];

    for _ in 0..CAPACITY {
        let e = sut.acquire();
        assert_that!(e, is_some);
        ids.push(e.unwrap());
    }

    for i in 0..CAPACITY {
        ids.remove(CAPACITY - i - 1);
        let e = sut.acquire();
        assert_that!(e, is_some);
        assert_that!(e.unwrap().value(), eq(CAPACITY - i - 1) as u32);
    }

    for i in 0..CAPACITY {
        let e = sut.acquire();
        assert_that!(e, is_some);
        assert_that!(e.as_ref().unwrap().value(), eq i as u32);
        ids.push(e.unwrap());
    }
}

#[test]
fn mpmc_unique_index_set_borrowed_indices_works() {
    let sut = FixedSizeUniqueIndexSet::<CAPACITY>::new();
    let mut ids = vec![];

    for i in 0..CAPACITY {
        let e = sut.acquire();
        assert_that!(e, is_some);
        ids.push(e.unwrap());
        assert_that!(sut.borrowed_indices(), eq i + 1);
    }

    for i in 0..CAPACITY {
        ids.pop();
        assert_that!(sut.borrowed_indices(), eq CAPACITY - i - 1);
    }
}

#[test]
fn mpmc_unique_index_set_acquire_and_release_works_with_uninitialized_memory() {
    init_stack!(
        memory =
            Memory::<{ UniqueIndexSet::const_memory_size(128) }, BumpAllocator>::new_filled(0xff,)
    );
    let memory = memory.unwrap();
    let sut = unsafe { UniqueIndexSet::new_uninit(CAPACITY) };
    unsafe { assert_that!(sut.init(memory.allocator()), is_ok) };

    let mut ids = vec![];

    unsafe {
        for _ in 0..CAPACITY {
            let e = sut.acquire();
            assert_that!(e, is_some);
            ids.push(e.unwrap());
        }

        for i in 0..CAPACITY {
            ids.remove(CAPACITY - i - 1);
            let e = sut.acquire();
            assert_that!(e, is_some);
            assert_that!(e.unwrap().value(), eq(CAPACITY - i - 1) as u32);
        }

        for i in 0..CAPACITY {
            let e = sut.acquire();
            assert_that!(e, is_some);
            assert_that!(e.as_ref().unwrap().value(), eq i as u32);
            ids.push(e.unwrap());
        }
    }
}

#[test]
fn mpmc_unique_index_set_acquire_release_as_lifo_behavior() {
    let sut = FixedSizeUniqueIndexSet::<CAPACITY>::new();
    let mut ids = vec![];

    for _ in 0..CAPACITY {
        let e = sut.acquire();
        assert_that!(e, is_some);
        ids.push(e.unwrap());
    }

    for _ in 0..CAPACITY {
        ids.remove(0);
    }

    for i in 0..CAPACITY {
        let e = sut.acquire();
        assert_that!(e, is_some);
        assert_that!(e.as_ref().unwrap().value(), eq(CAPACITY - i - 1) as u32);
        ids.push(e.unwrap());
    }
}

#[test]
fn mpmc_unique_index_set_concurrent_acquire_release() {
    const REPETITIONS: i64 = 10000;
    let number_of_threads = (SystemInfo::NumberOfCpuCores.value()).clamp(2, usize::MAX);

    let sut = FixedSizeUniqueIndexSet::<CAPACITY>::new();
    let barrier = Barrier::new(number_of_threads);
    let mut result: Vec<Mutex<Vec<u64>>> = vec![];
    let thread_counter = AtomicUsize::new(0);

    for _ in 0..number_of_threads {
        result.push(Mutex::new(vec![0; CAPACITY]));
    }

    thread::scope(|s| {
        for _ in 0..number_of_threads {
            s.spawn(|| {
                let mut ids = vec![];
                let mut guard = result[thread_counter.fetch_add(1, Ordering::Relaxed)]
                    .lock()
                    .unwrap();
                let mut repetition = 0;

                barrier.wait();
                loop {
                    match sut.acquire() {
                        Some(e) => {
                            guard[e.value() as usize] += 1;
                            ids.push(e);
                        }
                        None => {
                            repetition += 1;
                            ids.clear();
                            if repetition == REPETITIONS {
                                break;
                            }
                        }
                    }
                }
            });
        }
    });

    // check if the sut is still in an consistent state
    let mut ids = vec![];
    let mut id_counter = [0u64; CAPACITY];

    for i in 0..CAPACITY {
        let e = sut.acquire();
        assert_that!(e, is_some);
        id_counter[i] += 1;
        ids.push(e.unwrap());
    }

    for i in 0..CAPACITY {
        assert_that!(id_counter[i], eq 1);
    }

    let e = sut.acquire();
    assert_that!(e, is_none);
}
