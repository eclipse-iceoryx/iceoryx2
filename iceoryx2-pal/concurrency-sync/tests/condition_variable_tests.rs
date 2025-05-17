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
    hint::spin_loop,
    sync::atomic::{AtomicBool, AtomicU32, Ordering},
    time::Duration,
};

use iceoryx2_pal_concurrency_sync::{
    barrier::Barrier, condition_variable::*, WaitAction, WaitResult,
};
use iceoryx2_pal_testing::{assert_that, watchdog::Watchdog};

const TIMEOUT: Duration = Duration::from_millis(25);

struct ThreadInWait<const NUMBER_OF_THREADS: usize> {
    thread_id: AtomicU32,
    thread_in_wait: [AtomicBool; NUMBER_OF_THREADS],
}

impl<const NUMBER_OF_THREADS: usize> ThreadInWait<NUMBER_OF_THREADS> {
    fn new() -> Self {
        const FALSE: AtomicBool = AtomicBool::new(false);
        Self {
            thread_id: AtomicU32::new(0),
            thread_in_wait: [FALSE; NUMBER_OF_THREADS],
        }
    }

    fn get_id(&self) -> usize {
        self.thread_id.fetch_add(1, Ordering::Relaxed) as usize
    }

    fn signal_is_in_wait(&self, id: usize) {
        self.thread_in_wait[id].store(true, Ordering::Relaxed);
    }

    fn block_until_all_threads_are_waiting(&self) {
        loop {
            let mut wait_for_thread = false;
            for v in &self.thread_in_wait {
                if !v.load(Ordering::Relaxed) {
                    wait_for_thread = true;
                    break;
                }
            }

            if !wait_for_thread {
                break;
            }
        }
    }
}

#[test]
fn condition_variable_notify_one_unblocks_one() {
    const NUMBER_OF_THREADS: usize = 3;
    let _watchdog = Watchdog::new();
    let barrier = Barrier::new(NUMBER_OF_THREADS as u32 + 1);
    let sut = ConditionVariable::new();
    let mtx = Mutex::new();
    let counter = AtomicU32::new(0);
    let triggered_thread = AtomicU32::new(0);
    let thread_in_wait = ThreadInWait::<NUMBER_OF_THREADS>::new();

    std::thread::scope(|s| {
        for _ in 0..NUMBER_OF_THREADS {
            s.spawn(|| {
                barrier.wait(|_, _| {}, |_| {});
                mtx.lock(|_, _| WaitAction::Continue);
                let id = thread_in_wait.get_id();
                let wait_result = sut.wait(
                    &mtx,
                    |_| {},
                    |_, _| {
                        thread_in_wait.signal_is_in_wait(id);
                        while triggered_thread.load(Ordering::Relaxed) < 1 {
                            spin_loop()
                        }
                        WaitAction::Continue
                    },
                    |_, _| WaitAction::Continue,
                );
                counter.fetch_add(1, Ordering::Relaxed);
                mtx.unlock(|_| {});
                assert_that!(wait_result, eq WaitResult::Success);
            });
        }

        barrier.wait(|_, _| {}, |_| {});
        thread_in_wait.block_until_all_threads_are_waiting();
        std::thread::sleep(TIMEOUT);
        let counter_old = counter.load(Ordering::Relaxed);

        for i in 0..NUMBER_OF_THREADS {
            sut.notify_one(|_| {
                triggered_thread.fetch_add(1, Ordering::Relaxed);
            });

            // this can cause a deadlock but the watchdog takes care of it
            while counter.load(Ordering::Relaxed) as usize <= i {}
        }

        assert_that!(counter_old, eq 0);
    });
}

#[test]
fn condition_variable_notify_all_unblocks_all() {
    const NUMBER_OF_THREADS: usize = 5;
    let _watchdog = Watchdog::new();
    let barrier = Barrier::new(NUMBER_OF_THREADS as u32 + 1);
    let sut = ConditionVariable::new();
    let mtx = Mutex::new();
    let counter = AtomicU32::new(0);
    let triggered_thread = AtomicU32::new(0);
    let thread_in_wait = ThreadInWait::<NUMBER_OF_THREADS>::new();

    std::thread::scope(|s| {
        let mut threads = vec![];
        for _ in 0..NUMBER_OF_THREADS {
            threads.push(s.spawn(|| {
                barrier.wait(|_, _| {}, |_| {});
                mtx.lock(|_, _| WaitAction::Continue);
                let id = thread_in_wait.get_id();
                let wait_result = sut.wait(
                    &mtx,
                    |_| {},
                    |_, _| {
                        thread_in_wait.signal_is_in_wait(id);
                        while triggered_thread.load(Ordering::Relaxed) < 1 {
                            spin_loop()
                        }
                        WaitAction::Continue
                    },
                    |_, _| WaitAction::Continue,
                );
                counter.fetch_add(1, Ordering::Relaxed);
                mtx.unlock(|_| {});
                assert_that!(wait_result, eq WaitResult::Success);
            }));
        }

        barrier.wait(|_, _| {}, |_| {});

        thread_in_wait.block_until_all_threads_are_waiting();
        std::thread::sleep(TIMEOUT);
        let counter_old = counter.load(Ordering::Relaxed);

        sut.notify_all(|_| {
            triggered_thread.fetch_add(1, Ordering::Relaxed);
        });

        for t in threads {
            t.join().unwrap();
        }

        assert_that!(counter_old, eq 0);
        assert_that!(counter.load(Ordering::Relaxed), eq NUMBER_OF_THREADS as u32);
    });
}

#[test]
fn condition_variable_mutex_is_locked_when_wait_returns() {
    const NUMBER_OF_THREADS: usize = 5;
    let _watchdog = Watchdog::new();
    let barrier = Barrier::new(NUMBER_OF_THREADS as u32 + 1);
    let sut = ConditionVariable::new();
    let mtx = Mutex::new();
    let counter = AtomicU32::new(0);
    let triggered_thread = AtomicU32::new(0);
    let thread_in_wait = ThreadInWait::<NUMBER_OF_THREADS>::new();

    std::thread::scope(|s| {
        for _ in 0..NUMBER_OF_THREADS {
            s.spawn(|| {
                barrier.wait(|_, _| {}, |_| {});
                mtx.lock(|_, _| WaitAction::Continue);
                let id = thread_in_wait.get_id();
                let wait_result = sut.wait(
                    &mtx,
                    |_| {},
                    |_, _| {
                        thread_in_wait.signal_is_in_wait(id);
                        while triggered_thread.load(Ordering::Relaxed) < 1 {
                            spin_loop()
                        }
                        WaitAction::Continue
                    },
                    |_, _| WaitAction::Continue,
                );
                counter.fetch_add(1, Ordering::Relaxed);
                assert_that!(wait_result, eq WaitResult::Success);
                assert_that!(mtx.try_lock(), eq WaitResult::Interrupted);
                // unlock thread since we own it
                mtx.unlock(|_| {});
            });
        }

        barrier.wait(|_, _| {}, |_| {});

        thread_in_wait.block_until_all_threads_are_waiting();
        std::thread::sleep(TIMEOUT);
        let counter_old = counter.load(Ordering::Relaxed);

        sut.notify_all(|_| {
            triggered_thread.fetch_add(1, Ordering::Relaxed);
        });
        std::thread::sleep(TIMEOUT);

        assert_that!(counter_old, eq 0);
    });
}

#[test]
fn condition_variable_wait_returns_false_when_functor_returns_false() {
    let sut = ConditionVariable::new();
    let mtx = Mutex::new();
    mtx.lock(|_, _| WaitAction::Continue);
    assert_that!(sut.wait(&mtx, |_| {}, |_, _| WaitAction::Abort, |_, _| WaitAction::Continue), eq WaitResult::Interrupted);
    mtx.unlock(|_| {});
}
