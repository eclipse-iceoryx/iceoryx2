// Copyright (c) 2026 Contributors to the Eclipse Foundation
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

use iceoryx2_bb_container::atomic_memcpy::*;
use iceoryx2_bb_container::string::StaticString;
use iceoryx2_bb_testing_macros::test;
// use iceoryx2_bb_posix::barrier::*;
// use iceoryx2_bb_posix::thread::thread_scope;
use iceoryx2_bb_testing::assert_that;
use std::sync::Barrier;
use std::thread;

#[test]
pub fn atomic_memcpy_cannot_be_created_when_sizes_do_not_match() {
    const SIZE: usize = size_of::<u64>();
    let value: u8 = 0;
    let sut = AtomicMemcpy::<u8, SIZE>::new(value);
    assert_that!(sut, is_err);
    assert_that!(sut.err().unwrap(), eq AtomicMemcpyError::AtomicMemcpyCreateError);
}

// TODO: test other types
#[test]
pub fn new_creates_atomic_memcpy_containing_passed_value() {
    const SIZE: usize = size_of::<u64>();
    let value = 963;
    let sut = AtomicMemcpy::<u64, SIZE>::new(value);
    assert_that!(sut, is_ok);

    let read_value = unsafe { sut.unwrap().read().assume_init() };
    assert_that!(read_value, eq value);
}

#[test]
pub fn new_creates_atomic_memcpy_containing_passed_static_string() {
    const SIZE: usize = size_of::<StaticString<6>>();
    let value = StaticString::<6>::try_from("ato").unwrap();
    let sut = AtomicMemcpy::<StaticString<6>, SIZE>::new(value);
    assert_that!(sut, is_ok);

    let read_value = unsafe { sut.unwrap().read().assume_init() };
    assert_that!(read_value, eq value.as_bytes_const());
}

#[test]
pub fn atomic_memcpy_contains_passed_value_after_write() {
    const SIZE: usize = size_of::<u64>();
    let sut = AtomicMemcpy::<u64, SIZE>::new(0).unwrap();

    let new_value: u64 = 752389;
    unsafe {
        sut.write(new_value);
        assert_that!(sut.read().assume_init(), eq new_value);
    }
}

#[test]
pub fn atomic_memcpy_contains_passed_static_string_after_write() {
    const SIZE: usize = size_of::<StaticString<20>>();
    let sut = AtomicMemcpy::<StaticString<20>, SIZE>::new(StaticString::<20>::new()).unwrap();

    let new_value = StaticString::<20>::try_from("atomheartmother").unwrap();
    unsafe {
        sut.write(new_value);
        assert_that!(sut.read().assume_init(), eq new_value);
    }
}

// TODO: requires_std threading + synchronization?
#[test]
pub fn concurrent_read_without_write_always_returns_correct_data() {
    const SIZE: usize = size_of::<u64>();
    let value = 481935403;
    let sut = AtomicMemcpy::<u64, SIZE>::new(value).unwrap();

    let number_of_threads = 16;
    const REPETITIONS: usize = 500;
    // Use of std thread + barrier because Miri fails with "error: unsupported operation: can't call foreign function"
    // for `pthread_attr_init` and `pthread_barrieratrr_init` (iceoryx2-pal/posix/src/linux/pthread.rs)
    let barrier = Barrier::new(number_of_threads);
    thread::scope(|s| {
        for _ in 0..number_of_threads {
            s.spawn(|| {
                barrier.wait();
                for _ in 0..REPETITIONS {
                    unsafe {
                        let read_value = sut.read();
                        assert_that!(read_value.assume_init(), eq value);
                    }
                }
            });
        }
    });
}

#[test]
pub fn concurrent_write_does_not_trigger_ub_with_miri() {
    const SIZE: usize = size_of::<u64>();
    let value = u64::MAX;
    let sut = AtomicMemcpy::<u64, SIZE>::new(value).unwrap();

    let number_of_threads = 16;
    const REPETITIONS: usize = 500;
    let barrier = Barrier::new(number_of_threads);
    thread::scope(|s| {
        for _ in 0..number_of_threads {
            s.spawn(|| {
                barrier.wait();
                for _ in 0..REPETITIONS {
                    unsafe {
                        sut.write(value);
                    }
                }
            });
        }
    });

    unsafe {
        let read_value = sut.read();
        assert_that!(read_value.assume_init(), eq value);
    }
}

#[test]
pub fn concurrent_read_and_write_does_not_trigger_ub_with_miri() {
    const SIZE: usize = size_of::<u64>();
    let value = 3249780;
    let sut = AtomicMemcpy::<u64, SIZE>::new(value).unwrap();

    let number_of_threads = 16;
    const REPETITIONS: usize = 500;
    let barrier = Barrier::new(number_of_threads);
    thread::scope(|s| {
        for _ in 0..number_of_threads / 2 {
            s.spawn(|| {
                barrier.wait();
                for _ in 0..REPETITIONS {
                    unsafe {
                        let read_value = sut.read();
                        // dummy assert to prevent the read operation to be optimized away
                        assert_that!(core::mem::size_of_val(&read_value), eq SIZE);
                    }
                }
            });
        }

        for _ in 0..number_of_threads / 2 {
            s.spawn(|| {
                barrier.wait();
                for _ in 0..REPETITIONS {
                    unsafe {
                        sut.write(value);
                    }
                }
            });
        }
    });
}
