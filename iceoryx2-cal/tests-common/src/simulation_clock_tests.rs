// Copyright (c) 2026 Contributors to the Eclipse Foundation
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

use iceoryx2_bb_concurrency::atomic::AtomicBool;
use iceoryx2_bb_posix::file::AccessMode;
use iceoryx2_bb_posix::testing::generate_file_path;
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing_macros::test;
use iceoryx2_cal::clock::simulation_clock::*;
use iceoryx2_cal::clock::*;
use iceoryx2_cal::dynamic_storage::DynamicStorage;
use iceoryx2_cal::dynamic_storage::DynamicStorageBuilder;
use iceoryx2_cal::dynamic_storage::posix_shared_memory::Storage;
use iceoryx2_cal::named_concept::*;
use iceoryx2_cal::testing::*;

struct TestFixture {
    name: FileName,
    config: TimeControllerConfig,
}

impl TestFixture {
    fn new() -> Self {
        let storage_config = generate_isolated_config::<
            <Storage<SimulationClockState> as NamedConceptMgmt>::Configuration,
        >();
        Self {
            name: generate_file_path().file_name(),
            config: TimeControllerConfig::default()
                .storage(storage_config)
                .capacity(10),
        }
    }

    fn name(&self) -> &FileName {
        &self.name
    }

    fn config(&self) -> &TimeControllerConfig {
        &self.config
    }
}

impl Drop for TestFixture {
    fn drop(&mut self) {
        let state_config = <Storage<AtomicBool> as NamedConceptMgmt>::Configuration::default()
            .prefix(self.config.get_prefix())
            .path_hint(self.config.get_path_hint());
        <Storage<AtomicBool> as DynamicStorage<AtomicBool>>::Builder::new(
            &TIME_CONTROLLER_STATE_NAME,
        )
        .config(&state_config)
        .open(AccessMode::ReadWrite)
        .unwrap()
        .acquire_ownership();
    }
}

fn create_time_controller_config() -> TimeControllerConfig {
    let storage_config = generate_isolated_config::<
        <Storage<SimulationClockState> as NamedConceptMgmt>::Configuration,
    >();
    TimeControllerConfig::default()
        .storage(storage_config)
        .capacity(10)
}

#[test]
fn time_controller_can_be_created_once() {
    let test = TestFixture::new();
    let sut1 =
        SimulationTimeControllerBuilder::new(test.name(), test.config()).create_time_controller();
    assert_that!(sut1, is_ok);

    let sut2 =
        SimulationTimeControllerBuilder::new(test.name(), test.config()).create_time_controller();
    assert_that!(sut2, is_err);
    assert_that!(sut2.err().unwrap(), eq TimeControllerBuilderCreateError::AlreadyCreatedByAnotherInstance);
}

#[test]
fn open_time_controller_fails_when_not_created_before() {
    let name = generate_file_path().file_name();
    let config = create_time_controller_config();
    let sut = SimulationTimeControllerBuilder::new(&name, &config).open_time_controller();
    assert_that!(sut, is_err);
    assert_that!(sut.err().unwrap(), eq TimeControllerBuilderOpenError::TimeControllerDoesNotExist);
}

#[test]
fn time_controller_can_be_opened_once() {
    let test = TestFixture::new();
    let time_controller = SimulationTimeControllerBuilder::new(test.name(), test.config())
        .create_time_controller()
        .unwrap();
    drop(time_controller);

    let sut1 =
        SimulationTimeControllerBuilder::new(test.name(), test.config()).open_time_controller();
    assert_that!(sut1, is_ok);

    let sut2 =
        SimulationTimeControllerBuilder::new(test.name(), test.config()).open_time_controller();
    assert_that!(sut2, is_err);
    assert_that!(sut2.err().unwrap(), eq TimeControllerBuilderOpenError::AlreadyOpenedByAnotherInstance);
}

#[test]
fn open_or_create_time_controller_works_when_not_created_before() {
    let test = TestFixture::new();
    let sut1 = SimulationTimeControllerBuilder::new(test.name(), test.config())
        .open_or_create_time_controller();
    assert_that!(sut1, is_ok);

    let sut2 = SimulationTimeControllerBuilder::new(test.name(), test.config())
        .open_or_create_time_controller();
    assert_that!(sut2, is_err);
    assert_that!(sut2.err().unwrap(), eq TimeControllerBuilderOpenOrCreateError::AlreadyOpenedByAnotherInstance);
}

#[test]
fn open_or_create_time_controller_works_when_time_controller_was_dropped() {
    let test = TestFixture::new();
    let time_controller = SimulationTimeControllerBuilder::new(test.name(), test.config())
        .create_time_controller()
        .unwrap();
    drop(time_controller);

    let sut = SimulationTimeControllerBuilder::new(test.name(), test.config())
        .open_or_create_time_controller();
    assert_that!(sut, is_ok);
}
