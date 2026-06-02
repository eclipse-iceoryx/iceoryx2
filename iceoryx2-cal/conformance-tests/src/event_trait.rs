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

extern crate iceoryx2_bb_loggers;

use iceoryx2_bb_testing_macros::conformance_tests;

#[allow(clippy::module_inception)]
#[conformance_tests]
pub mod event_trait {
    use alloc::collections::btree_set::BTreeSet;
    use alloc::{vec, vec::Vec};
    use core::time::Duration;
    use iceoryx2_bb_concurrency::atomic::AtomicU64;
    use iceoryx2_bb_concurrency::atomic::Ordering;
    use iceoryx2_bb_elementary_traits::testing::abandonable::Abandonable;
    use iceoryx2_bb_posix::barrier::*;
    use iceoryx2_bb_posix::clock::{Time, nanosleep};
    use iceoryx2_bb_posix::mutex::{MutexBuilder, MutexHandle};
    use iceoryx2_bb_posix::testing::generate_file_path;
    use iceoryx2_bb_posix::thread::thread_scope;
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_bb_testing::watchdog::Watchdog;
    use iceoryx2_bb_testing_macros::conformance_test;
    use iceoryx2_cal::event::event_state::{EventActivation, EventState};
    use iceoryx2_cal::event::{EventId, *};
    use iceoryx2_cal::named_concept::*;
    use iceoryx2_cal::testing::*;

    const TIMEOUT: Duration = Duration::from_millis(25);

    #[conformance_test]
    pub fn create_works<E: EventState, Sut: Event<E>>() {
        let name = generate_file_path().file_name();
        let config = generate_isolated_config::<Sut>();

        let sut_listener = Sut::ListenerBuilder::new(&name)
            .config(&config)
            .create()
            .unwrap();
        let sut_notifier = Sut::NotifierBuilder::new(&name)
            .config(&config)
            .open()
            .unwrap();

        assert_that!(*sut_listener.name(), eq name);
        assert_that!(*sut_notifier.name(), eq name);
    }

    #[conformance_test]
    pub fn listener_cleans_up_when_out_of_scope<E: EventState, Sut: Event<E>>() {
        let name = generate_file_path().file_name();
        let config = generate_isolated_config::<Sut>();

        assert_that!(Sut::does_exist_cfg(&name, &config).unwrap(), eq false);
        let sut_listener = Sut::ListenerBuilder::new(&name)
            .config(&config)
            .create()
            .unwrap();
        assert_that!(Sut::does_exist_cfg(&name, &config).unwrap(), eq true);

        drop(sut_listener);
        assert_that!(Sut::does_exist_cfg(&name, &config).unwrap(), eq false);
    }

    #[conformance_test]
    pub fn cannot_be_created_twice<E: EventState, Sut: Event<E>>() {
        let name = generate_file_path().file_name();
        let config = generate_isolated_config::<Sut>();

        let _sut = Sut::ListenerBuilder::new(&name)
            .config(&config)
            .create()
            .unwrap();
        let sut = Sut::ListenerBuilder::new(&name).config(&config).create();

        assert_that!(sut, is_err);
        assert_that!(sut.err().unwrap(), eq ListenerCreateError::AlreadyExists);
    }

    #[conformance_test]
    pub fn cannot_open_non_existing<E: EventState, Sut: Event<E>>() {
        let name = generate_file_path().file_name();
        let config = generate_isolated_config::<Sut>();

        let sut = Sut::NotifierBuilder::new(&name).config(&config).open();

        assert_that!(sut, is_err);
        assert_that!(sut.err().unwrap(), eq NotifierOpenError::DoesNotExist);
    }

    #[conformance_test]
    pub fn notify_with_same_id_does_not_lead_to_non_blocking_timed_wait<
        E: EventState,
        Sut: Event<E>,
    >() {
        let _watchdog = Watchdog::new();
        const REPETITIONS: u64 = 8;
        let name = generate_file_path().file_name();
        let config = generate_isolated_config::<Sut>();

        let sut_listener = Sut::ListenerBuilder::new(&name)
            .config(&config)
            .create()
            .unwrap();
        let sut_notifier = Sut::NotifierBuilder::new(&name)
            .config(&config)
            .open()
            .unwrap();

        let trigger_id = EventId::new(0);

        for _ in 0..REPETITIONS {
            sut_notifier.notify(trigger_id).unwrap();
        }

        assert_that!(sut_listener.try_wait(|_| {}).unwrap(), ge 1);

        let now = Time::now().unwrap();
        let result = sut_listener.timed_wait(|_| {}, TIMEOUT).unwrap();

        if result > 0 {
            assert_that!(now.elapsed().unwrap(), time_at_least TIMEOUT );
        }
    }

    fn sending_notification_works<
        E: EventState,
        Sut: Event<E>,
        F: Fn(&Sut::Listener) -> Result<Option<EventId>, ListenerWaitError>,
    >(
        wait_call: F,
    ) {
        let _watchdog = Watchdog::new();
        const REPETITIONS: usize = 8;
        let name = generate_file_path().file_name();
        let config = generate_isolated_config::<Sut>();

        let sut_listener = Sut::ListenerBuilder::new(&name)
            .config(&config)
            .create()
            .unwrap();
        let sut_notifier = Sut::NotifierBuilder::new(&name)
            .config(&config)
            .open()
            .unwrap();

        for i in 0..REPETITIONS {
            sut_notifier.notify(EventId::new(i)).unwrap();
            let result = wait_call(&sut_listener).unwrap();
            assert_that!(result.unwrap(), eq EventId::new(i));
        }
    }

    #[conformance_test]
    pub fn sending_notification_and_try_wait_works<E: EventState, Sut: Event<E>>() {
        sending_notification_works::<E, Sut, _>(|sut| {
            let mut event_id = None;
            sut.try_wait(|id| event_id = Some(id.id))?;
            Ok(event_id)
        });
    }

    #[conformance_test]
    pub fn sending_notification_and_timed_wait_works<E: EventState, Sut: Event<E>>() {
        sending_notification_works::<E, Sut, _>(|sut| {
            let mut event_id = None;
            sut.timed_wait(|id| event_id = Some(id.id), TIMEOUT)?;
            Ok(event_id)
        });
    }

    #[conformance_test]
    pub fn sending_notification_and_blocking_wait_works<E: EventState, Sut: Event<E>>() {
        sending_notification_works::<E, Sut, _>(|sut| {
            let mut event_id = None;
            sut.blocking_wait(|id| event_id = Some(id.id))?;
            Ok(event_id)
        });
    }

    fn sending_multiple_notifications_before_wait_works<
        E: EventState,
        Sut: Event<E>,
        F: Fn(&Sut::Listener) -> Result<Vec<EventActivation>, ListenerWaitError>,
    >(
        wait_call: F,
    ) {
        const REPETITIONS: usize = 8;
        let name = generate_file_path().file_name();
        let config = generate_isolated_config::<Sut>();

        let sut_listener = Sut::ListenerBuilder::new(&name)
            .config(&config)
            .create()
            .unwrap();
        let sut_notifier = Sut::NotifierBuilder::new(&name)
            .config(&config)
            .open()
            .unwrap();

        for i in 0..REPETITIONS {
            sut_notifier.notify(EventId::new(i)).unwrap();
        }

        let mut ids = BTreeSet::new();
        let events = wait_call(&sut_listener).unwrap();
        for event in events {
            assert_that!(event.count, eq 1);
            assert_that!(ids.insert(event.id), eq true);
        }
    }

    #[conformance_test]
    pub fn sending_multiple_notifications_before_try_wait_works<E: EventState, Sut: Event<E>>() {
        sending_multiple_notifications_before_wait_works::<E, Sut, _>(|sut| {
            let mut event_ids = vec![];
            sut.try_wait(|id| event_ids.push(id))?;
            Ok(event_ids)
        });
    }

    #[conformance_test]
    pub fn sending_multiple_notifications_before_timed_wait_works<E: EventState, Sut: Event<E>>() {
        sending_multiple_notifications_before_wait_works::<E, Sut, _>(|sut| {
            let mut event_ids = vec![];
            sut.timed_wait(|id| event_ids.push(id), TIMEOUT)?;
            Ok(event_ids)
        });
    }

    #[conformance_test]
    pub fn sending_multiple_notifications_before_blocking_wait_works<
        E: EventState,
        Sut: Event<E>,
    >() {
        sending_multiple_notifications_before_wait_works::<E, Sut, _>(|sut| {
            let mut event_ids = vec![];
            sut.blocking_wait(|id| event_ids.push(id))?;
            Ok(event_ids)
        });
    }

    fn sending_multiple_notifications_from_multiple_sources_before_wait_works<
        E: EventState,
        Sut: Event<E>,
        F: Fn(&Sut::Listener) -> Result<Vec<EventActivation>, ListenerWaitError>,
    >(
        wait_call: F,
    ) {
        const REPETITIONS: u64 = 2;
        const SOURCES: u64 = 4;
        let name = generate_file_path().file_name();
        let config = generate_isolated_config::<Sut>();
        let mut sources = vec![];

        let sut_listener = Sut::ListenerBuilder::new(&name)
            .config(&config)
            .event_id_max(EventId::new(128))
            .create()
            .unwrap();
        for _ in 0..SOURCES {
            sources.push(
                Sut::NotifierBuilder::new(&name)
                    .config(&config)
                    .open()
                    .unwrap(),
            );
        }

        let mut event_ids = vec![];
        let mut event_counter = 0;
        for _ in 0..REPETITIONS {
            for notifier in &sources {
                event_counter += 1;
                assert_that!(notifier.notify(EventId::new(event_counter)), is_ok);
                event_ids.push(event_counter);
            }
        }

        let events = wait_call(&sut_listener).unwrap();
        for event in &events {
            assert_that!(event_ids, contains event.id.as_value());
        }
        assert_that!(events, len event_counter);
    }

    #[conformance_test]
    pub fn sending_multiple_notifications_from_multiple_sources_before_try_wait_works<
        E: EventState,
        Sut: Event<E>,
    >() {
        sending_multiple_notifications_from_multiple_sources_before_wait_works::<E, Sut, _>(
            |sut| {
                let mut results = vec![];
                sut.try_wait(|event| results.push(event))?;
                Ok(results)
            },
        );
    }

    #[conformance_test]
    pub fn sending_multiple_notifications_from_multiple_sources_before_timed_wait_works<
        E: EventState,
        Sut: Event<E>,
    >() {
        sending_multiple_notifications_from_multiple_sources_before_wait_works::<E, Sut, _>(
            |sut| {
                let mut results = vec![];
                sut.timed_wait(|event| results.push(event), TIMEOUT)?;
                Ok(results)
            },
        );
    }

    #[conformance_test]
    pub fn sending_multiple_notifications_from_multiple_sources_before_blocking_wait_works<
        E: EventState,
        Sut: Event<E>,
    >() {
        sending_multiple_notifications_from_multiple_sources_before_wait_works::<E, Sut, _>(
            |sut| {
                let mut results = vec![];
                sut.blocking_wait(|event| results.push(event))?;
                Ok(results)
            },
        );
    }

    #[conformance_test]
    pub fn try_wait_does_not_block<E: EventState, Sut: Event<E>>() {
        let name = generate_file_path().file_name();
        let config = generate_isolated_config::<Sut>();

        let sut_listener = Sut::ListenerBuilder::new(&name)
            .config(&config)
            .create()
            .unwrap();
        let _sut_notifier = Sut::NotifierBuilder::new(&name)
            .config(&config)
            .open()
            .unwrap();

        let result = sut_listener.try_wait(|_| {}).unwrap();
        assert_that!(result, eq 0);
    }

    #[conformance_test]
    pub fn timed_wait_does_block_for_at_least_timeout<E: EventState, Sut: Event<E>>() {
        let name = generate_file_path().file_name();
        let config = generate_isolated_config::<Sut>();

        let sut_listener = Sut::ListenerBuilder::new(&name)
            .config(&config)
            .create()
            .unwrap();
        let _sut_notifier = Sut::NotifierBuilder::new(&name)
            .config(&config)
            .open()
            .unwrap();

        let start = Time::now().unwrap();
        let result = sut_listener.timed_wait(|_| {}, TIMEOUT).unwrap();
        assert_that!(result, eq 0);
        assert_that!(start.elapsed().unwrap(), time_at_least TIMEOUT);
    }

    #[conformance_test]
    pub fn blocking_wait_blocks_until_notification_arrives<E: EventState, Sut: Event<E>>() {
        let _watchdog = Watchdog::new();
        let name = generate_file_path().file_name();
        let handle = MutexHandle::new();
        let config = MutexBuilder::new()
            .create(generate_isolated_config::<Sut>(), &handle)
            .unwrap();

        let counter = AtomicU64::new(0);
        let counter_old = AtomicU64::new(0);
        let handle = BarrierHandle::new();
        let barrier = BarrierBuilder::new(2).create(&handle).unwrap();

        thread_scope(|s| {
            s.thread_builder().spawn(|| {
                let sut_listener = Sut::ListenerBuilder::new(&name)
                    .config(&config.lock().unwrap())
                    .create()
                    .unwrap();
                barrier.wait();
                let mut call_counter = 0;
                let result = sut_listener
                    .blocking_wait(|event| {
                        call_counter += 1;
                        assert_that!(event.id, eq EventId::new(4))
                    })
                    .unwrap();
                counter.store(1, Ordering::SeqCst);
                assert_that!(result, eq 1);
                assert_that!(call_counter, eq 1);
            })?;

            barrier.wait();
            let sut_notifier = Sut::NotifierBuilder::new(&name)
                .config(&config.lock().unwrap())
                .open()
                .unwrap();
            nanosleep(TIMEOUT).unwrap();
            counter_old.store(counter.load(Ordering::SeqCst), Ordering::SeqCst);
            sut_notifier.notify(EventId::new(4)).unwrap();

            Ok(())
        })
        .unwrap();
        assert_that!(counter_old.load(Ordering::SeqCst), eq 0);
        assert_that!(counter.load(Ordering::SeqCst), eq 1);
    }

    /// windows sporadically instantly wakes up in a timed receive operation
    #[cfg(not(target_os = "windows"))]
    #[conformance_test]
    pub fn timed_wait_blocks_until_notification_arrives<E: EventState, Sut: Event<E>>() {
        let _watchdog = Watchdog::new();
        let name = generate_file_path().file_name();
        let handle = MutexHandle::new();
        let config = MutexBuilder::new()
            .create(generate_isolated_config::<Sut>(), &handle)
            .unwrap();

        let counter = AtomicU64::new(0);
        let counter_old = AtomicU64::new(0);
        let handle = BarrierHandle::new();
        let barrier = BarrierBuilder::new(2).create(&handle).unwrap();

        thread_scope(|s| {
            s.thread_builder().spawn(|| {
                let sut_listener = Sut::ListenerBuilder::new(&name)
                    .config(&config.lock().unwrap())
                    .create()
                    .unwrap();
                barrier.wait();
                let mut call_counter = 0;
                let result = sut_listener
                    .timed_wait(
                        |event| {
                            call_counter += 1;
                            assert_that!(event.id.as_value(), eq 2);
                        },
                        TIMEOUT * 1000,
                    )
                    .unwrap();
                counter.store(1, Ordering::SeqCst);
                assert_that!(result, eq 1);
                assert_that!(call_counter, eq 1);
            })?;

            barrier.wait();
            let sut_notifier = Sut::NotifierBuilder::new(&name)
                .config(&config.lock().unwrap())
                .open()
                .unwrap();
            nanosleep(TIMEOUT).unwrap();
            counter_old.store(counter.load(Ordering::SeqCst), Ordering::SeqCst);
            sut_notifier.notify(EventId::new(2)).unwrap();

            Ok(())
        })
        .unwrap();
        assert_that!(counter_old.load(Ordering::SeqCst), eq 0);
        assert_that!(counter.load(Ordering::SeqCst), eq 1);
    }

    #[conformance_test]
    pub fn defaults_for_configuration_are_set_correctly<E: EventState, Sut: Event<E>>() {
        let config = <Sut as NamedConceptMgmt>::Configuration::default();
        assert_that!(*config.get_suffix(), eq Sut::default_suffix());
    }

    #[conformance_test]
    pub fn setting_trigger_id_limit_works<E: EventState, Sut: Event<E>>() {
        const EVENT_ID_MAX: EventId = EventId::new(123);
        let name = generate_file_path().file_name();
        let config = generate_isolated_config::<Sut>();

        let _sut_listener = Sut::ListenerBuilder::new(&name)
            .event_id_max(EVENT_ID_MAX)
            .config(&config)
            .create()
            .unwrap();
        let sut_notifier = Sut::NotifierBuilder::new(&name)
            .config(&config)
            .open()
            .unwrap();

        assert_that!(sut_notifier.event_id_max(), eq EVENT_ID_MAX);
    }

    #[conformance_test]
    pub fn triggering_up_to_trigger_id_max_works<E: EventState, Sut: Event<E>>() {
        const EVENT_ID_MAX: EventId = EventId::new(24);
        let name = generate_file_path().file_name();
        let config = generate_isolated_config::<Sut>();

        let sut_listener = Sut::ListenerBuilder::new(&name)
            .event_id_max(EVENT_ID_MAX)
            .config(&config)
            .create()
            .unwrap();
        let sut_notifier = Sut::NotifierBuilder::new(&name)
            .config(&config)
            .open()
            .unwrap();

        for i in 0..EVENT_ID_MAX.as_value() {
            assert_that!(sut_notifier.notify(EventId::new(i)), is_ok);
        }

        let result = sut_notifier.notify(EventId::new(EVENT_ID_MAX.as_value() + 1));
        assert_that!(result, is_err);
        assert_that!(
            result.err().unwrap(), eq
            NotifierNotifyError::EventIdOutOfBounds
        );

        let mut ids = BTreeSet::new();
        let result = sut_listener
            .try_wait(|event| {
                assert_that!(event.id, lt EVENT_ID_MAX);
                assert_that!(ids.insert(event.id), eq true);
            })
            .unwrap();
        assert_that!(result, eq EVENT_ID_MAX.as_value() as u64);

        let result = sut_listener.try_wait(|_| {}).unwrap();
        assert_that!(result, eq 0);
    }

    fn wait_all_collects_all_triggers<
        E: EventState,
        Sut: Event<E>,
        F: FnMut(&mut Vec<EventId>, &Sut::Listener),
    >(
        mut wait_call: F,
    ) {
        let _watchdog = Watchdog::new();
        const REPETITIONS: usize = 8;
        let name = generate_file_path().file_name();
        let config = generate_isolated_config::<Sut>();

        let sut_listener = Sut::ListenerBuilder::new(&name)
            .event_id_max(EventId::new(REPETITIONS))
            .config(&config)
            .create()
            .unwrap();
        let sut_notifier = Sut::NotifierBuilder::new(&name)
            .config(&config)
            .open()
            .unwrap();

        for i in 1..=REPETITIONS {
            for n in 0..i {
                sut_notifier.notify(EventId::new(n as _)).unwrap();
            }

            let mut vec_of_ids = vec![];
            wait_call(&mut vec_of_ids, &sut_listener);

            assert_that!(vec_of_ids, len i);
            for n in 0..i {
                assert_that!(vec_of_ids, contains EventId::new(n));
            }
        }
    }

    #[conformance_test]
    pub fn try_wait_all_collects_all_triggers<E: EventState, Sut: Event<E>>() {
        wait_all_collects_all_triggers::<E, Sut, _>(|v, sut: &Sut::Listener| {
            sut.try_wait(|id| v.push(id.id)).unwrap();
        });
    }

    #[conformance_test]
    pub fn timed_wait_all_collects_all_triggers<E: EventState, Sut: Event<E>>() {
        wait_all_collects_all_triggers::<E, Sut, _>(|v, sut: &Sut::Listener| {
            sut.timed_wait(|id| v.push(id.id), TIMEOUT * 1000).unwrap();
        });
    }

    #[conformance_test]
    pub fn blocking_wait_all_collects_all_triggers<E: EventState, Sut: Event<E>>() {
        wait_all_collects_all_triggers::<E, Sut, _>(|v, sut: &Sut::Listener| {
            sut.blocking_wait(|id| v.push(id.id)).unwrap();
        });
    }

    fn wait_wakes_up_on_notify<
        E: EventState,
        Sut: Event<E>,
        F: FnMut(&mut Vec<EventId>, &Sut::Listener) + Send,
    >(
        wait_call: F,
    ) {
        let mut wait_call = wait_call;
        let _watchdog = Watchdog::new();
        let name = generate_file_path().file_name();
        let barrier_handle = BarrierHandle::new();
        let barrier = BarrierBuilder::new(2).create(&barrier_handle).unwrap();
        let counter = AtomicU64::new(0);
        let id = EventId::new(5);
        let mutex_handle = MutexHandle::new();
        let config = MutexBuilder::new()
            .create(generate_isolated_config::<Sut>(), &mutex_handle)
            .unwrap();

        thread_scope(|s| {
            s.thread_builder().spawn(|| {
                let sut_listener = Sut::ListenerBuilder::new(&name)
                    .config(&config.lock().unwrap())
                    .create()
                    .unwrap();
                barrier.wait();

                let mut id_vec = vec![];
                wait_call(&mut id_vec, &sut_listener);
                counter.fetch_add(1, Ordering::Relaxed);

                assert_that!(id_vec, len 1);
                assert_that!(id_vec[0], eq id);
            })?;

            barrier.wait();
            let sut_notifier = Sut::NotifierBuilder::new(&name)
                .config(&config.lock().unwrap())
                .open()
                .unwrap();
            nanosleep(TIMEOUT).unwrap();
            assert_that!(counter.load(Ordering::Relaxed), eq 0);
            sut_notifier.notify(id).unwrap();

            Ok(())
        })
        .unwrap();
        assert_that!(counter.load(Ordering::Relaxed), eq 1);
    }

    #[conformance_test]
    pub fn timed_wait_wakes_up_on_notify<E: EventState, Sut: Event<E>>() {
        wait_wakes_up_on_notify::<E, Sut, _>(|v, sut: &Sut::Listener| {
            sut.timed_wait(|id| v.push(id.id), TIMEOUT * 1000).unwrap();
        });
    }

    #[conformance_test]
    pub fn blocking_wait_wakes_up_on_notify<E: EventState, Sut: Event<E>>() {
        wait_wakes_up_on_notify::<E, Sut, _>(|v, sut: &Sut::Listener| {
            sut.blocking_wait(|id| v.push(id.id)).unwrap();
        });
    }

    #[conformance_test]
    pub fn out_of_scope_listener_shall_not_corrupt_notifier<E: EventState, Sut: Event<E>>() {
        let name = generate_file_path().file_name();
        let config = generate_isolated_config::<Sut>();

        let sut_listener = Sut::ListenerBuilder::new(&name)
            .config(&config)
            .create()
            .unwrap();
        let sut_notifier = Sut::NotifierBuilder::new(&name)
            .config(&config)
            .open()
            .unwrap();

        drop(sut_listener);

        let result = sut_notifier.notify(EventId::new(0));

        // either present a disconnect error when available or continue sending without counterpart, for
        // instance when the event is network socket based
        if result.is_err() {
            assert_that!(result.err().unwrap(), eq NotifierNotifyError::Disconnected);
        }
    }

    #[conformance_test]
    pub fn abandoning_listener_keeps_event<E: EventState, Sut: Event<E>>() {
        let _watchdog = Watchdog::new();
        let name = generate_file_path().file_name();
        let config = generate_isolated_config::<Sut>();

        let sut_listener = Sut::ListenerBuilder::new(&name)
            .config(&config)
            .create()
            .unwrap();

        Sut::Listener::abandon(sut_listener);

        assert_that!(Sut::does_exist_cfg(&name, &config), eq Ok(true));
        assert_that!(unsafe { Sut::remove_cfg(&name, &config).unwrap() }, eq true);
    }

    #[conformance_test]
    pub fn abandoning_notifier_keeps_event<E: EventState, Sut: Event<E>>() {
        let _watchdog = Watchdog::new();
        let name = generate_file_path().file_name();
        let config = generate_isolated_config::<Sut>();

        let _sut_listener = Sut::ListenerBuilder::new(&name)
            .config(&config)
            .create()
            .unwrap();

        let sut_notifier = Sut::NotifierBuilder::new(&name)
            .config(&config)
            .open()
            .unwrap();

        sut_notifier.abandon();

        assert_that!(Sut::does_exist_cfg(&name, &config), eq Ok(true));
    }

    #[conformance_test]
    pub fn sending_notification_many_times_never_leads_to_error<E: EventState, Sut: Event<E>>() {
        let _watchdog = Watchdog::new();
        const REPETITIONS: usize = 4096 * 128;
        let name = generate_file_path().file_name();
        let config = generate_isolated_config::<Sut>();

        let sut_listener = Sut::ListenerBuilder::new(&name)
            .config(&config)
            .create()
            .unwrap();
        let sut_notifier = Sut::NotifierBuilder::new(&name)
            .config(&config)
            .fail_when_buffer_is_full(true)
            .open()
            .unwrap();

        for _ in 0..REPETITIONS {
            assert_that!(sut_notifier.notify(EventId::new(3)), is_ok);
        }

        let result = sut_listener
            .try_wait(|event| assert_that!(event.id, eq EventId::new(3)))
            .unwrap();
        assert_that!(result, le 1);
    }

    #[conformance_test]
    pub fn notifier_and_listener_return_same_max_event_count<E: EventState, Sut: Event<E>>() {
        let name = generate_file_path().file_name();
        let config = generate_isolated_config::<Sut>();

        let sut_listener = Sut::ListenerBuilder::new(&name)
            .config(&config)
            .create()
            .unwrap();
        let sut_notifier = Sut::NotifierBuilder::new(&name)
            .config(&config)
            .open()
            .unwrap();

        assert_that!(sut_notifier.max_event_count(), eq sut_listener.max_event_count());
    }

    #[conformance_test]
    pub fn events_are_counted<E: EventState, Sut: Event<E>>() {
        let _watchdog = Watchdog::new();
        let name = generate_file_path().file_name();
        let config = generate_isolated_config::<Sut>();

        let sut_listener = Sut::ListenerBuilder::new(&name)
            .config(&config)
            .create()
            .unwrap();
        let sut_notifier = Sut::NotifierBuilder::new(&name)
            .config(&config)
            .open()
            .unwrap();

        let event_count = sut_listener.max_event_count().min(10);
        for _ in 0..event_count {
            sut_notifier.notify(EventId::new(0)).unwrap();
        }

        let result = sut_listener
            .try_wait(|event| assert_that!(event.count, eq event_count))
            .unwrap();
        assert_that!(result, eq event_count);
    }

    #[conformance_test]
    pub fn many_events_notified_multiple_times_are_counted<E: EventState, Sut: Event<E>>() {
        let _watchdog = Watchdog::new();
        const NUMBER_OF_EVENTS: usize = 10;
        let name = generate_file_path().file_name();
        let config = generate_isolated_config::<Sut>();

        let sut_listener = Sut::ListenerBuilder::new(&name)
            .config(&config)
            .event_id_max(EventId::new(NUMBER_OF_EVENTS - 1))
            .create()
            .unwrap();
        let sut_notifier = Sut::NotifierBuilder::new(&name)
            .config(&config)
            .open()
            .unwrap();

        let event_count = sut_listener.max_event_count().min(10);

        for event_id in 0..NUMBER_OF_EVENTS {
            for _ in 0..event_count {
                sut_notifier.notify(EventId::new(event_id)).unwrap();
            }
        }

        let result = sut_listener
            .try_wait(|event| {
                assert_that!(event.count, eq event_count);
            })
            .unwrap();
        assert_that!(result, eq(NUMBER_OF_EVENTS as u64 * event_count));
    }
}
