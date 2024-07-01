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

use iceoryx2_bb_posix::thread::*;
use iceoryx2_bb_testing::assert_that;
use std::sync::Arc;
use std::sync::Barrier;
use std::time::{Duration, Instant};

#[test]
fn thread_set_name_works() {
    let barrier = Arc::new(Barrier::new(2));
    let name = ThreadName::from(b"oh-a-thread");
    let thread = {
        let barrier = barrier.clone();
        ThreadBuilder::new()
            .name(&name)
            .spawn(move || {
                barrier.wait();
                let handle = ThreadHandle::from_self();
                barrier.wait();
                assert_that!(handle.get_name().unwrap(), eq b"oh-a-thread");
            })
            .unwrap()
    };

    barrier.wait();
    let name = *thread.get_name().unwrap();
    barrier.wait();
    drop(thread);

    assert_that!(name, eq b"oh-a-thread");
}

#[test]
fn thread_creation_does_not_block() {
    let barrier = Arc::new(Barrier::new(2));
    let thread = {
        let barrier = barrier.clone();
        ThreadBuilder::new()
            .spawn(move || {
                barrier.wait();
            })
            .unwrap()
    };
    barrier.wait();
    drop(thread);
}

#[test]
fn thread_affinity_is_at_least_core_zero() {
    let barrier = Arc::new(Barrier::new(2));
    let thread = {
        let barrier = barrier.clone();
        ThreadBuilder::new()
            .spawn(move || {
                barrier.wait();
                let handle = ThreadHandle::from_self();
                let affinity = handle.get_affinity().unwrap();
                barrier.wait();
                assert_that!(affinity, is_not_empty);
                assert_that!(affinity[0], eq 0);
            })
            .unwrap()
    };

    barrier.wait();
    let affinity = thread.get_affinity().unwrap();
    barrier.wait();
    assert_that!(affinity, is_not_empty);
    assert_that!(affinity[0], eq 0);
}

#[test]
fn thread_set_affinity_on_creation_works() {
    let barrier = Arc::new(Barrier::new(2));
    let thread = {
        let barrier = barrier.clone();
        ThreadBuilder::new()
            .affinity(0)
            .spawn(move || {
                barrier.wait();
                let handle = ThreadHandle::from_self();
                let affinity = handle.get_affinity().unwrap();
                barrier.wait();
                assert_that!(affinity, len 1);
                assert_that!(affinity[0], eq 0);
            })
            .unwrap()
    };

    barrier.wait();
    let affinity = thread.get_affinity().unwrap();
    barrier.wait();
    assert_that!(affinity, len 1);
    assert_that!(affinity[0], eq 0);
}

#[test]
fn thread_set_affinity_from_handle_works() {
    let barrier = Arc::new(Barrier::new(2));
    let thread = {
        let barrier = barrier.clone();
        ThreadBuilder::new()
            .affinity(0)
            .spawn(move || {
                let mut handle = ThreadHandle::from_self();
                handle.set_affinity(0).unwrap();
                barrier.wait();
                let affinity = handle.get_affinity().unwrap();
                barrier.wait();
                assert_that!(affinity, len 1);
                assert_that!(affinity[0], eq 0);
            })
            .unwrap()
    };

    barrier.wait();
    let affinity = thread.get_affinity().unwrap();
    barrier.wait();
    assert_that!(affinity, len 1);
    assert_that!(affinity[0], eq 0);
}

#[test]
fn thread_set_affinity_from_thread_works() {
    let barrier = Arc::new(Barrier::new(2));
    let mut thread = {
        let barrier = barrier.clone();
        ThreadBuilder::new()
            .affinity(0)
            .spawn(move || {
                barrier.wait();
                let handle = ThreadHandle::from_self();
                let affinity = handle.get_affinity().unwrap();
                barrier.wait();
                assert_that!(affinity, len 1);
                assert_that!(affinity[0], eq 0);
            })
            .unwrap()
    };

    thread.set_affinity(0).unwrap();
    barrier.wait();
    let affinity = thread.get_affinity().unwrap();
    barrier.wait();
    assert_that!(affinity, len 1);
    assert_that!(affinity[0], eq 0);
}

#[test]
fn thread_destructor_does_not_block_on_empty_thread() {
    let start = Instant::now();
    {
        let _thread = {
            ThreadBuilder::new()
                .spawn(|| {
                    // nothing to see, move along
                })
                .unwrap()
        };
    }
    assert_that!(start.elapsed(), lt(Duration::from_millis(10)));
}

#[test]
fn thread_destructor_does_block_on_busy_thread() {
    const SLEEP_DURATION: Duration = Duration::from_millis(100);
    let start = Instant::now();
    {
        let _thread = {
            ThreadBuilder::new()
                .spawn(|| {
                    let start = Instant::now();
                    while start.elapsed() < SLEEP_DURATION {
                        std::thread::sleep(SLEEP_DURATION - start.elapsed());
                    }
                })
                .unwrap()
        };
    }
    assert_that!(start.elapsed(), gt(SLEEP_DURATION));
}

#[test]
fn thread_cancel_works() {
    let barrier = Arc::new(Barrier::new(2));
    let mut thread = {
        let barrier = barrier.clone();
        ThreadBuilder::new()
            .affinity(0)
            .spawn(move || {
                barrier.wait();
                loop {
                    // NOTE we have to use 'sleep' instead of 'barrier.wait()' since the latter blocks cancellation
                    std::thread::sleep(Duration::from_millis(100));
                }
            })
            .unwrap()
    };

    // if the thread is not executed we observe a deadlock here
    barrier.wait();
    thread.cancel();
}

#[test]
fn thread_exit_works() {
    let _thread = {
        ThreadBuilder::new()
            .affinity(0)
            .spawn(move || {
                // TODO: [#258] This sleep prevents a 'SIGABRT' on Linux with release builds with rustc v1.81.0-nightly!
                #[cfg(target_os = "linux")]
                std::thread::sleep(Duration::from_nanos(1));
                thread_exit();
                #[allow(unreachable_code)]
                {
                    panic!("This should not happen when 'thread_exit' works as intended");
                }
            })
            .unwrap()
    };
}
