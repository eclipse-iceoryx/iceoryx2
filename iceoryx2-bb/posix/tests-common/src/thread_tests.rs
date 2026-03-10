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

use iceoryx2_bb_concurrency::atomic::AtomicU64;
use iceoryx2_bb_concurrency::atomic::Ordering;
use iceoryx2_bb_posix::barrier::BarrierBuilder;
use iceoryx2_bb_posix::barrier::BarrierHandle;
use iceoryx2_bb_posix::barrier::Handle;
use iceoryx2_bb_posix::thread::thread_scope;
use iceoryx2_bb_posix::thread::MAX_SCOPED_THREADS;
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing::watchdog::Watchdog;
use iceoryx2_bb_testing_nostd_macros::requires_std;

#[cfg(feature = "std")]
pub use std_testing::*;

#[cfg(feature = "std")]
mod std_testing {
    pub use core::time::Duration;
    pub use std::sync::Barrier;
    pub use std::time::Instant;

    pub use alloc::sync::Arc;

    pub use iceoryx2_bb_posix::system_configuration::SystemInfo;
    pub use iceoryx2_bb_posix::thread::ThreadBuilder;
    pub use iceoryx2_bb_posix::thread::ThreadHandle;
    pub use iceoryx2_bb_posix::thread::ThreadName;
    pub use iceoryx2_bb_posix::thread::ThreadProperties;
    pub use iceoryx2_bb_posix::thread::ThreadSetAffinityError;
    pub use iceoryx2_bb_posix::thread::ThreadSpawnError;
    pub use iceoryx2_bb_testing::test_requires;
    pub use iceoryx2_pal_posix::posix::{self, POSIX_SUPPORT_CPU_AFFINITY};
}

#[requires_std("threading", "synchronization")]
pub fn thread_set_name_works() {
    use std::sync::Barrier;

    let barrier = Arc::new(Barrier::new(2));
    let name = ThreadName::try_from(b"oh-a-thread").unwrap();
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
    let name = thread.get_name().unwrap().clone();
    barrier.wait();
    drop(thread);

    assert_that!(name, eq b"oh-a-thread");
}

#[requires_std("threading", "synchronization")]
pub fn thread_creation_does_not_block() {
    use std::sync::Barrier;

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

#[requires_std("threading", "synchronization", "watchdog")]
pub fn thread_affinity_is_set_to_all_existing_cores_when_nothing_was_configured() {
    test_requires!(POSIX_SUPPORT_CPU_AFFINITY);
    let _watchdog = Watchdog::new();
    let number_of_cpu_cores = SystemInfo::NumberOfCpuCores.value();
    let init_barrier = Arc::new(Barrier::new(2));
    let test_barrier = Arc::new(Barrier::new(2));
    let thread = {
        let thread_init_barrier = init_barrier.clone();
        let thread_test_barrier = test_barrier.clone();
        ThreadBuilder::new()
            .spawn(move || {
                thread_init_barrier.wait();
                let handle = ThreadHandle::from_self();
                let affinity_result = handle.get_affinity();
                thread_test_barrier.wait();
                let affinity = affinity_result.unwrap();
                for core in 0..number_of_cpu_cores {
                    assert_that!(affinity, contains core);
                }
            })
            .unwrap()
    };

    init_barrier.wait();
    let affinity = thread.get_affinity().unwrap();
    test_barrier.wait();

    for core in 0..number_of_cpu_cores {
        assert_that!(affinity, contains core);
    }
}

#[requires_std("threading", "synchronization", "watchdog")]
pub fn thread_set_affinity_to_one_cpu_core_on_creation_works() {
    test_requires!(POSIX_SUPPORT_CPU_AFFINITY);
    let _watchdog = Watchdog::new();
    let barrier = Arc::new(Barrier::new(2));
    let thread = {
        let barrier = barrier.clone();
        ThreadBuilder::new()
            .affinity(&[0])
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

#[requires_std("threading", "synchronization", "watchdog")]
pub fn thread_set_affinity_to_two_cpu_cores_on_creation_works() {
    test_requires!(POSIX_SUPPORT_CPU_AFFINITY);
    test_requires!(SystemInfo::NumberOfCpuCores.value() > 1);
    let _watchdog = Watchdog::new();
    let barrier = Arc::new(Barrier::new(2));
    let thread = {
        let barrier = barrier.clone();
        ThreadBuilder::new()
            .affinity(&[0, 1])
            .spawn(move || {
                barrier.wait();
                let handle = ThreadHandle::from_self();
                let affinity = handle.get_affinity().unwrap();
                barrier.wait();
                assert_that!(affinity, len 2);
                assert_that!(affinity, contains 0);
                assert_that!(affinity, contains 1);
            })
            .unwrap()
    };

    barrier.wait();
    let affinity = thread.get_affinity().unwrap();
    barrier.wait();

    assert_that!(affinity, len 2);
    assert_that!(affinity, contains 0);
    assert_that!(affinity, contains 1);
}

#[requires_std("threading", "watchdog")]
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

#[requires_std("threading", "watchdog")]
pub fn thread_set_affinity_to_cores_greater_than_cpu_set_size_fails() {
    test_requires!(POSIX_SUPPORT_CPU_AFFINITY);
    let _watchdog = Watchdog::new();
    let thread = ThreadBuilder::new()
        .affinity(&[posix::CPU_SETSIZE])
        .spawn(|| {});

    assert_that!(thread, is_err);
    assert_that!(thread.err(), eq Some(ThreadSpawnError::CpuCoreOutsideOfSupportedCpuRangeForAffinity));
}

#[requires_std("threading", "synchronization", "watchdog")]
pub fn thread_set_affinity_to_one_core_from_handle_works() {
    test_requires!(POSIX_SUPPORT_CPU_AFFINITY);
    let _watchdog = Watchdog::new();
    let barrier = Arc::new(Barrier::new(2));
    let thread = {
        let barrier = barrier.clone();
        ThreadBuilder::new()
            .spawn(move || {
                let mut handle = ThreadHandle::from_self();
                handle.set_affinity(&[0]).unwrap();
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

#[requires_std("threading", "synchronization", "watchdog")]
pub fn thread_set_affinity_to_two_cores_from_handle_works() {
    test_requires!(POSIX_SUPPORT_CPU_AFFINITY);
    test_requires!(SystemInfo::NumberOfCpuCores.value() > 1);
    let _watchdog = Watchdog::new();
    let barrier = Arc::new(Barrier::new(2));
    let thread = {
        let barrier = barrier.clone();
        ThreadBuilder::new()
            .spawn(move || {
                let mut handle = ThreadHandle::from_self();
                handle.set_affinity(&[0, 1]).unwrap();
                barrier.wait();
                let affinity = handle.get_affinity().unwrap();
                barrier.wait();
                assert_that!(affinity, len 2);
                assert_that!(affinity, contains 0);
                assert_that!(affinity, contains 1);
            })
            .unwrap()
    };

    barrier.wait();
    let affinity = thread.get_affinity().unwrap();
    barrier.wait();

    assert_that!(affinity, len 2);
    assert_that!(affinity, contains 0);
    assert_that!(affinity, contains 1);
}

#[requires_std("threading", "synchronization", "watchdog")]
pub fn thread_set_affinity_to_non_existing_cores_from_handle_fails() {
    test_requires!(POSIX_SUPPORT_CPU_AFFINITY);
    let _watchdog = Watchdog::new();
    let number_of_cpu_cores = SystemInfo::NumberOfCpuCores.value();
    let barrier = Arc::new(Barrier::new(2));
    let thread = {
        let barrier = barrier.clone();
        ThreadBuilder::new()
            .spawn(move || {
                // thread is started
                barrier.wait();
                let mut handle = ThreadHandle::from_self();

                let original_affinity = handle.get_affinity().unwrap();
                barrier.wait();

                let result = handle.set_affinity(&[number_of_cpu_cores + 1]);
                assert_that!(result, is_err);
                assert_that!(result.err(), eq Some(ThreadSetAffinityError::InvalidCpuCores));

                barrier.wait();
                let affinity = handle.get_affinity().unwrap();
                barrier.wait();
                assert_that!(original_affinity, eq affinity);
            })
            .unwrap()
    };

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

#[requires_std("threading", "synchronization", "watchdog")]
pub fn thread_set_affinity_to_one_core_from_thread_works() {
    test_requires!(POSIX_SUPPORT_CPU_AFFINITY);
    let _watchdog = Watchdog::new();
    let barrier = Arc::new(Barrier::new(2));
    let mut thread = {
        let barrier = barrier.clone();
        let spawn_result = ThreadBuilder::new().spawn(move || {
            barrier.wait();
            let handle = ThreadHandle::from_self();
            let mut affinity = Vec::new();
            match handle.get_affinity() {
                Ok(value) => affinity = value,
                Err(error) => println!("Expected value but got error: {error:?}"),
            }
            barrier.wait();
            assert_that!(affinity, len 1);
            assert_that!(affinity[0], eq 0);
        });

        if let Err(error) = spawn_result {
            println!("Expected value but got error: {error:?}");
            assert!(false);
        }

        spawn_result.unwrap()
    };

    thread.set_affinity(&[0]).unwrap();
    barrier.wait();
    let affinity = thread.get_affinity().unwrap();
    barrier.wait();
    assert_that!(affinity, len 1);
    assert_that!(affinity[0], eq 0);
}

#[requires_std("threading", "synchronization", "watchdog")]
pub fn thread_set_affinity_to_two_cores_from_thread_works() {
    test_requires!(POSIX_SUPPORT_CPU_AFFINITY);
    test_requires!(SystemInfo::NumberOfCpuCores.value() > 1);
    let _watchdog = Watchdog::new();
    let barrier = Arc::new(Barrier::new(2));
    let mut thread = {
        let barrier = barrier.clone();
        ThreadBuilder::new()
            .spawn(move || {
                barrier.wait();
                let handle = ThreadHandle::from_self();
                let affinity = handle.get_affinity().unwrap();
                barrier.wait();
                assert_that!(affinity, len 2);
                assert_that!(affinity, contains 0);
                assert_that!(affinity, contains 1);
            })
            .unwrap()
    };

    thread.set_affinity(&[0, 1]).unwrap();
    barrier.wait();
    let affinity = thread.get_affinity().unwrap();
    barrier.wait();
    assert_that!(affinity, len 2);
    assert_that!(affinity, contains 0);
    assert_that!(affinity, contains 1);
}

#[requires_std("threading", "synchronization", "watchdog")]
pub fn thread_set_affinity_to_non_existing_cores_from_thread_fails() {
    test_requires!(POSIX_SUPPORT_CPU_AFFINITY);
    let _watchdog = Watchdog::new();
    let number_of_cpu_cores = SystemInfo::NumberOfCpuCores.value();
    let barrier = Arc::new(Barrier::new(2));
    let mut thread = {
        let barrier = barrier.clone();
        ThreadBuilder::new()
            .spawn(move || {
                let handle = ThreadHandle::from_self();
                let original_affinity = handle.get_affinity().unwrap();
                barrier.wait();
                barrier.wait();
                let affinity = handle.get_affinity().unwrap();
                assert_that!(affinity, eq original_affinity);
            })
            .unwrap()
    };

    let original_affinity = thread.get_affinity().unwrap();

    barrier.wait();
    let result = thread.set_affinity(&[number_of_cpu_cores + 1]);
    assert_that!(result, is_err);
    assert_that!(result.err(), eq Some(ThreadSetAffinityError::InvalidCpuCores));

    let affinity = thread.get_affinity().unwrap();
    barrier.wait();

    assert_that!(affinity, eq original_affinity);
}

#[requires_std("threading", "synchronization", "watchdog", "time")]
pub fn thread_destructor_does_not_block_on_empty_thread() {
    let _watchdog = Watchdog::new();
    let barrier = Arc::new(Barrier::new(2));
    let thread = {
        let barrier = barrier.clone();
        ThreadBuilder::new()
            .spawn(move || {
                barrier.wait();
                // nothing to see, move along
            })
            .unwrap()
    };

    barrier.wait();
    let start = Instant::now();
    drop(thread);
    assert_that!(start.elapsed(), lt(Duration::from_millis(10)));
}

#[requires_std("threading", "synchronization", "watchdog", "time")]
pub fn thread_destructor_does_block_on_busy_thread() {
    let _watchdog = Watchdog::new();
    const SLEEP_DURATION: Duration = Duration::from_millis(100);
    let barrier = Arc::new(Barrier::new(2));
    let thread = {
        let barrier = barrier.clone();
        let spawn_result = ThreadBuilder::new().spawn(move || {
            barrier.wait();
            let start = Instant::now();
            while start.elapsed() < SLEEP_DURATION {
                std::thread::sleep(SLEEP_DURATION - start.elapsed());
            }
        });

        if let Err(error) = spawn_result {
            println!("Expected value but got error: {error:?}");
            assert!(false);
        }

        spawn_result.unwrap()
    };

    barrier.wait();
    let start = Instant::now();
    drop(thread);
    assert_that!(start.elapsed(), time_at_least SLEEP_DURATION);
}

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
