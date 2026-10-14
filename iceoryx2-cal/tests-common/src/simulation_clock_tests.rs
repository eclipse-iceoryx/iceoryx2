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

use core::time::Duration;

use iceoryx2_bb_concurrency::atomic::AtomicBool;
use iceoryx2_bb_posix::file::AccessMode;
use iceoryx2_bb_posix::testing::generate_file_path;
use iceoryx2_bb_si_units::TimePoint;
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing_macros::test;
use iceoryx2_cal::clock::simulation_clock::*;
use iceoryx2_cal::clock::*;
use iceoryx2_cal::dynamic_storage::DynamicStorage;
use iceoryx2_cal::dynamic_storage::DynamicStorageBuilder;
use iceoryx2_cal::dynamic_storage::posix_shared_memory::Storage;
use iceoryx2_cal::event::UnixDatagramShmCountingBitSet;
use iceoryx2_cal::named_concept::*;
use iceoryx2_cal::testing::*;

struct TestFixture {
    name: FileName,
    config: TimeControllerConfig,
    storage_config: <Storage<SimulationClockState> as NamedConceptMgmt>::Configuration,
}

impl TestFixture {
    fn new() -> Self {
        let storage_config = generate_isolated_config::<
            <Storage<SimulationClockState> as NamedConceptMgmt>::Configuration,
        >();
        Self {
            name: generate_file_path().file_name(),
            config: TimeControllerConfig::default()
                .storage_config(storage_config.clone())
                .capacity(10),
            storage_config,
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
        // clean up time controller state
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

        // clean up clock state storage
        <Storage<SimulationClockState> as DynamicStorage<SimulationClockState>>::Builder::new(
            self.name(),
        )
        .config(&self.storage_config)
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
        .storage_config(storage_config)
        .capacity(10)
}

fn cleanup_clock_state(name: &FileName, config: &TimeControllerConfig) {
    <Storage<SimulationClockState> as DynamicStorage<SimulationClockState>>::Builder::new(name)
        .config(config.get_storage_config())
        .open(AccessMode::ReadWrite)
        .unwrap()
        .acquire_ownership();
}

fn cleanup_time_controller_state(config: &TimeControllerConfig) {
    let state_config = <Storage<AtomicBool> as NamedConceptMgmt>::Configuration::default()
        .prefix(config.get_prefix())
        .path_hint(config.get_path_hint());
    <Storage<AtomicBool> as DynamicStorage<AtomicBool>>::Builder::new(&TIME_CONTROLLER_STATE_NAME)
        .config(&state_config)
        .open(AccessMode::ReadWrite)
        .unwrap()
        .acquire_ownership();
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

#[test]
fn advance_returns_old_timepoint() {
    let test = TestFixture::new();
    let duration = Duration::from_nanos(14873);

    let time_controller = SimulationTimeControllerBuilder::new(test.name(), test.config())
        .create_time_controller()
        .unwrap();

    let tp0 = time_controller.advance(duration);
    assert_that!(tp0, eq TimePoint::new());
    let tp1 = time_controller.advance(2 * duration);
    assert_that!(tp1, eq TimePoint::from_nanos(duration.as_nanos()));
    let tp2 = time_controller.advance(duration);
    assert_that!(tp2, eq TimePoint::from_nanos(3 * duration.as_nanos()));
}

#[test]
fn open_clock_succeeds_when_time_controller_was_created_before() {
    let test = TestFixture::new();
    let _time_controller = SimulationTimeControllerBuilder::new(test.name(), test.config())
        .create_time_controller()
        .unwrap();

    assert_that!(
        SimulationClockBuilder::<UnixDatagramShmCountingBitSet>::new(test.name())
            .config(test.config())
            .open_clock(),
        is_ok
    );
}

#[test]
fn open_clock_succeeds_without_time_controller() {
    let name = generate_file_path().file_name();
    let config = create_time_controller_config();

    assert_that!(
        SimulationClockBuilder::<UnixDatagramShmCountingBitSet>::new(&name)
            .config(&config)
            .open_clock(),
        is_ok
    );

    cleanup_clock_state(&name, &config);
}

#[test]
fn open_two_clocks_works() {
    let name = generate_file_path().file_name();
    let config = create_time_controller_config();

    let _clock = SimulationClockBuilder::<UnixDatagramShmCountingBitSet>::new(&name)
        .config(&config)
        .open_clock()
        .unwrap();

    assert_that!(
        SimulationClockBuilder::<UnixDatagramShmCountingBitSet>::new(&name)
            .config(&config)
            .open_clock(),
        is_ok
    );

    cleanup_clock_state(&name, &config);
}

#[test]
fn time_controller_can_be_created_after_clock() {
    let test = TestFixture::new();

    let _clock = SimulationClockBuilder::<UnixDatagramShmCountingBitSet>::new(test.name())
        .config(test.config())
        .open_clock()
        .unwrap();

    assert_that!(
        SimulationTimeControllerBuilder::new(test.name(), test.config()).create_time_controller(),
        is_ok
    );
    assert_that!(
        SimulationTimeControllerBuilder::new(test.name(), test.config())
            .open_or_create_time_controller(),
        is_ok
    );
    assert_that!(
        SimulationTimeControllerBuilder::new(test.name(), test.config()).open_time_controller(),
        is_ok
    );
}

#[test]
fn now_returns_current_time() {
    let test = TestFixture::new();
    let duration = Duration::from_nanos(452381790);

    let time_controller = SimulationTimeControllerBuilder::new(test.name(), test.config())
        .create_time_controller()
        .unwrap();
    let clock = SimulationClockBuilder::<UnixDatagramShmCountingBitSet>::new(test.name())
        .config(test.config())
        .open_clock()
        .unwrap();

    assert_that!(clock.now().unwrap(), eq TimePoint::new());

    time_controller.advance(duration);
    assert_that!(clock.now().unwrap(), eq TimePoint::from_nanos(duration.as_nanos()));

    time_controller.advance(2 * duration);
    assert_that!(clock.now().unwrap(), eq TimePoint::from_nanos(3 * duration.as_nanos()));
}

#[test]
fn now_returns_current_time_with_configured_creation_time() {
    let name = generate_file_path().file_name();
    let tp = TimePoint::from_nanos(6253);
    let config = create_time_controller_config().creation_time(tp);
    let duration = Duration::from_nanos(452381790);

    let time_controller = SimulationTimeControllerBuilder::new(&name, &config)
        .create_time_controller()
        .unwrap();
    let clock = SimulationClockBuilder::<UnixDatagramShmCountingBitSet>::new(&name)
        .config(&config)
        .open_clock()
        .unwrap();

    assert_that!(clock.now().unwrap(), eq tp);

    time_controller.advance(duration);
    assert_that!(clock.now().unwrap(), eq tp + duration);

    time_controller.advance(2 * duration);
    assert_that!(clock.now().unwrap(), eq tp + (3 * duration));

    cleanup_clock_state(&name, &config);
    cleanup_time_controller_state(&config);
}
