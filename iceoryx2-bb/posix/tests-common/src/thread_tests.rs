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

#![allow(clippy::disallowed_types)]

use alloc::sync::Arc;
use core::time::Duration;

use iceoryx2_bb_concurrency::atomic::{AtomicU32, AtomicU64, Ordering};
use iceoryx2_bb_posix::barrier::BarrierBuilder;
use iceoryx2_bb_posix::barrier::BarrierHandle;
use iceoryx2_bb_posix::barrier::Handle;
use iceoryx2_bb_posix::clock::nanosleep;
use iceoryx2_bb_posix::clock::Time;
use iceoryx2_bb_posix::system_configuration::SystemInfo;
use iceoryx2_bb_posix::thread::thread_scope;
use iceoryx2_bb_posix::thread::ThreadBuilder;
use iceoryx2_bb_posix::thread::ThreadHandle;
use iceoryx2_bb_posix::thread::ThreadName;
use iceoryx2_bb_posix::thread::ThreadProperties;
use iceoryx2_bb_posix::thread::ThreadSetAffinityError;
use iceoryx2_bb_posix::thread::ThreadSpawnError;
use iceoryx2_bb_posix::thread::MAX_SCOPED_THREADS;
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing::test_requires;
use iceoryx2_bb_testing::watchdog::Watchdog;
use iceoryx2_bb_testing_macros::inventory_test;
use iceoryx2_pal_posix::posix::{self, POSIX_SUPPORT_CPU_AFFINITY};

struct SpinBarrier {
    counter: AtomicU32,
}

impl SpinBarrier {
    fn new() -> Arc<Self> {
        Arc::new(Self {
            counter: AtomicU32::new(0),
        })
    }

    fn wait(&self) {
        let before = self.counter.fetch_add(1, Ordering::SeqCst);
        let round = before / 2;
        let target = (round + 1) * 2;
        while self.counter.load(Ordering::SeqCst) < target {
            core::hint::spin_loop();
        }
    }
}

#[inventory_test]
pub fn thread_set_name_works() {
    let name = ThreadName::try_from(b"oh-a-thread").unwrap();
    let barrier = SpinBarrier::new();
    let barrier_clone = barrier.clone();
    let thread = ThreadBuilder::new()
        .name(&name)
        .spawn(move || {
            barrier_clone.wait();
            let handle = ThreadHandle::from_self();
            barrier_clone.wait();
            assert_that!(handle.get_name().unwrap(), eq b"oh-a-thread");
        })
        .unwrap();

    barrier.wait();
    let name = *thread.get_name().unwrap();
    barrier.wait();
    drop(thread);

    assert_that!(name, eq b"oh-a-thread");
}

#[inventory_test]
pub fn thread_creation_does_not_block() {
    let barrier = SpinBarrier::new();
    let barrier_clone = barrier.clone();
    let thread = ThreadBuilder::new()
        .spawn(move || {
            barrier_clone.wait();
        })
        .unwrap();
    barrier.wait();
    drop(thread);
}

#[inventory_test]
pub fn thread_affinity_is_set_to_all_existing_cores_when_nothing_was_configured() {
    test_requires!(POSIX_SUPPORT_CPU_AFFINITY);
    let _watchdog = Watchdog::new();
    let number_of_cpu_cores = SystemInfo::NumberOfCpuCores.value();
    let barrier = SpinBarrier::new();
    let barrier_clone = barrier.clone();
    let thread = ThreadBuilder::new()
        .spawn(move || {
            barrier_clone.wait();
            let handle = ThreadHandle::from_self();
            let affinity = handle.get_affinity().unwrap();
            barrier_clone.wait();
            for core in 0..number_of_cpu_cores {
                assert_that!(affinity, contains core);
            }
        })
        .unwrap();

    barrier.wait();
    let affinity = thread.get_affinity().unwrap();
    barrier.wait();

    for core in 0..number_of_cpu_cores {
        assert_that!(affinity, contains core);
    }
}

#[inventory_test]
pub fn thread_set_affinity_to_one_cpu_core_on_creation_works() {
    test_requires!(POSIX_SUPPORT_CPU_AFFINITY);
    let _watchdog = Watchdog::new();
    let barrier = SpinBarrier::new();
    let barrier_clone = barrier.clone();
    let thread = ThreadBuilder::new()
        .affinity(&[0])
        .spawn(move || {
            barrier_clone.wait();
            let handle = ThreadHandle::from_self();
            let affinity = handle.get_affinity().unwrap();
            barrier_clone.wait();
            assert_that!(affinity, len 1);
            assert_that!(affinity[0], eq 0);
        })
        .unwrap();

    barrier.wait();
    let affinity = thread.get_affinity().unwrap();
    barrier.wait();
    assert_that!(affinity, len 1);
    assert_that!(affinity[0], eq 0);
}

#[inventory_test]
pub fn thread_set_affinity_to_two_cpu_cores_on_creation_works() {
    test_requires!(POSIX_SUPPORT_CPU_AFFINITY);
    test_requires!(SystemInfo::NumberOfCpuCores.value() > 1);
    let _watchdog = Watchdog::new();
    let barrier = SpinBarrier::new();
    let barrier_clone = barrier.clone();
    let thread = ThreadBuilder::new()
        .affinity(&[0, 1])
        .spawn(move || {
            barrier_clone.wait();
            let handle = ThreadHandle::from_self();
            let affinity = handle.get_affinity().unwrap();
            barrier_clone.wait();
            assert_that!(affinity, len 2);
            assert_that!(affinity, contains 0);
            assert_that!(affinity, contains 1);
        })
        .unwrap();

    barrier.wait();
    let affinity = thread.get_affinity().unwrap();
    barrier.wait();

    assert_that!(affinity, len 2);
    assert_that!(affinity, contains 0);
    assert_that!(affinity, contains 1);
}

#[inventory_test]
pub fn thread_set_affinity_to_non_existing_cpu_cores_on_creation_fails() {
    test_requires!(POSIX_SUPPORT_CPU_AFFINITY);
    let _watchdog = Watchdog::new();
    let number_of_cpu_cores = SystemInfo::NumberOfCpuCores.value();
    let thread = ThreadBuilder::new()
        .affinity(&[number_of_cpu_cores + 1])
        .spawn(|| {});

    assert_that!(thread, is_err);
    assert_that!(thread.err(), eq Some(ThreadSpawnError::CpuCoreOutsideOfSupportedCpuRangeForAffinity));
}

#[inventory_test]
pub fn thread_set_affinity_to_cores_greater_than_cpu_set_size_fails() {
    test_requires!(POSIX_SUPPORT_CPU_AFFINITY);
    let _watchdog = Watchdog::new();
    let thread = ThreadBuilder::new()
        .affinity(&[posix::CPU_SETSIZE])
        .spawn(|| {});

    assert_that!(thread, is_err);
    assert_that!(thread.err(), eq Some(ThreadSpawnError::CpuCoreOutsideOfSupportedCpuRangeForAffinity));
}

#[inventory_test]
pub fn thread_set_affinity_to_one_core_from_handle_works() {
    test_requires!(POSIX_SUPPORT_CPU_AFFINITY);
    let _watchdog = Watchdog::new();
    let barrier = SpinBarrier::new();
    let barrier_clone = barrier.clone();
    let thread = ThreadBuilder::new()
        .spawn(move || {
            let mut handle = ThreadHandle::from_self();
            handle.set_affinity(&[0]).unwrap();
            barrier_clone.wait();
            let affinity = handle.get_affinity().unwrap();
            barrier_clone.wait();
            assert_that!(affinity, len 1);
            assert_that!(affinity[0], eq 0);
        })
        .unwrap();

    barrier.wait();
    let affinity = thread.get_affinity().unwrap();
    barrier.wait();
    assert_that!(affinity, len 1);
    assert_that!(affinity[0], eq 0);
}

#[inventory_test]
pub fn thread_set_affinity_to_two_cores_from_handle_works() {
    test_requires!(POSIX_SUPPORT_CPU_AFFINITY);
    test_requires!(SystemInfo::NumberOfCpuCores.value() > 1);
    let _watchdog = Watchdog::new();
    let barrier = SpinBarrier::new();
    let barrier_clone = barrier.clone();
    let thread = ThreadBuilder::new()
        .spawn(move || {
            let mut handle = ThreadHandle::from_self();
            handle.set_affinity(&[0, 1]).unwrap();
            barrier_clone.wait();
            let affinity = handle.get_affinity().unwrap();
            barrier_clone.wait();
            assert_that!(affinity, len 2);
            assert_that!(affinity, contains 0);
            assert_that!(affinity, contains 1);
        })
        .unwrap();

    barrier.wait();
    let affinity = thread.get_affinity().unwrap();
    barrier.wait();

    assert_that!(affinity, len 2);
    assert_that!(affinity, contains 0);
    assert_that!(affinity, contains 1);
}

#[inventory_test]
pub fn thread_set_affinity_to_non_existing_cores_from_handle_fails() {
    test_requires!(POSIX_SUPPORT_CPU_AFFINITY);
    let _watchdog = Watchdog::new();
    let number_of_cpu_cores = SystemInfo::NumberOfCpuCores.value();
    let barrier = SpinBarrier::new();
    let barrier_clone = barrier.clone();
    let thread = ThreadBuilder::new()
        .spawn(move || {
            // thread is started
            barrier_clone.wait();
            let mut handle = ThreadHandle::from_self();

            let original_affinity = handle.get_affinity().unwrap();
            barrier_clone.wait();

            let result = handle.set_affinity(&[number_of_cpu_cores + 1]);
            assert_that!(result, is_err);
            assert_that!(result.err(), eq Some(ThreadSetAffinityError::InvalidCpuCores));

            barrier_clone.wait();
            let affinity = handle.get_affinity().unwrap();
            barrier_clone.wait();
            assert_that!(original_affinity, eq affinity);
        })
        .unwrap();

    // thread is started
    barrier.wait();

    // acquire original affinity
    let original_affinity = thread.get_affinity().unwrap();
    barrier.wait();

    barrier.wait();
    let affinity = thread.get_affinity().unwrap();
    barrier.wait();

    assert_that!(affinity, eq original_affinity);
}

#[inventory_test]
pub fn thread_set_affinity_to_one_core_from_thread_works() {
    test_requires!(POSIX_SUPPORT_CPU_AFFINITY);
    let _watchdog = Watchdog::new();
    let barrier = SpinBarrier::new();
    let barrier_clone = barrier.clone();
    let mut thread = ThreadBuilder::new()
        .spawn(move || {
            barrier_clone.wait();
            let handle = ThreadHandle::from_self();
            let affinity = handle.get_affinity().unwrap();
            barrier_clone.wait();
            assert_that!(affinity, len 1);
            assert_that!(affinity[0], eq 0);
        })
        .unwrap();

    thread.set_affinity(&[0]).unwrap();
    barrier.wait();
    let affinity = thread.get_affinity().unwrap();
    barrier.wait();
    assert_that!(affinity, len 1);
    assert_that!(affinity[0], eq 0);
}

#[inventory_test]
pub fn thread_set_affinity_to_two_cores_from_thread_works() {
    test_requires!(POSIX_SUPPORT_CPU_AFFINITY);
    test_requires!(SystemInfo::NumberOfCpuCores.value() > 1);
    let _watchdog = Watchdog::new();
    let barrier = SpinBarrier::new();
    let barrier_clone = barrier.clone();
    let mut thread = ThreadBuilder::new()
        .spawn(move || {
            barrier_clone.wait();
            let handle = ThreadHandle::from_self();
            let affinity = handle.get_affinity().unwrap();
            barrier_clone.wait();
            assert_that!(affinity, len 2);
            assert_that!(affinity, contains 0);
            assert_that!(affinity, contains 1);
        })
        .unwrap();

    thread.set_affinity(&[0, 1]).unwrap();
    barrier.wait();
    let affinity = thread.get_affinity().unwrap();
    barrier.wait();

    assert_that!(affinity, len 2);
    assert_that!(affinity, contains 0);
    assert_that!(affinity, contains 1);
}

#[inventory_test]
pub fn thread_set_affinity_to_non_existing_cores_from_thread_fails() {
    test_requires!(POSIX_SUPPORT_CPU_AFFINITY);
    let _watchdog = Watchdog::new();
    let number_of_cpu_cores = SystemInfo::NumberOfCpuCores.value();
    let barrier = SpinBarrier::new();
    let barrier_clone = barrier.clone();
    let mut thread = ThreadBuilder::new()
        .spawn(move || {
            let handle = ThreadHandle::from_self();
            let original_affinity = handle.get_affinity().unwrap();
            barrier_clone.wait();
            barrier_clone.wait();
            let affinity = handle.get_affinity().unwrap();
            assert_that!(affinity, eq original_affinity);
        })
        .unwrap();

    let original_affinity = thread.get_affinity().unwrap();

    barrier.wait();
    let result = thread.set_affinity(&[number_of_cpu_cores + 1]);
    assert_that!(result, is_err);
    assert_that!(result.err(), eq Some(ThreadSetAffinityError::InvalidCpuCores));

    let affinity = thread.get_affinity().unwrap();
    barrier.wait();

    assert_that!(affinity, eq original_affinity);
}

#[inventory_test]
pub fn thread_destructor_does_not_block_on_empty_thread() {
    let _watchdog = Watchdog::new();
    let barrier = SpinBarrier::new();
    let barrier_clone = barrier.clone();
    let thread = ThreadBuilder::new()
        .spawn(move || {
            barrier_clone.wait();
            // nothing to see, move along
        })
        .unwrap();

    barrier.wait();
    let start = Time::now().unwrap();
    drop(thread);
    assert_that!(start.elapsed().unwrap(), lt Duration::from_millis(10));
}

#[inventory_test]
pub fn thread_destructor_does_block_on_busy_thread() {
    let _watchdog = Watchdog::new();
    const SLEEP_DURATION: Duration = Duration::from_millis(100);
    let barrier = SpinBarrier::new();
    let barrier_clone = barrier.clone();
    let thread = ThreadBuilder::new()
        .spawn(move || {
            barrier_clone.wait();
            nanosleep(SLEEP_DURATION).expect("failed to sleep");
        })
        .unwrap();

    barrier.wait();
    let start = Time::now().unwrap();
    drop(thread);
    assert_that!(start.elapsed().unwrap(), time_at_least SLEEP_DURATION);
}

#[inventory_test]
pub fn thread_scoped_threads_work() {
    let _watchdog = Watchdog::new();
    let number_of_threads = MAX_SCOPED_THREADS;
    let handle = BarrierHandle::new();
    let barrier = BarrierBuilder::new((number_of_threads + 1) as _)
        .create(&handle)
        .unwrap();
    let shared_counter = AtomicU64::new(0);

    thread_scope(|s| {
        for _ in 0..number_of_threads {
            s.thread_builder()
                .spawn(|| {
                    barrier.wait();
                    for _ in 0..1000 {
                        shared_counter.fetch_add(1, Ordering::Relaxed);
                    }
                })
                .unwrap();
        }

        barrier.wait();

        Ok(())
    })
    .unwrap();

    assert_that!(
        shared_counter.load(Ordering::Relaxed),
        eq(number_of_threads * 1000) as u64
    );
}
