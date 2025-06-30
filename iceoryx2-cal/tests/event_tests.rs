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

#[generic_tests::define]
mod event {
    use core::sync::atomic::{AtomicU64, Ordering};
    use core::time::Duration;
    use std::collections::HashSet;
    use std::sync::{Barrier, Mutex};
    use std::time::Instant;

    use iceoryx2_bb_container::semantic_string::*;
    use iceoryx2_bb_posix::barrier::*;
    use iceoryx2_bb_system_types::file_name::FileName;
    use iceoryx2_bb_testing::watchdog::Watchdog;
    use iceoryx2_bb_testing::{assert_that, test_requires};
    use iceoryx2_cal::event::{TriggerId, *};
    use iceoryx2_cal::named_concept::*;
    use iceoryx2_cal::testing::*;

    const TIMEOUT: Duration = Duration::from_millis(25);

    #[test]
    fn create_works<Sut: Event>() {
        let name = generate_name();
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

    #[test]
    fn listener_cleans_up_when_out_of_scope<Sut: Event>() {
        let name = generate_name();
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

    #[test]
    fn cannot_be_created_twice<Sut: Event>() {
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let _sut = Sut::ListenerBuilder::new(&name)
            .config(&config)
            .create()
            .unwrap();
        let sut = Sut::ListenerBuilder::new(&name).config(&config).create();

        assert_that!(sut, is_err);
        assert_that!(sut.err().unwrap(), eq ListenerCreateError::AlreadyExists);
    }

    #[test]
    fn cannot_open_non_existing<Sut: Event>() {
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut = Sut::NotifierBuilder::new(&name).config(&config).open();

        assert_that!(sut, is_err);
        assert_that!(sut.err().unwrap(), eq NotifierCreateError::DoesNotExist);
    }

    #[test]
    fn notify_with_same_id_does_not_lead_to_non_blocking_timed_wait<Sut: Event>() {
        let _watchdog = Watchdog::new();
        const REPETITIONS: u64 = 8;
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut_listener = Sut::ListenerBuilder::new(&name)
            .config(&config)
            .create()
            .unwrap();
        let sut_notifier = Sut::NotifierBuilder::new(&name)
            .config(&config)
            .open()
            .unwrap();

        let trigger_id = TriggerId::new(0);

        for _ in 0..REPETITIONS {
            sut_notifier.notify(trigger_id).unwrap();
        }

        assert_that!(sut_listener.try_wait_one().unwrap(), is_some);

        let now = Instant::now();
        let result = sut_listener.timed_wait_one(TIMEOUT).unwrap();

        if result.is_some() {
            assert_that!(result, eq Some(trigger_id));
        } else {
            assert_that!(now.elapsed(), time_at_least TIMEOUT );
        }
    }

    fn sending_notification_works<
        Sut: Event,
        F: Fn(&Sut::Listener) -> Result<Option<TriggerId>, ListenerWaitError>,
    >(
        wait_call: F,
    ) {
        let _watchdog = Watchdog::new();
        const REPETITIONS: usize = 8;
        let name = generate_name();
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
            sut_notifier.notify(TriggerId::new(i)).unwrap();
            let result = wait_call(&sut_listener).unwrap();
            assert_that!(result.unwrap(), eq TriggerId::new(i));
        }
    }

    #[test]
    fn sending_notification_and_try_wait_works<Sut: Event>() {
        sending_notification_works::<Sut, _>(|sut| sut.try_wait_one());
    }

    #[test]
    fn sending_notification_and_timed_wait_works<Sut: Event>() {
        sending_notification_works::<Sut, _>(|sut| sut.timed_wait_one(TIMEOUT));
    }

    #[test]
    fn sending_notification_and_blocking_wait_works<Sut: Event>() {
        sending_notification_works::<Sut, _>(|sut| sut.blocking_wait_one());
    }

    fn sending_multiple_notifications_before_wait_works<
        Sut: Event,
        F: Fn(&Sut::Listener) -> Result<Option<TriggerId>, ListenerWaitError>,
    >(
        wait_call: F,
    ) {
        const REPETITIONS: usize = 8;
        let name = generate_name();
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
            sut_notifier.notify(TriggerId::new(i)).unwrap();
        }

        let mut ids = HashSet::new();
        for _ in 0..REPETITIONS {
            let result = wait_call(&sut_listener).unwrap();
            assert_that!(result, is_some);
            let result = result.unwrap();
            assert_that!(result.as_value(), lt REPETITIONS);
            assert_that!(ids.insert(result), eq true);
        }
    }

    #[test]
    fn sending_multiple_notifications_before_try_wait_works<Sut: Event>() {
        sending_multiple_notifications_before_wait_works::<Sut, _>(|sut| sut.try_wait_one());
    }

    #[test]
    fn sending_multiple_notifications_before_timed_wait_works<Sut: Event>() {
        sending_multiple_notifications_before_wait_works::<Sut, _>(|sut| {
            sut.timed_wait_one(TIMEOUT)
        });
    }

    #[test]
    fn sending_multiple_notifications_before_blocking_wait_works<Sut: Event>() {
        sending_multiple_notifications_before_wait_works::<Sut, _>(|sut| sut.blocking_wait_one());
    }

    fn sending_multiple_notifications_from_multiple_sources_before_wait_works<
        Sut: Event,
        F: Fn(&Sut::Listener) -> Result<Option<TriggerId>, ListenerWaitError>,
    >(
        wait_call: F,
    ) {
        const REPETITIONS: usize = 2;
        const SOURCES: usize = 4;
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();
        let mut sources = vec![];

        let sut_listener = Sut::ListenerBuilder::new(&name)
            .config(&config)
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
        for i in 0..REPETITIONS {
            for (n, notifier) in sources.iter().enumerate() {
                let event_id = n * (SOURCES + REPETITIONS + 1) + i;
                assert_that!(notifier.notify(TriggerId::new(event_id)), is_ok);
                event_ids.push(event_id);
            }
        }

        for _ in 0..REPETITIONS {
            for _ in 0..SOURCES {
                let result = wait_call(&sut_listener).unwrap();
                assert_that!(result, is_some);
                assert_that!(event_ids, contains result.unwrap().as_value());
            }
        }
    }

    #[test]
    fn sending_multiple_notifications_from_multiple_sources_before_try_wait_works<Sut: Event>() {
        sending_multiple_notifications_from_multiple_sources_before_wait_works::<Sut, _>(|sut| {
            sut.try_wait_one()
        });
    }

    #[test]
    fn sending_multiple_notifications_from_multiple_sources_before_timed_wait_works<Sut: Event>() {
        sending_multiple_notifications_from_multiple_sources_before_wait_works::<Sut, _>(|sut| {
            sut.timed_wait_one(TIMEOUT)
        });
    }

    #[test]
    fn sending_multiple_notifications_from_multiple_sources_before_blocking_wait_works<
        Sut: Event,
    >() {
        sending_multiple_notifications_from_multiple_sources_before_wait_works::<Sut, _>(|sut| {
            sut.blocking_wait_one()
        });
    }

    #[test]
    fn try_wait_does_not_block<Sut: Event>() {
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut_listener = Sut::ListenerBuilder::new(&name)
            .config(&config)
            .create()
            .unwrap();
        let _sut_notifier = Sut::NotifierBuilder::new(&name)
            .config(&config)
            .open()
            .unwrap();

        let result = sut_listener.try_wait_one().unwrap();
        assert_that!(result, is_none);
    }

    #[test]
    fn timed_wait_does_block_for_at_least_timeout<Sut: Event>() {
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut_listener = Sut::ListenerBuilder::new(&name)
            .config(&config)
            .create()
            .unwrap();
        let _sut_notifier = Sut::NotifierBuilder::new(&name)
            .config(&config)
            .open()
            .unwrap();

        let start = Instant::now();
        let result = sut_listener.timed_wait_one(TIMEOUT).unwrap();
        assert_that!(result, is_none);
        assert_that!(start.elapsed(), time_at_least TIMEOUT);
    }

    #[test]
    fn blocking_wait_blocks_until_notification_arrives<Sut: Event>() {
        let _watchdog = Watchdog::new();
        let name = generate_name();
        let config = Mutex::new(generate_isolated_config::<Sut>());

        let counter = AtomicU64::new(0);
        let handle = BarrierHandle::new();
        let barrier = BarrierBuilder::new(2).create(&handle).unwrap();

        std::thread::scope(|s| {
            let t = s.spawn(|| {
                let sut_listener = Sut::ListenerBuilder::new(&name)
                    .config(&config.lock().unwrap())
                    .create()
                    .unwrap();
                barrier.wait();
                let result = sut_listener.blocking_wait_one().unwrap();
                counter.store(1, Ordering::SeqCst);
                assert_that!(result, is_some);
                assert_that!(result.unwrap(), eq TriggerId::new(89));
            });

            barrier.wait();
            let sut_notifier = Sut::NotifierBuilder::new(&name)
                .config(&config.lock().unwrap())
                .open()
                .unwrap();
            std::thread::sleep(TIMEOUT);
            let counter_old = counter.load(Ordering::SeqCst);
            sut_notifier.notify(TriggerId::new(89)).unwrap();
            t.join().unwrap();

            assert_that!(counter_old, eq 0);
            assert_that!(counter.load(Ordering::SeqCst), eq 1);
        });
    }

    /// windows sporadically instantly wakes up in a timed receive operation
    #[cfg(not(target_os = "windows"))]
    #[test]
    fn timed_wait_blocks_until_notification_arrives<Sut: Event>() {
        let _watchdog = Watchdog::new();
        let name = generate_name();
        let config = Mutex::new(generate_isolated_config::<Sut>());

        let counter = AtomicU64::new(0);
        let handle = BarrierHandle::new();
        let barrier = BarrierBuilder::new(2).create(&handle).unwrap();

        std::thread::scope(|s| {
            let t = s.spawn(|| {
                let sut_listener = Sut::ListenerBuilder::new(&name)
                    .config(&config.lock().unwrap())
                    .create()
                    .unwrap();
                barrier.wait();
                let result = sut_listener.timed_wait_one(TIMEOUT * 1000).unwrap();
                counter.store(1, Ordering::SeqCst);
                assert_that!(result, is_some);
                assert_that!(result.unwrap(), eq TriggerId::new(82));
            });

            barrier.wait();
            let sut_notifier = Sut::NotifierBuilder::new(&name)
                .config(&config.lock().unwrap())
                .open()
                .unwrap();
            std::thread::sleep(TIMEOUT);
            let counter_old = counter.load(Ordering::SeqCst);
            sut_notifier.notify(TriggerId::new(82)).unwrap();
            t.join().unwrap();

            assert_that!(counter_old, eq 0);
            assert_that!(counter.load(Ordering::SeqCst), eq 1);
        });
    }

    #[test]
    fn list_events_works<Sut: Event>() {
        let mut sut_names = vec![];
        const LIMIT: usize = 10;
        let config = generate_isolated_config::<Sut>();

        assert_that!(<Sut as NamedConceptMgmt>::list_cfg(&config).unwrap(), len 0);
        for i in 0..LIMIT {
            sut_names.push(generate_name());
            assert_that!(<Sut as NamedConceptMgmt>::does_exist_cfg(&sut_names[i], &config), eq Ok(false));
            core::mem::forget(
                Sut::ListenerBuilder::new(&sut_names[i])
                    .config(&config)
                    .create(),
            );
            assert_that!(<Sut as NamedConceptMgmt>::does_exist_cfg(&sut_names[i], &config), eq Ok(true));

            let list = <Sut as NamedConceptMgmt>::list_cfg(&config).unwrap();
            assert_that!(list, len i + 1);
            let does_exist_in_list = |value| {
                for e in &list {
                    if e == value {
                        return true;
                    }
                }
                false
            };

            for name in &sut_names {
                assert_that!(does_exist_in_list(name), eq true);
            }
        }

        for i in 0..LIMIT {
            assert_that!(unsafe{<Sut as NamedConceptMgmt>::remove_cfg(&sut_names[i], &config)}, eq Ok(true));
            assert_that!(unsafe{<Sut as NamedConceptMgmt>::remove_cfg(&sut_names[i], &config)}, eq Ok(false));
        }

        assert_that!(<Sut as NamedConceptMgmt>::list_cfg(&config).unwrap(), len 0);
    }

    #[test]
    fn custom_suffix_keeps_events_separated<Sut: Event>() {
        let config = generate_isolated_config::<Sut>();
        let config_1 = config
            .clone()
            .suffix(unsafe { &FileName::new_unchecked(b".suffix_1") });
        let config_2 = config.suffix(unsafe { &FileName::new_unchecked(b".suffix_2") });

        let sut_name = generate_name();

        assert_that!(<Sut as NamedConceptMgmt>::does_exist_cfg(&sut_name, &config_1), eq Ok(false));
        assert_that!(<Sut as NamedConceptMgmt>::does_exist_cfg(&sut_name, &config_2), eq Ok(false));
        assert_that!(<Sut as NamedConceptMgmt>::list_cfg(&config_1).unwrap(), len 0);
        assert_that!(<Sut as NamedConceptMgmt>::list_cfg(&config_2).unwrap(), len 0);

        let sut_1 = Sut::ListenerBuilder::new(&sut_name)
            .config(&config_1)
            .create()
            .unwrap();

        assert_that!(<Sut as NamedConceptMgmt>::does_exist_cfg(&sut_name, &config_1), eq Ok(true));
        assert_that!(<Sut as NamedConceptMgmt>::does_exist_cfg(&sut_name, &config_2), eq Ok(false));
        assert_that!(<Sut as NamedConceptMgmt>::list_cfg(&config_1).unwrap(), len 1);
        assert_that!(<Sut as NamedConceptMgmt>::list_cfg(&config_2).unwrap(), len 0);

        let sut_2 = Sut::ListenerBuilder::new(&sut_name)
            .config(&config_2)
            .create()
            .unwrap();

        assert_that!(<Sut as NamedConceptMgmt>::does_exist_cfg(&sut_name, &config_1), eq Ok(true));
        assert_that!(<Sut as NamedConceptMgmt>::does_exist_cfg(&sut_name, &config_2), eq Ok(true));
        assert_that!(<Sut as NamedConceptMgmt>::list_cfg(&config_1).unwrap(), len 1);
        assert_that!(<Sut as NamedConceptMgmt>::list_cfg(&config_2).unwrap(), len 1);

        assert_that!(<Sut as NamedConceptMgmt>::list_cfg(&config_1).unwrap()[0], eq sut_name);
        assert_that!(<Sut as NamedConceptMgmt>::list_cfg(&config_2).unwrap()[0], eq sut_name);

        assert_that!(*sut_1.name(), eq sut_name);
        assert_that!(*sut_2.name(), eq sut_name);

        core::mem::forget(sut_1);
        core::mem::forget(sut_2);

        assert_that!(unsafe {<Sut as NamedConceptMgmt>::remove_cfg(&sut_name, &config_1)}, eq Ok(true));
        assert_that!(unsafe {<Sut as NamedConceptMgmt>::remove_cfg(&sut_name, &config_1)}, eq Ok(false));
        assert_that!(unsafe {<Sut as NamedConceptMgmt>::remove_cfg(&sut_name, &config_2)}, eq Ok(true));
        assert_that!(unsafe {<Sut as NamedConceptMgmt>::remove_cfg(&sut_name, &config_2)}, eq Ok(false));
    }

    #[test]
    fn defaults_for_configuration_are_set_correctly<Sut: Event>() {
        let config = <Sut as NamedConceptMgmt>::Configuration::default();
        assert_that!(*config.get_suffix(), eq Sut::default_suffix());
        assert_that!(*config.get_path_hint(), eq Sut::default_path_hint());
        assert_that!(*config.get_prefix(), eq Sut::default_prefix());
    }

    #[test]
    fn setting_trigger_id_limit_works<Sut: Event>() {
        test_requires!(Sut::has_trigger_id_limit());

        const TRIGGER_ID_MAX: TriggerId = TriggerId::new(1234);
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let _sut_listener = Sut::ListenerBuilder::new(&name)
            .trigger_id_max(TRIGGER_ID_MAX)
            .config(&config)
            .create()
            .unwrap();
        let sut_notifier = Sut::NotifierBuilder::new(&name)
            .config(&config)
            .open()
            .unwrap();

        if Sut::has_trigger_id_limit() {
            assert_that!(sut_notifier.trigger_id_max(), eq TRIGGER_ID_MAX);
        } else {
            assert_that!(sut_notifier.trigger_id_max(), eq TriggerId::new(usize::MAX));
        }
    }

    #[test]
    fn triggering_up_to_trigger_id_max_works<Sut: Event>() {
        test_requires!(Sut::has_trigger_id_limit());

        const TRIGGER_ID_MAX: TriggerId = TriggerId::new(1024);
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut_listener = Sut::ListenerBuilder::new(&name)
            .trigger_id_max(TRIGGER_ID_MAX)
            .config(&config)
            .create()
            .unwrap();
        let sut_notifier = Sut::NotifierBuilder::new(&name)
            .config(&config)
            .open()
            .unwrap();

        for i in 0..TRIGGER_ID_MAX.as_value() {
            assert_that!(sut_notifier.notify(TriggerId::new(i)), is_ok);
        }

        let result = sut_notifier.notify(TriggerId::new(TRIGGER_ID_MAX.as_value() + 1));
        assert_that!(result, is_err);
        assert_that!(
            result.err().unwrap(), eq
            NotifierNotifyError::TriggerIdOutOfBounds
        );

        let mut ids = HashSet::new();
        for _ in 0..TRIGGER_ID_MAX.as_value() {
            let event_id = sut_listener.try_wait_one().unwrap().unwrap();

            assert_that!(event_id, lt TRIGGER_ID_MAX);
            assert_that!(ids.insert(event_id), eq true);
        }

        let event_id = sut_listener.try_wait_one().unwrap();
        assert_that!(event_id, is_none);
    }

    fn wait_all_collects_all_triggers<Sut: Event, F: FnMut(&mut Vec<TriggerId>, &Sut::Listener)>(
        mut wait_call: F,
    ) {
        let _watchdog = Watchdog::new();
        const REPETITIONS: usize = 8;
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut_listener = Sut::ListenerBuilder::new(&name)
            .trigger_id_max(TriggerId::new(REPETITIONS))
            .config(&config)
            .create()
            .unwrap();
        let sut_notifier = Sut::NotifierBuilder::new(&name)
            .config(&config)
            .open()
            .unwrap();

        for i in 1..=REPETITIONS {
            for n in 0..i {
                sut_notifier.notify(TriggerId::new(n as _)).unwrap();
            }

            let mut vec_of_ids = vec![];
            wait_call(&mut vec_of_ids, &sut_listener);

            assert_that!(vec_of_ids, len { i });
            for n in 0..i {
                assert_that!(vec_of_ids, contains TriggerId::new(n));
            }
        }
    }

    #[test]
    fn try_wait_all_collects_all_triggers<Sut: Event>() {
        wait_all_collects_all_triggers::<Sut, _>(|v, sut: &Sut::Listener| {
            sut.try_wait_all(|id| v.push(id)).unwrap();
        });
    }

    #[test]
    fn timed_wait_all_collects_all_triggers<Sut: Event>() {
        wait_all_collects_all_triggers::<Sut, _>(|v, sut: &Sut::Listener| {
            sut.timed_wait_all(|id| v.push(id), TIMEOUT * 1000).unwrap();
        });
    }

    #[test]
    fn blocking_wait_all_collects_all_triggers<Sut: Event>() {
        wait_all_collects_all_triggers::<Sut, _>(|v, sut: &Sut::Listener| {
            sut.blocking_wait_all(|id| v.push(id)).unwrap();
        });
    }

    #[test]
    fn try_wait_all_does_not_block<Sut: Event>() {
        let _watchdog = Watchdog::new();
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut_listener = Sut::ListenerBuilder::new(&name)
            .config(&config)
            .create()
            .unwrap();

        let mut callback_called = false;
        sut_listener
            .try_wait_all(|_| callback_called = true)
            .unwrap();
        assert_that!(callback_called, eq false);
    }

    #[test]
    fn timed_wait_all_does_block_for_at_least_timeout<Sut: Event>() {
        let _watchdog = Watchdog::new();
        let name = generate_name();
        let config = generate_isolated_config::<Sut>();

        let sut_listener = Sut::ListenerBuilder::new(&name)
            .config(&config)
            .create()
            .unwrap();

        let mut callback_called = false;
        let now = Instant::now();
        sut_listener
            .timed_wait_all(|_| callback_called = true, TIMEOUT)
            .unwrap();
        assert_that!(callback_called, eq false);
        assert_that!(now.elapsed(), time_at_least TIMEOUT);
    }

    fn wait_all_wakes_up_on_notify<
        Sut: Event,
        F: FnMut(&mut Vec<TriggerId>, &Sut::Listener) + Send,
    >(
        mut wait_call: F,
    ) {
        let _watchdog = Watchdog::new();
        let name = generate_name();
        let barrier = Barrier::new(2);
        let counter = AtomicU64::new(0);
        let id = TriggerId::new(5);
        let config = Mutex::new(generate_isolated_config::<Sut>());

        std::thread::scope(|s| {
            let t1 = s.spawn(|| {
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
            });

            barrier.wait();
            let sut_notifier = Sut::NotifierBuilder::new(&name)
                .config(&config.lock().unwrap())
                .open()
                .unwrap();
            std::thread::sleep(TIMEOUT);
            assert_that!(counter.load(Ordering::Relaxed), eq 0);
            sut_notifier.notify(id).unwrap();
            t1.join().unwrap();
            assert_that!(counter.load(Ordering::Relaxed), eq 1);
        });
    }

    #[test]
    fn timed_wait_all_wakes_up_on_notify<Sut: Event>() {
        wait_all_wakes_up_on_notify::<Sut, _>(|v, sut: &Sut::Listener| {
            sut.timed_wait_all(|id| v.push(id), TIMEOUT * 1000).unwrap();
        });
    }

    #[test]
    fn blocking_wait_all_wakes_up_on_notify<Sut: Event>() {
        wait_all_wakes_up_on_notify::<Sut, _>(|v, sut: &Sut::Listener| {
            sut.blocking_wait_all(|id| v.push(id)).unwrap();
        });
    }

    #[test]
    fn out_of_scope_listener_shall_not_corrupt_notifier<Sut: Event>() {
        let name = generate_name();
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

        let result = sut_notifier.notify(TriggerId::new(0));

        // either present a disconnect error when available or continue sending without counterpart, for
        // instance when the event is network socket based
        if result.is_err() {
            assert_that!(result.err().unwrap(), eq NotifierNotifyError::Disconnected);
        }
    }

    #[instantiate_tests(<iceoryx2_cal::event::process_local_socketpair::EventImpl>)]
    mod process_local_socket_pair {}

    #[instantiate_tests(<iceoryx2_cal::event::unix_datagram_socket::EventImpl>)]
    mod unix_datagram {}

    #[instantiate_tests(<iceoryx2_cal::event::sem_bitset_process_local::Event>)]
    mod sem_bitset_process_local {}

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    #[instantiate_tests(<iceoryx2_cal::event::sem_bitset_posix_shared_memory::Event>)]
    mod sem_bitset_posix_shared_memory {}
}
