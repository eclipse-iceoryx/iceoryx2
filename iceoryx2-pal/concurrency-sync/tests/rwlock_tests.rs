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

use core::{
    sync::atomic::{AtomicU32, Ordering},
    time::Duration,
};

use iceoryx2_pal_concurrency_sync::{barrier::Barrier, rwlock::*, WaitAction, WaitResult};
use iceoryx2_pal_testing::assert_that;

const TIMEOUT: Duration = Duration::from_millis(25);

#[test]
fn rwlock_reader_preference_try_write_lock_blocks_read_locks() {
    let sut = RwLockReaderPreference::new();

    assert_that!(sut.try_write_lock(), eq WaitResult::Success);
    assert_that!(sut.try_write_lock(), eq WaitResult::Interrupted);
    assert_that!(sut.write_lock(|_, _| WaitAction::Abort), eq WaitResult::Interrupted);

    assert_that!(sut.try_read_lock(), eq WaitResult::Interrupted);
    assert_that!(sut.read_lock(|_, _| WaitAction::Abort), eq WaitResult::Interrupted);
}

#[test]
fn rwlock_reader_preference_multiple_read_locks_block_write_lock() {
    let sut = RwLockReaderPreference::new();

    assert_that!(sut.try_read_lock(), eq WaitResult::Success);
    assert_that!(sut.try_read_lock(), eq WaitResult::Success);
    assert_that!(sut.read_lock(|_, _| WaitAction::Abort), eq WaitResult::Success);
    assert_that!(sut.read_lock(|_, _| WaitAction::Abort), eq WaitResult::Success);

    assert_that!(sut.try_write_lock(), eq WaitResult::Interrupted);
    assert_that!(sut.write_lock(|_, _| WaitAction::Abort), eq WaitResult::Interrupted);
}

#[test]
fn rwlock_reader_preference_write_lock_and_unlock_works() {
    let sut = RwLockReaderPreference::new();

    assert_that!(sut.write_lock(|_, _| WaitAction::Abort), eq WaitResult::Success);

    assert_that!(sut.try_write_lock(), eq WaitResult::Interrupted);
    assert_that!(sut.try_read_lock(), eq WaitResult::Interrupted);
    assert_that!(sut.read_lock(|_, _| WaitAction::Abort), eq WaitResult::Interrupted);

    sut.unlock(|_| {});

    assert_that!(sut.try_write_lock(), eq WaitResult::Success);

    assert_that!(sut.write_lock(|_, _| WaitAction::Abort), eq WaitResult::Interrupted);
    assert_that!(sut.try_read_lock(), eq WaitResult::Interrupted);
    assert_that!(sut.read_lock(|_, _| WaitAction::Abort), eq WaitResult::Interrupted);

    sut.unlock(|_| {});

    assert_that!(sut.write_lock(|_, _| WaitAction::Abort), eq WaitResult::Success);
}

#[test]
fn rwlock_reader_preference_try_read_lock_and_unlock_works() {
    const NUMBER_OF_READ_LOCKS: usize = 123;
    let sut = RwLockReaderPreference::new();

    for _ in 0..NUMBER_OF_READ_LOCKS {
        assert_that!(sut.try_read_lock(), eq WaitResult::Success);
        assert_that!(sut.try_write_lock(), eq WaitResult::Interrupted);
        assert_that!(sut.write_lock(|_, _| WaitAction::Abort), eq WaitResult::Interrupted);
    }

    for _ in 0..NUMBER_OF_READ_LOCKS {
        sut.unlock(|_| {});
    }

    assert_that!(sut.try_write_lock(), eq WaitResult::Success);
}

#[test]
fn rwlock_reader_preference_read_lock_and_unlock_works() {
    const NUMBER_OF_READ_LOCKS: usize = 67;
    let sut = RwLockReaderPreference::new();

    for _ in 0..NUMBER_OF_READ_LOCKS {
        assert_that!(sut.read_lock(|_, _| WaitAction::Abort), eq WaitResult::Success);
        assert_that!(sut.try_write_lock(), eq WaitResult::Interrupted);
        assert_that!(sut.write_lock(|_, _| WaitAction::Abort), eq WaitResult::Interrupted);
    }

    for _ in 0..NUMBER_OF_READ_LOCKS {
        sut.unlock(|_| {});
    }

    assert_that!(sut.write_lock(|_, _| WaitAction::Abort), eq WaitResult::Success);
}

#[test]
fn rwlock_reader_preference_read_lock_blocks_only_write_locks() {
    const READ_THREADS: u32 = 4;
    const WRITE_THREADS: u32 = 4;

    let sut = RwLockReaderPreference::new();
    let barrier = Barrier::new(READ_THREADS + WRITE_THREADS + 1);
    let barrier_read = Barrier::new(READ_THREADS + 1);
    let barrier_write = Barrier::new(WRITE_THREADS + 1);

    let read_counter = AtomicU32::new(0);
    let write_counter = AtomicU32::new(0);

    std::thread::scope(|s| {
        assert_that!(sut.try_read_lock(), eq WaitResult::Success);
        for _ in 0..WRITE_THREADS {
            s.spawn(|| {
                barrier.wait(|_, _| {}, |_| {});
                sut.write_lock(|_, _| WaitAction::Continue);
                write_counter.fetch_add(1, Ordering::Relaxed);
                sut.unlock(|_| {});
                barrier_write.wait(|_, _| {}, |_| {});
            });
        }

        for _ in 0..READ_THREADS {
            s.spawn(|| {
                barrier.wait(|_, _| {}, |_| {});
                sut.read_lock(|_, _| WaitAction::Continue);
                read_counter.fetch_add(1, Ordering::Relaxed);
                barrier_read.wait(|_, _| {}, |_| {});
                sut.unlock(|_| {});
            });
        }

        let read_counter_old_1 = read_counter.load(Ordering::Relaxed);
        let write_counter_old_1 = write_counter.load(Ordering::Relaxed);
        barrier.wait(|_, _| {}, |_| {});

        barrier_read.wait(|_, _| {}, |_| {});
        let read_counter_old_2 = read_counter.load(Ordering::Relaxed);
        let write_counter_old_2 = write_counter.load(Ordering::Relaxed);

        sut.unlock(|_| {});
        barrier_write.wait(|_, _| {}, |_| {});

        assert_that!(read_counter_old_1, eq 0);
        assert_that!(write_counter_old_1, eq 0);
        assert_that!(read_counter_old_2, eq READ_THREADS);
        assert_that!(write_counter_old_2, eq 0);
        assert_that!(write_counter.load(Ordering::Relaxed), eq WRITE_THREADS);
    });
}

#[test]
fn rwlock_reader_preference_write_lock_blocks_everything() {
    const READ_THREADS: u32 = 4;
    const WRITE_THREADS: u32 = 4;

    let sut = RwLockReaderPreference::new();
    let barrier = Barrier::new(READ_THREADS + WRITE_THREADS + 1);
    let barrier_end = Barrier::new(READ_THREADS + WRITE_THREADS + 1);

    let read_counter = AtomicU32::new(0);
    let write_counter = AtomicU32::new(0);

    std::thread::scope(|s| {
        assert_that!(sut.try_write_lock(), eq WaitResult::Success);
        for _ in 0..WRITE_THREADS {
            s.spawn(|| {
                barrier.wait(|_, _| {}, |_| {});
                sut.write_lock(|_, _| WaitAction::Continue);
                let current_read_counter = read_counter.load(Ordering::Relaxed);
                write_counter.fetch_add(1, Ordering::Relaxed);
                std::thread::sleep(TIMEOUT);
                let test_result = current_read_counter == read_counter.load(Ordering::Relaxed);
                sut.unlock(|_| {});

                barrier_end.wait(|_, _| {}, |_| {});
                assert_that!(test_result, eq true);
            });
        }

        for _ in 0..READ_THREADS {
            s.spawn(|| {
                barrier.wait(|_, _| {}, |_| {});
                sut.read_lock(|_, _| WaitAction::Continue);
                read_counter.fetch_add(1, Ordering::Relaxed);
                sut.unlock(|_| {});

                barrier_end.wait(|_, _| {}, |_| {});
            });
        }

        let read_counter_old_1 = read_counter.load(Ordering::Relaxed);
        let write_counter_old_1 = write_counter.load(Ordering::Relaxed);
        barrier.wait(|_, _| {}, |_| {});

        std::thread::sleep(TIMEOUT);
        let read_counter_old_2 = read_counter.load(Ordering::Relaxed);
        let write_counter_old_2 = write_counter.load(Ordering::Relaxed);

        sut.unlock(|_| {});

        barrier_end.wait(|_, _| {}, |_| {});
        assert_that!(read_counter_old_1, eq 0);
        assert_that!(write_counter_old_1, eq 0);
        assert_that!(read_counter_old_2, eq 0);
        assert_that!(write_counter_old_2, eq 0);
        assert_that!(read_counter.load(Ordering::Relaxed), eq READ_THREADS);
        assert_that!(write_counter.load(Ordering::Relaxed), eq WRITE_THREADS);
    });
}

//////////////////////
/// Writer Preference
//////////////////////

#[test]
fn rwlock_writer_preference_try_write_lock_blocks_read_locks() {
    let sut = RwLockWriterPreference::new();

    assert_that!(sut.try_write_lock(), eq WaitResult::Success);
    assert_that!(sut.try_write_lock(), eq WaitResult::Interrupted);
    assert_that!(sut.write_lock(|_, _| WaitAction::Abort, |_| {}, |_| {}), eq WaitResult::Interrupted);

    assert_that!(sut.try_read_lock(), eq WaitResult::Interrupted);
    assert_that!(sut.read_lock(|_, _| WaitAction::Abort), eq WaitResult::Interrupted);
}

#[test]
fn rwlock_writer_preference_multiple_read_locks_block_write_lock() {
    let sut = RwLockWriterPreference::new();

    assert_that!(sut.try_read_lock(), eq WaitResult::Success);
    assert_that!(sut.try_read_lock(), eq WaitResult::Success);
    assert_that!(sut.read_lock(|_, _| WaitAction::Abort), eq WaitResult::Success);
    assert_that!(sut.read_lock(|_, _| WaitAction::Abort), eq WaitResult::Success);

    assert_that!(sut.try_write_lock(), eq WaitResult::Interrupted);
    assert_that!(sut.write_lock(|_, _| WaitAction::Abort, |_| {}, |_| {}), eq WaitResult::Interrupted);
}

#[test]
fn rwlock_writer_preference_write_lock_and_unlock_works() {
    let sut = RwLockWriterPreference::new();

    assert_that!(sut.write_lock(|_, _| WaitAction::Abort, |_| {}, |_| {}), eq WaitResult::Success);

    assert_that!(sut.try_write_lock(), eq WaitResult::Interrupted);
    assert_that!(sut.try_read_lock(), eq WaitResult::Interrupted);
    assert_that!(sut.read_lock(|_, _| WaitAction::Abort), eq WaitResult::Interrupted);

    sut.unlock(|_| {}, |_| {});

    assert_that!(sut.try_write_lock(), eq WaitResult::Success);

    assert_that!(sut.write_lock(|_, _| WaitAction::Abort, |_| {}, |_| {}), eq WaitResult::Interrupted);
    assert_that!(sut.try_read_lock(), eq WaitResult::Interrupted);
    assert_that!(sut.read_lock(|_, _| WaitAction::Abort), eq WaitResult::Interrupted);

    sut.unlock(|_| {}, |_| {});

    assert_that!(sut.write_lock(|_, _| WaitAction::Abort, |_| {}, |_| {}), eq WaitResult::Success);
}

#[test]
fn rwlock_writer_preference_try_read_lock_and_unlock_works() {
    const NUMBER_OF_READ_LOCKS: usize = 123;
    let sut = RwLockWriterPreference::new();

    for _ in 0..NUMBER_OF_READ_LOCKS {
        assert_that!(sut.try_read_lock(), eq WaitResult::Success);
        assert_that!(sut.try_write_lock(), eq WaitResult::Interrupted);
        assert_that!(sut.write_lock(|_, _| WaitAction::Abort, |_| {}, |_| {}), eq WaitResult::Interrupted);
    }

    for _ in 0..NUMBER_OF_READ_LOCKS {
        sut.unlock(|_| {}, |_| {});
    }

    assert_that!(sut.try_write_lock(), eq WaitResult::Success);
}

#[test]
fn rwlock_writer_preference_read_lock_and_unlock_works() {
    const NUMBER_OF_READ_LOCKS: usize = 67;
    let sut = RwLockWriterPreference::new();

    for _ in 0..NUMBER_OF_READ_LOCKS {
        assert_that!(sut.read_lock(|_, _| WaitAction::Abort), eq WaitResult::Success);
        assert_that!(sut.try_write_lock(), eq WaitResult::Interrupted);
        assert_that!(sut.write_lock(|_, _| WaitAction::Abort, |_| {}, |_| {}), eq WaitResult::Interrupted);
    }

    for _ in 0..NUMBER_OF_READ_LOCKS {
        sut.unlock(|_| {}, |_| {});
    }

    assert_that!(sut.write_lock(|_, _| WaitAction::Abort, |_| {}, |_| {}), eq WaitResult::Success);
}

#[test]
fn rwlock_writer_preference_write_lock_blocks_everything() {
    const READ_THREADS: u32 = 4;
    const WRITE_THREADS: u32 = 4;

    let sut = RwLockWriterPreference::new();
    let barrier = Barrier::new(READ_THREADS + WRITE_THREADS + 1);
    let barrier_end = Barrier::new(READ_THREADS + WRITE_THREADS + 1);

    let read_counter = AtomicU32::new(0);
    let write_counter = AtomicU32::new(0);

    std::thread::scope(|s| {
        assert_that!(sut.try_write_lock(), eq WaitResult::Success);
        for _ in 0..WRITE_THREADS {
            s.spawn(|| {
                barrier.wait(|_, _| {}, |_| {});
                sut.write_lock(|_, _| WaitAction::Continue, |_| {}, |_| {});
                let current_read_counter = read_counter.load(Ordering::Relaxed);
                write_counter.fetch_add(1, Ordering::Relaxed);
                std::thread::sleep(TIMEOUT);
                let test_result = current_read_counter == read_counter.load(Ordering::Relaxed);
                sut.unlock(|_| {}, |_| {});

                barrier_end.wait(|_, _| {}, |_| {});
                assert_that!(test_result, eq true);
            });
        }

        for _ in 0..READ_THREADS {
            s.spawn(|| {
                barrier.wait(|_, _| {}, |_| {});
                sut.read_lock(|_, _| WaitAction::Continue);
                read_counter.fetch_add(1, Ordering::Relaxed);
                sut.unlock(|_| {}, |_| {});

                barrier_end.wait(|_, _| {}, |_| {});
            });
        }

        let read_counter_old_1 = read_counter.load(Ordering::Relaxed);
        let write_counter_old_1 = write_counter.load(Ordering::Relaxed);
        barrier.wait(|_, _| {}, |_| {});

        std::thread::sleep(TIMEOUT);
        let read_counter_old_2 = read_counter.load(Ordering::Relaxed);
        let write_counter_old_2 = write_counter.load(Ordering::Relaxed);

        sut.unlock(|_| {}, |_| {});

        barrier_end.wait(|_, _| {}, |_| {});

        assert_that!(read_counter_old_1, eq 0);
        assert_that!(write_counter_old_1, eq 0);
        assert_that!(read_counter_old_2, eq 0);
        assert_that!(write_counter_old_2, eq 0);
        assert_that!(read_counter.load(Ordering::Relaxed), eq READ_THREADS);
        assert_that!(write_counter.load(Ordering::Relaxed), eq WRITE_THREADS);
    });
}
