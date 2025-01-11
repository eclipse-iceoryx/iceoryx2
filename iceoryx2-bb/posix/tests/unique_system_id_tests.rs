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

use core::time::Duration;
use std::{collections::HashSet, sync::Barrier};

use iceoryx2_bb_posix::{process::Process, system_configuration::SystemInfo, unique_system_id::*};
use iceoryx2_bb_testing::{assert_that, watchdog::Watchdog};

#[test]
fn unique_system_id_is_unique() {
    let sut1 = UniqueSystemId::new().unwrap();
    std::thread::sleep(Duration::from_secs(1));
    let sut2 = UniqueSystemId::new().unwrap();
    std::thread::sleep(Duration::from_secs(1));
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
fn unique_system_id_concurrently_created_ids_are_unique() {
    let _watchdog = Watchdog::new();
    const NUMBER_OF_ITERATIONS: usize = 1000;
    let number_of_threads = SystemInfo::NumberOfCpuCores.value() * 2;
    let barrier = Barrier::new(number_of_threads);

    std::thread::scope(|s| {
        let mut threads = vec![];
        for _ in 0..number_of_threads {
            threads.push(s.spawn(|| {
                let mut ids = Vec::new();
                ids.reserve(NUMBER_OF_ITERATIONS);

                barrier.wait();
                for _ in 0..NUMBER_OF_ITERATIONS {
                    ids.push(UniqueSystemId::new().unwrap().value());
                }

                ids
            }));
        }

        let mut id_set = HashSet::new();
        for t in threads {
            let ids = t.join().unwrap();
            assert_that!(ids, len NUMBER_OF_ITERATIONS);
            for id in ids {
                assert_that!(id_set.insert(id), eq true);
            }
        }
    });
}
