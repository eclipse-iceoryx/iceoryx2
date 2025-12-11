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
use iceoryx2_bb_container::semantic_string::*;
use iceoryx2_bb_elementary::math::ToB64;
use iceoryx2_bb_posix::{
    config::TEST_DIRECTORY,
    directory::{Directory, DirectoryCreateError},
    file::Permission,
    unique_system_id::UniqueSystemId,
};
use iceoryx2_bb_system_types::file_name::FileName;
use iceoryx2_log::fatal_panic;

pub fn generate_name() -> FileName {
    let mut file = FileName::new(b"test_").unwrap();
    file.push_bytes(
        UniqueSystemId::new()
            .unwrap()
            .value()
            .to_b64()
            .to_lowercase()
            .as_bytes(),
    )
    .unwrap();
    file
}

fn generate_prefix() -> FileName {
    let mut prefix = FileName::new(b"test_prefix_").unwrap();
    prefix
        .push_bytes(
            UniqueSystemId::new()
                .unwrap()
                .value()
                .to_b64()
                .to_lowercase()
                .as_bytes(),
        )
        .unwrap();

    prefix
}

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
        .prefix(&generate_prefix())
        .path_hint(&TEST_DIRECTORY)
}
