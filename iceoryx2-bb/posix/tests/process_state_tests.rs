// Copyright (c) 2024 Contributors to the Eclipse Foundation
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

use iceoryx2_bb_container::semantic_string::SemanticString;
use iceoryx2_bb_posix::config::*;
use iceoryx2_bb_posix::file::{File, FileBuilder};
use iceoryx2_bb_posix::unix_datagram_socket::CreationMode;
use iceoryx2_bb_posix::{process_state::*, unique_system_id::UniqueSystemId};
use iceoryx2_bb_system_types::{file_name::FileName, file_path::FilePath};
use iceoryx2_bb_testing::assert_that;

fn generate_file_path() -> FilePath {
    let mut file = FileName::new(b"process_state_tests").unwrap();
    file.push_bytes(
        UniqueSystemId::new()
            .unwrap()
            .value()
            .to_string()
            .as_bytes(),
    )
    .unwrap();

    FilePath::from_path_and_file(&test_directory(), &file).unwrap()
}

#[test]
pub fn process_state_guard_can_be_created() {
    let path = generate_file_path();

    let guard = ProcessGuard::new(&path).unwrap();

    assert_that!(*guard.path(), eq path);
    assert_that!(File::does_exist(&path).unwrap(), eq true);
}

#[test]
pub fn process_state_guard_removes_file_when_dropped() {
    let path = generate_file_path();

    let guard = ProcessGuard::new(&path).unwrap();
    assert_that!(File::does_exist(&path).unwrap(), eq true);
    drop(guard);
    assert_that!(File::does_exist(&path).unwrap(), eq false);
}

#[test]
pub fn process_state_guard_cannot_use_already_existing_file() {
    let path = generate_file_path();

    FileBuilder::new(&path)
        .creation_mode(CreationMode::PurgeAndCreate)
        .create()
        .unwrap();

    let guard = ProcessGuard::new(&path);
    assert_that!(guard.is_err(), eq true);
    assert_that!(guard.err().unwrap(), eq ProcessGuardCreateError::AlreadyExists);
}
