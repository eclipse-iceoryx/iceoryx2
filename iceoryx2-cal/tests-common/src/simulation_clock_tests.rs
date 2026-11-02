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

use alloc::sync::Arc;
use core::time::Duration;

use iceoryx2_bb_concurrency::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use iceoryx2_bb_posix::barrier::{BarrierBuilder, BarrierHandle};
use iceoryx2_bb_posix::clock::nanosleep;
use iceoryx2_bb_posix::file::AccessMode;
use iceoryx2_bb_posix::ipc_capable::Handle;
use iceoryx2_bb_posix::mutex::{Mutex, MutexBuilder, MutexHandle};
use iceoryx2_bb_posix::system_configuration::SystemInfo;
use iceoryx2_bb_posix::testing::generate_file_path;
use iceoryx2_bb_posix::thread::thread_scope;
use iceoryx2_bb_si_units::TimePoint;
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing::watchdog::Watchdog;
use iceoryx2_bb_testing_macros::test;
use iceoryx2_cal::clock::simulation_clock::*;
use iceoryx2_cal::clock::*;
use iceoryx2_cal::dynamic_storage::posix_shared_memory::Storage;
use iceoryx2_cal::dynamic_storage::{DynamicStorage, DynamicStorageBuilder};
use iceoryx2_cal::event::{
    Event, ListenerBuilder, ListenerWaitInterface, UnixDatagramShmCountingBitSet,
};
use iceoryx2_cal::named_concept::*;
use iceoryx2_cal::reactor::{Reactor, ReactorBuilder};
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
                .storage_config(storage_config.clone())
                .capacity(100),
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
        // clean up clock state storage
        cleanup_clock_state(self.name(), self.config());
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
    if let Ok(s) =
        <Storage<SimulationClockState> as DynamicStorage<SimulationClockState>>::Builder::new(name)
            .config(config.get_storage_config())
            .open(AccessMode::ReadWrite)
    {
        s.acquire_ownership();
    }
}

#[test]
fn time_controller_can_be_created_once() {
    let test = TestFixture::new();
    let sut1 = SimulationClockBuilder::<UnixDatagramShmCountingBitSet>::new(test.name())
        .config(test.config())
        .create_time_controller();
    assert_that!(sut1, is_ok);

    let sut2 = SimulationClockBuilder::<UnixDatagramShmCountingBitSet>::new(test.name())
        .config(test.config())
        .create_time_controller();
    assert_that!(sut2, is_err);
    assert_that!(sut2.err().unwrap(), eq TimeControllerBuilderCreateError::AlreadyCreatedByAnotherInstance);
}

#[test]
fn open_time_controller_fails_when_not_created_before() {
    let name = generate_file_path().file_name();
    let config = create_time_controller_config();
    let sut = SimulationClockBuilder::<UnixDatagramShmCountingBitSet>::new(&name)
        .config(&config)
        .open_time_controller();
    assert_that!(sut, is_err);
    assert_that!(sut.err().unwrap(), eq TimeControllerBuilderOpenError::TimeControllerDoesNotExist);
}

#[test]
fn time_controller_can_be_opened_once() {
    let test = TestFixture::new();
    let time_controller = SimulationClockBuilder::<UnixDatagramShmCountingBitSet>::new(test.name())
        .config(test.config())
        .create_time_controller()
        .unwrap();

    // add additional clock that opens the clock internals so that OSes like windows, that remove
    // all shm's when no process is using them, also pass the test
    let _clock = SimulationClockBuilder::<UnixDatagramShmCountingBitSet>::new(test.name())
        .config(test.config())
        .open_clock()
        .unwrap();

    drop(time_controller);

    let sut1 = SimulationClockBuilder::<UnixDatagramShmCountingBitSet>::new(test.name())
        .config(test.config())
        .open_time_controller();
    assert_that!(sut1, is_ok);

    let sut2 = SimulationClockBuilder::<UnixDatagramShmCountingBitSet>::new(test.name())
        .config(test.config())
        .open_time_controller();
    assert_that!(sut2, is_err);
    assert_that!(sut2.err().unwrap(), eq TimeControllerBuilderOpenError::AlreadyOpenedByAnotherInstance);
}

#[test]
fn open_or_create_time_controller_works_when_not_created_before() {
    let test = TestFixture::new();
    let sut1 = SimulationClockBuilder::<UnixDatagramShmCountingBitSet>::new(test.name())
        .config(test.config())
        .open_or_create_time_controller();
    assert_that!(sut1, is_ok);

    let sut2 = SimulationClockBuilder::<UnixDatagramShmCountingBitSet>::new(test.name())
        .config(test.config())
        .open_or_create_time_controller();
    assert_that!(sut2, is_err);
    assert_that!(sut2.err().unwrap(), eq TimeControllerBuilderOpenOrCreateError::AlreadyOpenedByAnotherInstance);
}

#[test]
fn open_or_create_time_controller_works_when_time_controller_was_dropped() {
    let test = TestFixture::new();
    let time_controller = SimulationClockBuilder::<UnixDatagramShmCountingBitSet>::new(test.name())
        .config(test.config())
        .create_time_controller()
        .unwrap();
    drop(time_controller);

    let sut = SimulationClockBuilder::<UnixDatagramShmCountingBitSet>::new(test.name())
        .config(test.config())
        .open_or_create_time_controller();
    assert_that!(sut, is_ok);
}

#[test]
fn advance_returns_old_timepoint() {
    let test = TestFixture::new();
    let duration = Duration::from_nanos(14873);

    let time_controller = SimulationClockBuilder::<UnixDatagramShmCountingBitSet>::new(test.name())
        .config(test.config())
        .create_time_controller()
        .unwrap();

    let tp0 = time_controller.advance(duration).unwrap();
    assert_that!(tp0, eq TimePoint::new());
    let tp1 = time_controller.advance(2 * duration).unwrap();
    assert_that!(tp1, eq TimePoint::from_nanos(duration.as_nanos()));
    let tp2 = time_controller.advance(duration).unwrap();
    assert_that!(tp2, eq TimePoint::from_nanos(3 * duration.as_nanos()));
}

#[test]
fn open_clock_succeeds_when_time_controller_was_created_before() {
    let test = TestFixture::new();
    let _time_controller =
        SimulationClockBuilder::<UnixDatagramShmCountingBitSet>::new(test.name())
            .config(test.config())
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
    let test = TestFixture::new();

    assert_that!(
        SimulationClockBuilder::<UnixDatagramShmCountingBitSet>::new(test.name())
            .config(test.config())
            .open_clock(),
        is_ok
    );
}

#[test]
fn open_two_clocks_works() {
    let test = TestFixture::new();

    let _clock = SimulationClockBuilder::<UnixDatagramShmCountingBitSet>::new(test.name())
        .config(test.config())
        .open_clock()
        .unwrap();

    assert_that!(
        SimulationClockBuilder::<UnixDatagramShmCountingBitSet>::new(test.name())
            .config(test.config())
            .open_clock(),
        is_ok
    );
}

#[test]
fn time_controller_can_be_created_after_clock() {
    let test = TestFixture::new();

    let _clock = SimulationClockBuilder::<UnixDatagramShmCountingBitSet>::new(test.name())
        .config(test.config())
        .open_clock()
        .unwrap();

    assert_that!(
        SimulationClockBuilder::<UnixDatagramShmCountingBitSet>::new(test.name())
            .config(test.config())
            .create_time_controller(),
        is_ok
    );
    assert_that!(
        SimulationClockBuilder::<UnixDatagramShmCountingBitSet>::new(test.name())
            .config(test.config())
            .open_or_create_time_controller(),
        is_ok
    );
    assert_that!(
        SimulationClockBuilder::<UnixDatagramShmCountingBitSet>::new(test.name())
            .config(test.config())
            .open_time_controller(),
        is_ok
    );
}

#[test]
fn now_returns_current_time() {
    let test = TestFixture::new();
    let duration = Duration::from_nanos(452381790);

    let time_controller = SimulationClockBuilder::<UnixDatagramShmCountingBitSet>::new(test.name())
        .config(test.config())
        .create_time_controller()
        .unwrap();
    let clock = SimulationClockBuilder::<UnixDatagramShmCountingBitSet>::new(test.name())
        .config(test.config())
        .open_clock()
        .unwrap();

    assert_that!(clock.now().unwrap(), eq TimePoint::new());

    time_controller
        .advance(duration)
        .expect("advance time worked");
    assert_that!(clock.now().unwrap(), eq TimePoint::from_nanos(duration.as_nanos()));

    time_controller
        .advance(2 * duration)
        .expect("advance time worked");
    assert_that!(clock.now().unwrap(), eq TimePoint::from_nanos(3 * duration.as_nanos()));
}

#[test]
fn now_returns_current_time_with_configured_creation_time() {
    let name = generate_file_path().file_name();
    let tp = TimePoint::from_nanos(6253);
    let config = create_time_controller_config().creation_time(tp);
    let duration = Duration::from_nanos(452381790);

    let time_controller = SimulationClockBuilder::<UnixDatagramShmCountingBitSet>::new(&name)
        .config(&config)
        .create_time_controller()
        .unwrap();
    let clock = SimulationClockBuilder::<UnixDatagramShmCountingBitSet>::new(&name)
        .config(&config)
        .open_clock()
        .unwrap();

    assert_that!(clock.now().unwrap(), eq tp);

    time_controller
        .advance(duration)
        .expect("advance time worked");
    assert_that!(clock.now().unwrap(), eq tp + duration);

    time_controller
        .advance(2 * duration)
        .expect("advance time worked");
    assert_that!(clock.now().unwrap(), eq tp + (3 * duration));

    cleanup_clock_state(&name, &config);
}

#[test]
fn schedule_on_notifies_listener_when_timeout_has_already_expired() {
    let test = TestFixture::new();
    let duration = Duration::from_nanos(100);

    let time_controller = SimulationClockBuilder::<UnixDatagramShmCountingBitSet>::new(test.name())
        .config(test.config())
        .create_time_controller()
        .unwrap();
    let clock = Arc::new(
        SimulationClockBuilder::<UnixDatagramShmCountingBitSet>::new(test.name())
            .config(test.config())
            .open_clock()
            .unwrap(),
    );

    let listener_name = unsafe { FileName::new_unchecked_const(b"foo") };
    let listener = <UnixDatagramShmCountingBitSet as Event>::ListenerBuilder::new(&listener_name)
        .create(clock.clone())
        .unwrap();
    let timeout = Timeout::At(TimePoint::from_nanos(40));

    let _ = time_controller.advance(duration);

    let handle = clock.schedule_on(timeout, &listener_name);
    assert_that!(handle, is_ok);
    let t = match handle.unwrap() {
        ScheduleOnHandle::TranslatedTimeout(t) => t,
        ScheduleOnHandle::Guard(_) => {
            assert_that!(shall_never_be_reached);
        }
    };
    assert_that!(t, eq Timeout::By(Duration::ZERO));
    assert_that!(listener.try_wait(&mut |_| {}).unwrap(), eq 1);
}

#[test]
fn schedule_on_returns_guard_when_timeout_has_not_expired() {
    let test = TestFixture::new();

    let clock = Arc::new(
        SimulationClockBuilder::<UnixDatagramShmCountingBitSet>::new(test.name())
            .config(test.config())
            .open_clock()
            .unwrap(),
    );

    let listener_name = unsafe { FileName::new_unchecked_const(b"foo") };
    let timeout = Timeout::At(TimePoint::from_nanos(100));

    let handle = clock.schedule_on(timeout, &listener_name);
    assert_that!(handle, is_ok);
    if let ScheduleOnHandle::TranslatedTimeout(_) = handle.unwrap() {
        assert_that!(shall_never_be_reached);
    }
}

#[test]
fn advance_notifies_when_timeout_was_scheduled_with_schedule_on() {
    let test = TestFixture::new();
    let duration = Duration::from_nanos(100);

    let time_controller = SimulationClockBuilder::<UnixDatagramShmCountingBitSet>::new(test.name())
        .config(test.config())
        .create_time_controller()
        .unwrap();
    let clock = Arc::new(
        SimulationClockBuilder::<UnixDatagramShmCountingBitSet>::new(test.name())
            .config(test.config())
            .open_clock()
            .unwrap(),
    );

    let listener_name = unsafe { FileName::new_unchecked_const(b"foo") };
    let listener = <UnixDatagramShmCountingBitSet as Event>::ListenerBuilder::new(&listener_name)
        .create(clock.clone())
        .unwrap();
    let timeout = Timeout::By(duration);
    let _handle = clock.schedule_on(timeout, &listener_name).unwrap();
    assert_that!(listener.try_wait(&mut |_| {}).unwrap(), eq 0);

    let _ = time_controller.advance(duration / 2);
    assert_that!(listener.try_wait(&mut |_| {}).unwrap(), eq 0);

    let _ = time_controller.advance(duration);
    assert_that!(listener.try_wait(&mut |_| {}).unwrap(), eq 1);
}

#[test]
fn guard_deregisters_timeout() {
    let test = TestFixture::new();
    let duration = Duration::from_nanos(100);

    let time_controller = SimulationClockBuilder::<UnixDatagramShmCountingBitSet>::new(test.name())
        .config(test.config())
        .create_time_controller()
        .unwrap();
    let clock = Arc::new(
        SimulationClockBuilder::<UnixDatagramShmCountingBitSet>::new(test.name())
            .config(test.config())
            .open_clock()
            .unwrap(),
    );

    let listener_name = unsafe { FileName::new_unchecked_const(b"foo") };
    let listener = <UnixDatagramShmCountingBitSet as Event>::ListenerBuilder::new(&listener_name)
        .create(clock.clone())
        .unwrap();
    let timeout = Timeout::By(duration);

    let handle = clock.schedule_on(timeout, &listener_name).unwrap();
    let guard = match handle {
        ScheduleOnHandle::TranslatedTimeout(_) => {
            assert_that!(shall_never_be_reached);
        }
        ScheduleOnHandle::Guard(g) => g,
    };

    drop(guard);

    let _ = time_controller.advance(2 * duration);
    assert_that!(listener.try_wait(&mut |_| {}).unwrap(), eq 0);
}

#[test]
fn schedule_returns_zero_timeout_when_passed_timeout_has_already_expired() {
    let test = TestFixture::new();
    let duration = Duration::from_nanos(100);
    let timeout = Timeout::At(TimePoint::from_nanos(50));

    let time_controller = SimulationClockBuilder::<UnixDatagramShmCountingBitSet>::new(test.name())
        .config(test.config())
        .create_time_controller()
        .unwrap();
    let clock = Arc::new(
        SimulationClockBuilder::<UnixDatagramShmCountingBitSet>::new(test.name())
            .config(test.config())
            .open_clock()
            .unwrap(),
    );

    let _ = time_controller.advance(duration);

    let handle = clock.schedule(timeout).unwrap();
    let t = match handle {
        ScheduleHandle::TranslatedTimeout(t) => t,
        ScheduleHandle::GuardAndListener(_) => {
            assert_that!(shall_never_be_reached);
        }
    };
    assert_that!(t, eq Timeout::By(Duration::ZERO));
}

#[test]
fn schedule_returns_guard_and_listener_when_timeout_has_not_expired() {
    let test = TestFixture::new();

    let clock = Arc::new(
        SimulationClockBuilder::<UnixDatagramShmCountingBitSet>::new(test.name())
            .config(test.config())
            .open_clock()
            .unwrap(),
    );

    let timeout = Timeout::At(TimePoint::from_nanos(100));

    let handle = clock.schedule(timeout);
    assert_that!(handle, is_ok);
    if let ScheduleHandle::TranslatedTimeout(_) = handle.unwrap() {
        assert_that!(shall_never_be_reached);
    }
}

#[test]
fn advance_notifies_when_timeout_was_scheduled_with_schedule() {
    let test = TestFixture::new();
    let duration = Duration::from_nanos(100);
    let timeout = Timeout::By(duration);

    let time_controller = SimulationClockBuilder::<UnixDatagramShmCountingBitSet>::new(test.name())
        .config(test.config())
        .create_time_controller()
        .unwrap();
    let clock = Arc::new(
        SimulationClockBuilder::<UnixDatagramShmCountingBitSet>::new(test.name())
            .config(test.config())
            .open_clock()
            .unwrap(),
    );

    let handle = clock.schedule(timeout).unwrap();
    let (_guard, listener) = match handle {
        ScheduleHandle::GuardAndListener((g, l)) => (g, l),
        _ => {
            assert_that!(shall_never_be_reached);
        }
    };
    let reactor = iceoryx2_cal::reactor::posix_select::ReactorBuilder::new()
        .create(clock.clone())
        .unwrap();
    let _reactor_guard = reactor.attach(&*listener);

    assert_that!(reactor.try_wait(|_| {}).unwrap(), eq 0);

    let _ = time_controller.advance(duration / 2);
    assert_that!(reactor.try_wait(|_| {}).unwrap(), eq 0);

    let _ = time_controller.advance(duration);
    assert_that!(reactor.try_wait(|_| {}).unwrap(), eq 1);
}

#[test]
fn guard_deregisters_timeout_that_was_registered_with_schedule() {
    let test = TestFixture::new();
    let duration = Duration::from_nanos(100);
    let timeout = Timeout::By(duration);

    let time_controller = SimulationClockBuilder::<UnixDatagramShmCountingBitSet>::new(test.name())
        .config(test.config())
        .create_time_controller()
        .unwrap();
    let clock = Arc::new(
        SimulationClockBuilder::<UnixDatagramShmCountingBitSet>::new(test.name())
            .config(test.config())
            .open_clock()
            .unwrap(),
    );

    let handle = clock.schedule(timeout).unwrap();
    let (guard, listener) = match handle {
        ScheduleHandle::GuardAndListener((g, l)) => (g, l),
        _ => {
            assert_that!(shall_never_be_reached);
        }
    };
    let reactor = iceoryx2_cal::reactor::posix_select::ReactorBuilder::new()
        .create(clock.clone())
        .unwrap();
    let _reactor_guard = reactor.attach(&*listener);

    drop(guard);

    let _ = time_controller.advance(2 * duration);
    assert_that!(reactor.try_wait(|_| {}).unwrap(), eq 0);
}

#[test]
fn blocking_sleep_does_not_block_when_timeout_expired() {
    let test = TestFixture::new();
    let _watchdog = Watchdog::new();
    let duration = Duration::from_millis(1);
    let timeout = Timeout::At(TimePoint::from_nanos(100));

    let time_controller = SimulationClockBuilder::<UnixDatagramShmCountingBitSet>::new(test.name())
        .config(test.config())
        .create_time_controller()
        .unwrap();
    let clock = SimulationClockBuilder::<UnixDatagramShmCountingBitSet>::new(test.name())
        .config(test.config())
        .open_clock()
        .unwrap();

    let _ = time_controller.advance(duration);

    assert_that!(clock.blocking_sleep(timeout), is_ok);
}

#[test]
fn blocking_sleep_blocks_until_timeout_has_expired() {
    let test = TestFixture::new();
    let _watchdog = Watchdog::new();
    let duration = Duration::from_millis(12);
    let timeout = Timeout::At(TimePoint::from_millis(12));

    let time_controller = SimulationClockBuilder::<UnixDatagramShmCountingBitSet>::new(test.name())
        .config(test.config())
        .create_time_controller()
        .unwrap();
    let clock = SimulationClockBuilder::<UnixDatagramShmCountingBitSet>::new(test.name())
        .config(test.config())
        .open_clock()
        .unwrap();

    let start = clock.now().unwrap();
    thread_scope(|s| {
        s.thread_builder().spawn(|| {
            let _ = time_controller.advance(duration / 3);
            let _ = time_controller.advance(duration / 3);
            let _ = time_controller.advance(duration / 3);
        })?;
        Ok(())
    })
    .unwrap();

    assert_that!(clock.blocking_sleep(timeout), is_ok);
    let end = clock.now().unwrap();
    assert_that!(end - start, time_at_least duration);
}

#[test]
fn concurrent_time_controller_creation_succeeds_once() {
    let test = TestFixture::new();
    let name = test.name();
    let config = test.config();
    let _watchdog = Watchdog::new();
    let number_of_threads = (SystemInfo::NumberOfCpuCores.value()).clamp(2, 16);
    const NUMBER_OF_ITERATIONS: usize = 25;
    let barrier_handle = BarrierHandle::new();
    let barrier = BarrierBuilder::new(2 * number_of_threads as u32)
        .create(&barrier_handle)
        .unwrap();
    let mutex_handle = MutexHandle::new();
    let time_controller_storage: Arc<
        Mutex<Option<SimulationTimeController<UnixDatagramShmCountingBitSet>>>,
    > = Arc::new(MutexBuilder::new().create(None, &mutex_handle).unwrap());
    let success_counter = AtomicU64::new(0);

    thread_scope(|s| {
        for _ in 0..number_of_threads {
            let storage = time_controller_storage.clone();
            s.thread_builder().spawn(|| {
                barrier.wait();
                for _ in 0..NUMBER_OF_ITERATIONS {
                    if let Ok(time_controller) =
                        SimulationClockBuilder::<UnixDatagramShmCountingBitSet>::new(name)
                            .config(config)
                            .create_time_controller()
                    {
                        success_counter.fetch_add(1, Ordering::Relaxed);
                        let mut s = storage.lock().unwrap();
                        assert_that!((*s).replace(time_controller), is_none);
                    }
                }
            })?;
        }
        for _ in 0..number_of_threads {
            let storage = time_controller_storage.clone();
            s.thread_builder().spawn(|| {
                barrier.wait();
                for _ in 0..NUMBER_OF_ITERATIONS {
                    if let Ok(time_controller) =
                        SimulationClockBuilder::<UnixDatagramShmCountingBitSet>::new(name)
                            .config(config)
                            .open_or_create_time_controller()
                    {
                        success_counter.fetch_add(1, Ordering::Relaxed);
                        let mut s = storage.lock().unwrap();
                        assert_that!((*s).replace(time_controller), is_none);
                    }
                }
            })?;
        }
        Ok(())
    })
    .unwrap();

    assert_that!(success_counter.load(Ordering::Relaxed), eq 1);
    assert_that!(time_controller_storage.lock().unwrap(), is_some);
}

#[test]
fn concurrent_time_controller_opening_succeeds_once() {
    let test = TestFixture::new();
    let name = test.name();
    let config = test.config();
    let _watchdog = Watchdog::new();
    let number_of_threads = (SystemInfo::NumberOfCpuCores.value()).clamp(2, 16);
    const NUMBER_OF_ITERATIONS: usize = 25;
    let barrier_handle = BarrierHandle::new();
    let barrier = BarrierBuilder::new(number_of_threads as u32)
        .create(&barrier_handle)
        .unwrap();
    let mutex_handle = MutexHandle::new();
    let time_controller_storage: Arc<
        Mutex<Option<SimulationTimeController<UnixDatagramShmCountingBitSet>>>,
    > = Arc::new(MutexBuilder::new().create(None, &mutex_handle).unwrap());
    let success_counter = AtomicU64::new(0);

    let time_controller = SimulationClockBuilder::<UnixDatagramShmCountingBitSet>::new(name)
        .config(config)
        .create_time_controller()
        .unwrap();
    let _clock = SimulationClockBuilder::<UnixDatagramShmCountingBitSet>::new(name)
        .config(config)
        .open_clock()
        .unwrap();
    drop(time_controller);

    thread_scope(|s| {
        for _ in 0..number_of_threads {
            s.thread_builder().spawn(|| {
                barrier.wait();
                for _ in 0..NUMBER_OF_ITERATIONS {
                    let storage = time_controller_storage.clone();
                    if let Ok(time_controller) =
                        SimulationClockBuilder::<UnixDatagramShmCountingBitSet>::new(name)
                            .config(config)
                            .open_time_controller()
                    {
                        success_counter.fetch_add(1, Ordering::Relaxed);
                        let mut s = storage.lock().unwrap();
                        assert_that!((*s).replace(time_controller), is_none);
                    }
                }
            })?;
        }
        Ok(())
    })
    .unwrap();

    assert_that!(success_counter.load(Ordering::Relaxed), eq 1);
    assert_that!(time_controller_storage.lock().unwrap(), is_some);
}

#[test]
fn open_clock_concurrently_works() {
    let test = TestFixture::new();
    let name = test.name();
    let config = test.config();
    let _watchdog = Watchdog::new();
    let number_of_threads = (SystemInfo::NumberOfCpuCores.value()).clamp(2, 16);
    const NUMBER_OF_ITERATIONS: usize = 25;
    let barrier_handle = BarrierHandle::new();
    let barrier = BarrierBuilder::new(number_of_threads as u32)
        .create(&barrier_handle)
        .unwrap();
    let clock = SimulationClockBuilder::<UnixDatagramShmCountingBitSet>::new(name)
        .config(config)
        .open_clock();
    assert_that!(clock, is_ok);

    thread_scope(|s| {
        for _ in 0..number_of_threads {
            s.thread_builder().spawn(|| {
                barrier.wait();
                for _ in 0..NUMBER_OF_ITERATIONS {
                    assert_that!(
                        SimulationClockBuilder::<UnixDatagramShmCountingBitSet>::new(name)
                            .config(config)
                            .open_clock(),
                        is_ok
                    );
                }
            })?;
        }
        Ok(())
    })
    .unwrap();
}

#[test]
fn now_returns_correct_timepoint_when_time_is_advanced_concurrently() {
    let test = TestFixture::new();
    let name = test.name();
    let config = test.config();
    let _watchdog = Watchdog::new();
    const NUMBER_OF_ITERATIONS: usize = 250;
    let barrier_handle = BarrierHandle::new();
    let barrier = BarrierBuilder::new(2).create(&barrier_handle).unwrap();
    const DURATION: Duration = Duration::from_millis(100);

    let time_controller = SimulationClockBuilder::<UnixDatagramShmCountingBitSet>::new(name)
        .config(config)
        .create_time_controller()
        .unwrap();
    let clock = SimulationClockBuilder::<UnixDatagramShmCountingBitSet>::new(name)
        .config(config)
        .open_clock()
        .unwrap();

    let keep_running = AtomicBool::new(true);

    thread_scope(|s| {
        s.thread_builder().spawn(|| {
            barrier.wait();
            while keep_running.load(Ordering::Relaxed) {
                time_controller
                    .advance(DURATION)
                    .expect("advance time succeeds");
            }
        })?;
        s.thread_builder().spawn(|| {
            let mut old_time = TimePoint::new();
            barrier.wait();
            for _ in 0..NUMBER_OF_ITERATIONS {
                let current_time = clock.now().unwrap();
                let diff = (current_time - old_time).as_millis() as u128;
                if diff > 0 {
                    assert_that!(diff % DURATION.as_millis(), eq 0);
                    old_time = current_time;
                }
            }
            keep_running.store(false, Ordering::Relaxed);
        })?;
        Ok(())
    })
    .unwrap();
}

#[test]
fn schedule_on_succeeds_when_time_is_advanced_concurrently() {
    let test = TestFixture::new();
    let name = test.name();
    let config = test.config();
    let _watchdog = Watchdog::new();
    const NUMBER_OF_ITERATIONS: u128 = 10;
    let barrier_handle = BarrierHandle::new();
    let barrier = BarrierBuilder::new(2).create(&barrier_handle).unwrap();
    const DURATION: Duration = Duration::from_millis(100);

    let time_controller = SimulationClockBuilder::<UnixDatagramShmCountingBitSet>::new(name)
        .config(config)
        .create_time_controller()
        .unwrap();
    let clock = Arc::new(
        SimulationClockBuilder::<UnixDatagramShmCountingBitSet>::new(name)
            .config(config)
            .open_clock()
            .unwrap(),
    );

    let keep_running = AtomicBool::new(true);
    let listener_name = unsafe { FileName::new_unchecked_const(b"Picard") };
    let listener = <UnixDatagramShmCountingBitSet as Event>::ListenerBuilder::new(&listener_name)
        .create(clock.clone())
        .unwrap();

    thread_scope(|s| {
        s.thread_builder().spawn(|| {
            barrier.wait();
            while keep_running.load(Ordering::Relaxed) {
                time_controller
                    .advance(DURATION)
                    .expect("advance time succeeds");
            }
        })?;

        s.thread_builder().spawn(|| {
            barrier.wait();
            for i in 0..NUMBER_OF_ITERATIONS {
                let timeout =
                    Timeout::At(TimePoint::from_millis((i + 1) * 4 * DURATION.as_millis()));
                let handle = clock.schedule_on(timeout, &listener_name);
                assert_that!(handle, is_ok);
                match handle.unwrap() {
                    ScheduleOnHandle::Guard(_g) => {
                        // The timeout was in the future when schedule_on() was called.
                        assert_that!(listener.blocking_wait(&mut |_| {}).unwrap(), eq 1);
                    }
                    ScheduleOnHandle::TranslatedTimeout(_) => {
                        // The timeout has already expired - try_wait() is sufficient because the listener
                        // must be already notified.
                        assert_that!(listener.try_wait(&mut |_| {}).unwrap(), eq 1);
                    }
                }
            }
            keep_running.store(false, Ordering::Relaxed);
        })?;
        Ok(())
    })
    .unwrap();
}

#[test]
fn schedule_succeeds_when_time_is_advanced_concurrently() {
    let test = TestFixture::new();
    let name = test.name();
    let config = test.config();
    let _watchdog = Watchdog::new();
    const NUMBER_OF_ITERATIONS: u128 = 10;
    let barrier_handle = BarrierHandle::new();
    let barrier = BarrierBuilder::new(2).create(&barrier_handle).unwrap();
    const DURATION: Duration = Duration::from_millis(100);

    let time_controller = SimulationClockBuilder::<UnixDatagramShmCountingBitSet>::new(name)
        .config(config)
        .create_time_controller()
        .unwrap();
    let clock = Arc::new(
        SimulationClockBuilder::<UnixDatagramShmCountingBitSet>::new(name)
            .config(config)
            .open_clock()
            .unwrap(),
    );

    let keep_running = AtomicBool::new(true);
    let sleep_duration = Duration::from_micros(100);

    thread_scope(|s| {
        s.thread_builder().spawn(|| {
            barrier.wait();
            while keep_running.load(Ordering::Relaxed) {
                time_controller
                    .advance(DURATION)
                    .expect("advance time succeeds");
                nanosleep(sleep_duration).unwrap();
            }
        })?;

        s.thread_builder().spawn(|| {
            let reactor = iceoryx2_cal::reactor::posix_select::ReactorBuilder::new()
                .create(clock.clone())
                .unwrap();
            barrier.wait();
            for i in 0..NUMBER_OF_ITERATIONS {
                let timeout =
                    Timeout::At(TimePoint::from_millis((i + 1) * 4 * DURATION.as_millis()));
                let handle = clock.schedule(timeout);
                assert_that!(handle, is_ok);
                match handle.unwrap() {
                    ScheduleHandle::GuardAndListener((_g, l)) => {
                        let _reactor_guard = reactor.attach(&*l);
                        assert_that!(reactor.blocking_wait(|_| {}).unwrap(), eq 1);
                    }
                    ScheduleHandle::TranslatedTimeout(_) => {}
                }
            }
            keep_running.store(false, Ordering::Relaxed);
        })?;
        Ok(())
    })
    .unwrap();
}

#[test]
fn blocking_sleep_is_unblocked_when_time_is_advanced_concurrently() {
    let test = TestFixture::new();
    let name = test.name();
    let config = test.config();
    let _watchdog = Watchdog::new();
    let number_of_threads = (SystemInfo::NumberOfCpuCores.value()).clamp(2, 16) as u32;
    let barrier_handle = BarrierHandle::new();
    let barrier = BarrierBuilder::new(1 + number_of_threads)
        .create(&barrier_handle)
        .unwrap();
    const DURATION: Duration = Duration::from_millis(100);

    let time_controller = SimulationClockBuilder::<UnixDatagramShmCountingBitSet>::new(name)
        .config(config)
        .create_time_controller()
        .unwrap();
    let clock = Arc::new(
        SimulationClockBuilder::<UnixDatagramShmCountingBitSet>::new(name)
            .config(config)
            .open_clock()
            .unwrap(),
    );

    let keep_running = AtomicBool::new(true);
    let finished_threads = AtomicU32::new(0);
    let timeout = Timeout::At(TimePoint::from_millis(DURATION.as_millis() * 10));
    let sleep_duration = Duration::from_micros(100);

    thread_scope(|s| {
        s.thread_builder().spawn(|| {
            barrier.wait();
            while finished_threads.load(Ordering::Relaxed) != number_of_threads {
                time_controller
                    .advance(DURATION)
                    .expect("advance time succeeds");
                nanosleep(sleep_duration).unwrap();
            }
        })?;

        for _ in 0..number_of_threads {
            s.thread_builder().spawn(|| {
                barrier.wait();
                assert_that!(clock.blocking_sleep(timeout), is_ok);
                finished_threads.fetch_add(1, Ordering::Relaxed);
            })?;
            keep_running.store(false, Ordering::Relaxed);
        }
        Ok(())
    })
    .unwrap();
}

#[test]
fn schedule_different_timeouts_concurrently_works() {
    let test = TestFixture::new();
    let name = test.name();
    let config = test.config();
    let _watchdog = Watchdog::new();
    let number_of_threads = (SystemInfo::NumberOfCpuCores.value()).clamp(2, 16) as u64;
    const NUMBER_OF_ITERATIONS: u64 = 100;
    let barrier_handle = BarrierHandle::new();
    let barrier = BarrierBuilder::new(number_of_threads as u32)
        .create(&barrier_handle)
        .unwrap();
    let duration = Duration::from_millis(number_of_threads);

    let time_controller = SimulationClockBuilder::<UnixDatagramShmCountingBitSet>::new(name)
        .config(config)
        .create_time_controller()
        .unwrap();
    let clock = Arc::new(
        SimulationClockBuilder::<UnixDatagramShmCountingBitSet>::new(name)
            .config(config)
            .open_clock()
            .unwrap(),
    );

    let listener_name = unsafe { FileName::new_unchecked_const(b"Make It So!") };
    let listener = <UnixDatagramShmCountingBitSet as Event>::ListenerBuilder::new(&listener_name)
        .create(clock.clone())
        .unwrap();

    time_controller
        .advance(duration)
        .expect("advance time succeeds");

    thread_scope(|s| {
        for i in 0..number_of_threads / 2 {
            s.thread_builder().spawn(|| {
                let timeout = Timeout::At(TimePoint::from_millis(i as u128 + 1));
                barrier.wait();
                for _ in 0..NUMBER_OF_ITERATIONS {
                    assert_that!(clock.schedule_on(timeout, &listener_name), is_ok);
                }
            })?;
        }

        for i in 0..number_of_threads / 2 {
            s.thread_builder().spawn(|| {
                let timeout = Timeout::At(TimePoint::from_millis(i as u128 + 1));
                barrier.wait();
                for _ in 0..NUMBER_OF_ITERATIONS {
                    assert_that!(clock.schedule(timeout), is_ok);
                }
            })?;
        }

        Ok(())
    })
    .unwrap();

    assert_that!(listener.try_wait(&mut |_| {}).unwrap(), eq number_of_threads / 2 * NUMBER_OF_ITERATIONS);
}

#[test]
fn schedule_same_timeouts_concurrently_works() {
    let test = TestFixture::new();
    let name = test.name();
    let config = test.config();
    let _watchdog = Watchdog::new();
    let number_of_threads = (SystemInfo::NumberOfCpuCores.value()).clamp(2, 16) as u64;
    const NUMBER_OF_ITERATIONS: u64 = 100;
    let barrier_handle = BarrierHandle::new();
    let barrier = BarrierBuilder::new(number_of_threads as u32)
        .create(&barrier_handle)
        .unwrap();
    let duration = Duration::from_millis(number_of_threads);

    let time_controller = SimulationClockBuilder::<UnixDatagramShmCountingBitSet>::new(name)
        .config(config)
        .create_time_controller()
        .unwrap();
    let clock = Arc::new(
        SimulationClockBuilder::<UnixDatagramShmCountingBitSet>::new(name)
            .config(config)
            .open_clock()
            .unwrap(),
    );

    let listener_name = unsafe { FileName::new_unchecked_const(b"Chekov") };
    let listener = <UnixDatagramShmCountingBitSet as Event>::ListenerBuilder::new(&listener_name)
        .create(clock.clone())
        .unwrap();
    let timeout = Timeout::At(TimePoint::new());

    time_controller
        .advance(duration)
        .expect("advance time succeeds");

    thread_scope(|s| {
        for _ in 0..number_of_threads / 2 {
            s.thread_builder().spawn(|| {
                barrier.wait();
                for _ in 0..NUMBER_OF_ITERATIONS {
                    assert_that!(clock.schedule_on(timeout, &listener_name), is_ok);
                }
            })?;
        }

        for _ in 0..number_of_threads / 2 {
            s.thread_builder().spawn(|| {
                barrier.wait();
                for _ in 0..NUMBER_OF_ITERATIONS {
                    assert_that!(clock.schedule(timeout), is_ok);
                }
            })?;
        }

        Ok(())
    })
    .unwrap();

    assert_that!(listener.try_wait(&mut |_| {}).unwrap(), eq number_of_threads / 2 * NUMBER_OF_ITERATIONS);
}
