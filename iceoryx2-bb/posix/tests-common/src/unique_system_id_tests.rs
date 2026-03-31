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

use alloc::collections::BTreeSet;
use alloc::vec::Vec;
use core::time::Duration;

use iceoryx2_bb_posix::barrier::*;
use iceoryx2_bb_posix::clock::nanosleep;
use iceoryx2_bb_posix::mutex::{Handle, MutexBuilder, MutexHandle};
use iceoryx2_bb_posix::process::Process;
use iceoryx2_bb_posix::system_configuration::SystemInfo;
use iceoryx2_bb_posix::thread::thread_scope;
use iceoryx2_bb_posix::unique_system_id::*;
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing::watchdog::Watchdog;
use iceoryx2_bb_testing_macros::test;

#[test]
pub fn is_unique() {
    let sut1 = UniqueSystemId::new().unwrap();
    nanosleep(Duration::from_secs(1)).unwrap();
    let sut2 = UniqueSystemId::new().unwrap();
    nanosleep(Duration::from_secs(1)).unwrap();
    let sut3 = UniqueSystemId::new().unwrap();

    assert_that!(sut1.value(), ne sut2.value());

    let pid = Process::from_self().id();

    assert_that!(sut1.pid(), eq pid);
    assert_that!(sut2.pid(), eq pid);

    assert_that!(sut2.creation_time().seconds(), gt sut1.creation_time().seconds());
    assert_that!(sut3.creation_time().seconds(), gt sut2.creation_time().seconds());
    assert_that!(sut1.creation_time().seconds() + 2, ge sut2.creation_time().seconds());
    assert_that!(sut1.creation_time().seconds() + 3, ge sut3.creation_time().seconds());
}

#[test]
pub fn concurrently_created_ids_are_unique() {
    let _watchdog = Watchdog::new();

    const NUMBER_OF_ITERATIONS: usize = 1000;
    let number_of_threads = SystemInfo::NumberOfCpuCores.value() * 2;

    let barrier_handle = BarrierHandle::new();
    let barrier = BarrierBuilder::new(number_of_threads as u32)
        .create(&barrier_handle)
        .unwrap();

    let ids_per_thread_handle = MutexHandle::<Vec<Vec<u128>>>::new();
    let ids_per_thread = MutexBuilder::new()
        .create(
            Vec::with_capacity(number_of_threads),
            &ids_per_thread_handle,
        )
        .expect("failed to create mutex");

    thread_scope(|s| {
        for _ in 0..number_of_threads {
            s.thread_builder()
                .spawn(|| {
                    let mut ids = Vec::with_capacity(NUMBER_OF_ITERATIONS);

                    barrier.wait();
                    for _ in 0..NUMBER_OF_ITERATIONS {
                        ids.push(UniqueSystemId::new().unwrap().value());
                    }

                    ids_per_thread
                        .lock()
                        .expect("failed to lock mutex")
                        .push(ids);
                })
                .expect("failed to spawn thread");
        }

        Ok(())
    })
    .expect("failed to spawn thread");

    let mut all_ids = BTreeSet::new();
    for collected_ids in ids_per_thread
        .lock()
        .expect("failed to lock mutex")
        .iter()
        .take(number_of_threads)
    {
        assert_that!(collected_ids, len NUMBER_OF_ITERATIONS);
        for id in collected_ids.iter() {
            assert_that!(all_ids.insert(*id), eq true);
        }
    }
}
