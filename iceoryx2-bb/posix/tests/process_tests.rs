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

use iceoryx2_bb_posix::process::*;
use iceoryx2_bb_testing::{assert_that, test_requires};
use iceoryx2_pal_posix::posix::{self, POSIX_SUPPORT_SCHEDULER};

#[test]
pub fn process_can_acquire_from_self() {
    let process = Process::from_self();
    assert_that!(process.id().value(), ne 0);

    let process2 = Process::from_pid(process.id());
    assert_that!(process.id().value(), eq process2.id().value());
}

#[test]
pub fn process_can_acquire_scheduler_information() {
    test_requires!(POSIX_SUPPORT_SCHEDULER);

    let process = Process::from_self();

    let process2 = Process::from_pid(process.id());

    assert_that!(process.get_priority(), eq process2.get_priority());
    assert_that!(process.get_scheduler(), eq process2.get_scheduler());
    assert_that!(process.get_priority(), is_ok);
    assert_that!(process.get_scheduler(), is_ok);
}

#[test]
pub fn process_is_alive_works() {
    let process = Process::from_self();
    assert_that!(process.is_alive(), eq true);

    let process2 = Process::from_pid(ProcessId::new(posix::pid_t::MAX - 1));
    assert_that!(process2.is_alive(), eq false);
}

#[test]
pub fn process_executable_path_works() {
    let process = Process::from_self();
    let executable_path = process.executable();

    assert_that!(executable_path, is_ok);
    let file_name = executable_path.as_ref().unwrap().file_name();
    let executable_file = core::str::from_utf8(&file_name).unwrap();
    println!("{executable_file}");
    assert_that!(executable_file.starts_with("process_tests"), eq true);
}
