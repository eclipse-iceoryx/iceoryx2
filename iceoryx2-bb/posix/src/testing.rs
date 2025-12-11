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

use alloc::string::ToString;

use iceoryx2_bb_container::semantic_string::SemanticString;
use iceoryx2_bb_system_types::file_name::FileName;
use iceoryx2_bb_system_types::file_path::FilePath;
use iceoryx2_log::fatal_panic;

use crate::config::TEST_DIRECTORY;
use crate::directory::{Directory, DirectoryCreateError};
use crate::permission::Permission;
use crate::process_state::ProcessGuard;
use crate::unique_system_id::UniqueSystemId;

pub fn __internal_process_guard_staged_death(state: ProcessGuard) {
    state.staged_death();
}

pub fn create_test_directory() {
    match Directory::create(&TEST_DIRECTORY, Permission::OWNER_ALL) {
        Ok(_) | Err(DirectoryCreateError::DirectoryAlreadyExists) => (),
        Err(e) => fatal_panic!(
            "Failed to create test directory {} due to {:?}.",
            TEST_DIRECTORY,
            e
        ),
    };
}

pub fn generate_file_name() -> FilePath {
    create_test_directory();
    let mut file = FileName::new(b"test_").unwrap();
    file.push_bytes(
        UniqueSystemId::new()
            .unwrap()
            .value()
            .to_string()
            .as_bytes(),
    )
    .unwrap();

    FilePath::from_path_and_file(&TEST_DIRECTORY, &file).unwrap()
}
