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
    use std::collections::HashSet;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{Duration, Instant};

    use iceoryx2_bb_container::semantic_string::*;
    use iceoryx2_bb_posix::barrier::*;
    use iceoryx2_bb_posix::unique_system_id::UniqueSystemId;
    use iceoryx2_bb_system_types::file_name::FileName;
    use iceoryx2_bb_testing::watchdog::Watchdog;
    use iceoryx2_bb_testing::{assert_that, test_requires};
    use iceoryx2_cal::event::{TriggerId, *};
    use iceoryx2_cal::named_concept::*;

    const TIMEOUT: Duration = Duration::from_millis(25);

    fn generate_name() -> FileName {
        let mut file = FileName::new(b"event_tests_").unwrap();
        file.push_bytes(
            UniqueSystemId::new()
                .unwrap()
                .value()
                .to_string()
                .as_bytes(),
        )
        .unwrap();
        file
    }

    #[test]
    fn create_works<Sut: Event>() {
        let name = generate_name();

        let sut_listener = Sut::ListenerBuilder::new(&name).create().unwrap();
        let sut_notifier = Sut::NotifierBuilder::new(&name).open().unwrap();

        assert_that!(*sut_listener.name(), eq name);
        assert_that!(*sut_notifier.name(), eq name);
    }

    #[test]
    fn listener_cleans_up_when_out_of_scope<Sut: Event>() {
        let name = generate_name();

        assert_that!(Sut::does_exist(&name).unwrap(), eq false);
        let sut_listener = Sut::ListenerBuilder::new(&name).create().unwrap();
        assert_that!(Sut::does_exist(&name).unwrap(), eq true);

        drop(sut_listener);
        assert_that!(Sut::does_exist(&name).unwrap(), eq false);
    }

    #[test]
    fn cannot_be_created_twice<Sut: Event>() {
        let name = generate_name();

        let _sut = Sut::ListenerBuilder::new(&name).create().unwrap();
        let sut = Sut::ListenerBuilder::new(&name).create();

        assert_that!(sut, is_err);
        assert_that!(sut.err().unwrap(), eq ListenerCreateError::AlreadyExists);
    }

    #[test]
    fn cannot_open_non_existing<Sut: Event>() {
        let name = generate_name();

        let sut = Sut::NotifierBuilder::new(&name).open();

        assert_that!(sut, is_err);
        assert_that!(sut.err().unwrap(), eq NotifierCreateError::DoesNotExist);
    }

    #[test]
    fn notify_with_same_id_does_not_lead_to_non_blocking_timed_wait<Sut: Event>() {
        let _watchdog = Watchdog::new();
        const REPETITIONS: u64 = 8;
        let name = generate_name();

        let sut_listener = Sut::ListenerBuilder::new(&name).create().unwrap();
        let sut_notifier = Sut::NotifierBuilder::new(&name).open().unwrap();

        let trigger_id = TriggerId::new(0);

        for _ in 0..REPETITIONS {
            sut_notifier.notify(trigger_id).unwrap();
        }

        assert_that!(sut_listener.try_wait().unwrap(), is_some);

        let now = Instant::now();
        let result = sut_listener.timed_wait(TIMEOUT).unwrap();

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
        const REPETITIONS: u64 = 8;
        let name = generate_name();

        let sut_listener = Sut::ListenerBuilder::new(&name).create().unwrap();
        let sut_notifier = Sut::NotifierBuilder::new(&name).open().unwrap();

        for i in 0..REPETITIONS {
            sut_notifier.notify(TriggerId::new(i)).unwrap();
            let result = wait_call(&sut_listener).unwrap();
            assert_that!(result.unwrap(), eq TriggerId::new(i));
        }
    }

    #[test]
    fn sending_notification_and_try_wait_works<Sut: Event>() {
        sending_notification_works::<Sut, _>(|sut| sut.try_wait());
    }

    #[test]
    fn sending_notification_and_timed_wait_works<Sut: Event>() {
        sending_notification_works::<Sut, _>(|sut| sut.timed_wait(TIMEOUT));
    }

    #[test]
    fn sending_notification_and_blocking_wait_works<Sut: Event>() {
        sending_notification_works::<Sut, _>(|sut| sut.blocking_wait());
    }

    fn sending_multiple_notifications_before_wait_works<
        Sut: Event,
        F: Fn(&Sut::Listener) -> Result<Option<TriggerId>, ListenerWaitError>,
    >(
        wait_call: F,
    ) {
        const REPETITIONS: u64 = 8;
        let name = generate_name();

        let sut_listener = Sut::ListenerBuilder::new(&name).create().unwrap();
        let sut_notifier = Sut::NotifierBuilder::new(&name).open().unwrap();

        for i in 0..REPETITIONS {
            sut_notifier.notify(TriggerId::new(i)).unwrap();
        }

        let mut ids = HashSet::new();
        for _ in 0..REPETITIONS {
            let result = wait_call(&sut_listener).unwrap();
            assert_that!(result, is_some);
            let result = result.unwrap();
            assert_that!(result.as_u64(), lt REPETITIONS);
            assert_that!(ids.insert(result), eq true);
        }
    }

    #[test]
    fn sending_multiple_notifications_before_try_wait_works<Sut: Event>() {
        sending_multiple_notifications_before_wait_works::<Sut, _>(|sut| sut.try_wait());
    }

    #[test]
    fn sending_multiple_notifications_before_timed_wait_works<Sut: Event>() {
        sending_multiple_notifications_before_wait_works::<Sut, _>(|sut| sut.timed_wait(TIMEOUT));
    }

    #[test]
    fn sending_multiple_notifications_before_blocking_wait_works<Sut: Event>() {
        sending_multiple_notifications_before_wait_works::<Sut, _>(|sut| sut.blocking_wait());
    }

    fn sending_multiple_notifications_from_multiple_sources_before_wait_works<
        Sut: Event,
        F: Fn(&Sut::Listener) -> Result<Option<TriggerId>, ListenerWaitError>,
    >(
        wait_call: F,
    ) {
        const REPETITIONS: u64 = 2;
        const SOURCES: u64 = 4;
        let name = generate_name();
        let mut sources = vec![];

        let sut_listener = Sut::ListenerBuilder::new(&name).create().unwrap();
        for _ in 0..SOURCES {
            sources.push(Sut::NotifierBuilder::new(&name).open().unwrap());
        }

        let mut event_ids = vec![];
        for i in 0..REPETITIONS {
            for (n, notifier) in sources.iter().enumerate() {
                let event_id = n as u64 * (SOURCES + REPETITIONS + 1) + i;
                assert_that!(notifier.notify(TriggerId::new(event_id)), is_ok);
                event_ids.push(event_id);
            }
        }

        for _ in 0..REPETITIONS {
            for _ in 0..SOURCES {
                let result = wait_call(&sut_listener).unwrap();
                assert_that!(result, is_some);
                assert_that!(event_ids, contains result.unwrap().as_u64());
            }
        }
    }

    #[test]
    fn sending_multiple_notifications_from_multiple_sources_before_try_wait_works<Sut: Event>() {
        sending_multiple_notifications_from_multiple_sources_before_wait_works::<Sut, _>(|sut| {
            sut.try_wait()
        });
    }

    #[test]
    fn sending_multiple_notifications_from_multiple_sources_before_timed_wait_works<Sut: Event>() {
        sending_multiple_notifications_from_multiple_sources_before_wait_works::<Sut, _>(|sut| {
            sut.timed_wait(TIMEOUT)
        });
    }

    #[test]
    fn sending_multiple_notifications_from_multiple_sources_before_blocking_wait_works<
        Sut: Event,
    >() {
        sending_multiple_notifications_from_multiple_sources_before_wait_works::<Sut, _>(|sut| {
            sut.blocking_wait()
        });
    }

    #[test]
    fn try_wait_does_not_block<Sut: Event>() {
        let name = generate_name();

        let sut_listener = Sut::ListenerBuilder::new(&name).create().unwrap();
        let _sut_notifier = Sut::NotifierBuilder::new(&name).open().unwrap();

        let result = sut_listener.try_wait().unwrap();
        assert_that!(result, is_none);
    }

    #[test]
    fn timed_wait_does_block_for_at_least_timeout<Sut: Event>() {
        let name = generate_name();

        let sut_listener = Sut::ListenerBuilder::new(&name).create().unwrap();
        let _sut_notifier = Sut::NotifierBuilder::new(&name).open().unwrap();

        let start = Instant::now();
        let result = sut_listener.timed_wait(TIMEOUT).unwrap();
        assert_that!(result, is_none);
        assert_that!(start.elapsed(), time_at_least TIMEOUT);
    }

    #[test]
    fn blocking_wait_blocks_until_notification_arrives<Sut: Event>() {
        let _watchdog = Watchdog::new();
        let name = generate_name();

        let counter = AtomicU64::new(0);
        let handle = BarrierHandle::new();
        let barrier = BarrierBuilder::new(2).create(&handle).unwrap();

        std::thread::scope(|s| {
            let t = s.spawn(|| {
                let sut_listener = Sut::ListenerBuilder::new(&name).create().unwrap();
                barrier.wait();
                let result = sut_listener.blocking_wait().unwrap();
                counter.store(1, Ordering::SeqCst);
                assert_that!(result, is_some);
                assert_that!(result.unwrap(), eq TriggerId::new(89));
            });

            barrier.wait();
            let sut_notifier = Sut::NotifierBuilder::new(&name).open().unwrap();
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

        let counter = AtomicU64::new(0);
        let handle = BarrierHandle::new();
        let barrier = BarrierBuilder::new(2).create(&handle).unwrap();

        std::thread::scope(|s| {
            let t = s.spawn(|| {
                let sut_listener = Sut::ListenerBuilder::new(&name).create().unwrap();
                barrier.wait();
                let result = sut_listener.timed_wait(TIMEOUT * 1000).unwrap();
                counter.store(1, Ordering::SeqCst);
                assert_that!(result, is_some);
                assert_that!(result.unwrap(), eq TriggerId::new(82));
            });

            barrier.wait();
            let sut_notifier = Sut::NotifierBuilder::new(&name).open().unwrap();
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

        assert_that!(<Sut as NamedConceptMgmt>::list().unwrap(), len 0);
        for i in 0..LIMIT {
            sut_names.push(generate_name());
            assert_that!(<Sut as NamedConceptMgmt>::does_exist(&sut_names[i]), eq Ok(false));
            std::mem::forget(Sut::ListenerBuilder::new(&sut_names[i]).create());
            assert_that!(<Sut as NamedConceptMgmt>::does_exist(&sut_names[i]), eq Ok(true));

            let list = <Sut as NamedConceptMgmt>::list().unwrap();
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
            assert_that!(unsafe{<Sut as NamedConceptMgmt>::remove(&sut_names[i])}, eq Ok(true));
            assert_that!(unsafe{<Sut as NamedConceptMgmt>::remove(&sut_names[i])}, eq Ok(false));
        }

        assert_that!(<Sut as NamedConceptMgmt>::list().unwrap(), len 0);
    }

    #[test]
    fn custom_suffix_keeps_events_separated<Sut: Event>() {
        let config_1 = <Sut as NamedConceptMgmt>::Configuration::default()
            .suffix(unsafe { FileName::new_unchecked(b".suffix_1") });
        let config_2 = <Sut as NamedConceptMgmt>::Configuration::default()
            .suffix(unsafe { FileName::new_unchecked(b".suffix_2") });

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

        std::mem::forget(sut_1);
        std::mem::forget(sut_2);

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

        let _sut_listener = Sut::ListenerBuilder::new(&name)
            .trigger_id_max(TRIGGER_ID_MAX)
            .create()
            .unwrap();
        let sut_notifier = Sut::NotifierBuilder::new(&name).open().unwrap();

        if Sut::has_trigger_id_limit() {
            assert_that!(sut_notifier.trigger_id_max(), eq TRIGGER_ID_MAX);
        } else {
            assert_that!(sut_notifier.trigger_id_max(), eq TriggerId::new(u64::MAX));
        }
    }

    #[test]
    fn triggering_up_to_trigger_id_max_works<Sut: Event>() {
        test_requires!(Sut::has_trigger_id_limit());

        const TRIGGER_ID_MAX: TriggerId = TriggerId::new(1024);
        let name = generate_name();

        let sut_listener = Sut::ListenerBuilder::new(&name)
            .trigger_id_max(TRIGGER_ID_MAX)
            .create()
            .unwrap();
        let sut_notifier = Sut::NotifierBuilder::new(&name).open().unwrap();

        for i in 0..TRIGGER_ID_MAX.as_u64() {
            assert_that!(sut_notifier.notify(TriggerId::new(i)), is_ok);
        }

        let result = sut_notifier.notify(TriggerId::new(TRIGGER_ID_MAX.as_u64() + 1));
        assert_that!(result, is_err);
        assert_that!(
            result.err().unwrap(), eq
            NotifierNotifyError::TriggerIdOutOfBounds
        );

        let mut ids = HashSet::new();
        for _ in 0..TRIGGER_ID_MAX.as_u64() {
            let event_id = sut_listener.try_wait().unwrap().unwrap();

            assert_that!(event_id, lt TRIGGER_ID_MAX);
            assert_that!(ids.insert(event_id), eq true);
        }

        let event_id = sut_listener.try_wait().unwrap();
        assert_that!(event_id, is_none);
    }

    #[instantiate_tests(<iceoryx2_cal::event::unix_datagram_socket::EventImpl>)]
    mod unix_datagram {}

    #[instantiate_tests(<iceoryx2_cal::event::process_local::EventImpl>)]
    mod process_local {}

    #[instantiate_tests(<iceoryx2_cal::event::sem_bitset_process_local::Event>)]
    mod sem_bitset_process_local {}

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    #[instantiate_tests(<iceoryx2_cal::event::sem_bitset_posix_shared_memory::Event>)]
    mod sem_bitset_posix_shared_memory {}
}
