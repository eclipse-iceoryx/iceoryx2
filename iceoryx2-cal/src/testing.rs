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

use crate::named_concept::*;
use iceoryx2_bb_posix::{
    config::TEST_DIRECTORY,
    directory::{Directory, DirectoryCreateError},
    file::Permission,
    testing::generate_file_path,
};
use iceoryx2_log::fatal_panic;

pub fn generate_isolated_config<T: NamedConceptMgmt>() -> T::Configuration {
    match Directory::create(&TEST_DIRECTORY, Permission::OWNER_ALL) {
        Ok(_) | Err(DirectoryCreateError::DirectoryAlreadyExists) => (),
        Err(e) => fatal_panic!(
            "Failed to create test directory {} due to {:?}.",
            TEST_DIRECTORY,
            e
        ),
    };

    T::Configuration::default()
        .prefix(&generate_file_path().file_name())
        .path_hint(&TEST_DIRECTORY)
}
