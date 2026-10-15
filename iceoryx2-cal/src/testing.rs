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

use crate::{
    clock::{Clock, ClockBuilder},
    named_concept::*,
};
use alloc::sync::Arc;
use iceoryx2_bb_posix::{
    config::TEST_DIRECTORY,
    directory::{Directory, DirectoryCreateError},
    file::Permission,
    testing::generate_file_path,
};
use iceoryx2_log::fatal_panic;

pub type StubClock = Arc<crate::clock::posix::PosixClock<crate::event::InternalStubEvent>>;

pub fn generate_isolated_config<T: NamedConceptConfiguration>() -> T {
    match Directory::create(&TEST_DIRECTORY, Permission::OWNER_ALL) {
        Ok(_) | Err(DirectoryCreateError::DirectoryAlreadyExists) => (),
        Err(e) => fatal_panic!(
            "Failed to create test directory {} due to {:?}.",
            TEST_DIRECTORY,
            e
        ),
    };

    T::default()
        .prefix(&generate_file_path().file_name())
        .path_hint(&TEST_DIRECTORY)
}

pub fn generate_custom_config<T: NamedConceptMgmt>(
    name: &FileName,
    path: &Path,
) -> T::Configuration {
    T::Configuration::default().prefix(name).path_hint(path)
}

#[derive(Debug)]
pub(crate) struct FailedToOpenStubClock;

pub(crate) fn create_stub_clock<C: Clock<crate::event::InternalStubEvent>>()
-> Result<C, FailedToOpenStubClock> {
    let clock_config = generate_isolated_config::<C::Configuration>();
    let name = generate_file_path().file_name();

    C::Builder::new(&name)
        .config(&clock_config)
        .open_clock()
        .map_err(|_| FailedToOpenStubClock)
}

pub fn create_test_clock() -> StubClock {
    Arc::new(create_stub_clock().unwrap())
}
