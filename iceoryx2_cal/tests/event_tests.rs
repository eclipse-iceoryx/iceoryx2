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
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{Duration, Instant};

    use iceoryx2_bb_container::semantic_string::*;
    use iceoryx2_bb_posix::barrier::{BarrierBuilder, BarrierHandle};
    use iceoryx2_bb_posix::unique_system_id::UniqueSystemId;
    use iceoryx2_bb_system_types::file_name::FileName;
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_cal::event::*;
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
    fn create_works<Sut: Event<u64>>() {
        let name = generate_name();

        let sut_listener = Sut::ListenerBuilder::new(&name).create().unwrap();
        let sut_notifier = Sut::NotifierBuilder::new(&name).open().unwrap();

        assert_that!(*sut_listener.name(), eq name);
        assert_that!(*sut_notifier.name(), eq name);
    }

    #[test]
    fn listener_cleans_up_when_out_of_scope<Sut: Event<u64>>() {
        let name = generate_name();

        assert_that!(Sut::does_exist(&name).unwrap(), eq false);
        let sut_listener = Sut::ListenerBuilder::new(&name).create().unwrap();
        assert_that!(Sut::does_exist(&name).unwrap(), eq true);

        drop(sut_listener);
        assert_that!(Sut::does_exist(&name).unwrap(), eq false);
    }

    #[test]
    fn cannot_be_created_twice<Sut: Event<u64>>() {
        let name = generate_name();

        let _sut = Sut::ListenerBuilder::new(&name).create().unwrap();
        let sut = Sut::ListenerBuilder::new(&name).create();

        assert_that!(sut, is_err);
        assert_that!(sut.err().unwrap(), eq ListenerCreateError::AlreadyExists);
    }

    #[test]
    fn cannot_open_non_existing<Sut: Event<u64>>() {
        let name = generate_name();

        let sut = Sut::NotifierBuilder::new(&name).open();

        assert_that!(sut, is_err);
        assert_that!(sut.err().unwrap(), eq NotifierCreateError::DoesNotExist);
    }

    fn sending_notification_works<
        Sut: Event<u64>,
        F: Fn(&Sut::Listener) -> Result<Option<u64>, ListenerWaitError>,
    >(
        wait_call: F,
    ) {
        const REPETITIONS: u64 = 32;
        let name = generate_name();

        let sut_listener = Sut::ListenerBuilder::new(&name).create().unwrap();
        let sut_notifier = Sut::NotifierBuilder::new(&name).open().unwrap();

        for i in 0..REPETITIONS {
            sut_notifier.notify(i).unwrap();
            let result = wait_call(&sut_listener).unwrap();
            assert_that!(result, eq Some(i));
        }
    }

    #[test]
    fn sending_notification_and_try_wait_works<Sut: Event<u64>>() {
        sending_notification_works::<Sut, _>(|sut| sut.try_wait());
    }

    #[test]
    fn sending_notification_and_timed_wait_works<Sut: Event<u64>>() {
        sending_notification_works::<Sut, _>(|sut| sut.timed_wait(TIMEOUT));
    }

    #[test]
    fn sending_notification_and_blocking_wait_works<Sut: Event<u64>>() {
        sending_notification_works::<Sut, _>(|sut| sut.blocking_wait());
    }

    fn sending_multiple_notifications_before_wait_works<
        Sut: Event<u64>,
        F: Fn(&Sut::Listener) -> Result<Option<u64>, ListenerWaitError>,
    >(
        wait_call: F,
    ) {
        const REPETITIONS: u64 = 8;
        let name = generate_name();

        let sut_listener = Sut::ListenerBuilder::new(&name).create().unwrap();
        let sut_notifier = Sut::NotifierBuilder::new(&name).open().unwrap();

        for i in 0..REPETITIONS {
            sut_notifier.notify(i).unwrap();
        }

        for i in 0..REPETITIONS {
            let result = wait_call(&sut_listener).unwrap();
            assert_that!(result, eq Some(i));
        }
    }

    #[test]
    fn sending_multiple_notifications_before_try_wait_works<Sut: Event<u64>>() {
        sending_multiple_notifications_before_wait_works::<Sut, _>(|sut| sut.try_wait());
    }

    #[test]
    fn sending_multiple_notifications_before_timed_wait_works<Sut: Event<u64>>() {
        sending_multiple_notifications_before_wait_works::<Sut, _>(|sut| sut.timed_wait(TIMEOUT));
    }

    #[test]
    fn sending_multiple_notifications_before_blocking_wait_works<Sut: Event<u64>>() {
        sending_multiple_notifications_before_wait_works::<Sut, _>(|sut| sut.blocking_wait());
    }

    fn sending_multiple_notifications_from_multiple_sources_before_wait_works<
        Sut: Event<u64>,
        F: Fn(&Sut::Listener) -> Result<Option<u64>, ListenerWaitError>,
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

        for i in 0..REPETITIONS {
            for notifier in &sources {
                notifier.notify(i).unwrap();
            }
        }

        for i in 0..REPETITIONS {
            for _ in 0..SOURCES {
                let result = wait_call(&sut_listener).unwrap();
                assert_that!(result, eq Some(i));
            }
        }
    }

    #[test]
    fn sending_multiple_notifications_from_multiple_sources_before_try_wait_works<
        Sut: Event<u64>,
    >() {
        sending_multiple_notifications_from_multiple_sources_before_wait_works::<Sut, _>(|sut| {
            sut.try_wait()
        });
    }

    #[test]
    fn sending_multiple_notifications_from_multiple_sources_before_timed_wait_works<
        Sut: Event<u64>,
    >() {
        sending_multiple_notifications_from_multiple_sources_before_wait_works::<Sut, _>(|sut| {
            sut.timed_wait(TIMEOUT)
        });
    }

    #[test]
    fn sending_multiple_notifications_from_multiple_sources_before_blocking_wait_works<
        Sut: Event<u64>,
    >() {
        sending_multiple_notifications_from_multiple_sources_before_wait_works::<Sut, _>(|sut| {
            sut.blocking_wait()
        });
    }

    #[test]
    fn try_wait_does_not_block<Sut: Event<u64>>() {
        let name = generate_name();

        let sut_listener = Sut::ListenerBuilder::new(&name).create().unwrap();
        let _sut_notifier = Sut::NotifierBuilder::new(&name).open().unwrap();

        let result = sut_listener.try_wait().unwrap();
        assert_that!(result, is_none);
    }

    #[test]
    fn timed_wait_does_block_for_at_least_timeout<Sut: Event<u64>>() {
        let name = generate_name();

        let sut_listener = Sut::ListenerBuilder::new(&name).create().unwrap();
        let _sut_notifier = Sut::NotifierBuilder::new(&name).open().unwrap();

        let start = Instant::now();
        let result = sut_listener.timed_wait(TIMEOUT).unwrap();
        assert_that!(result, is_none);
        assert_that!(start.elapsed(), time_at_least TIMEOUT);
    }

    #[test]
    fn blocking_wait_blocks_until_notification_arrives<Sut: Event<u64>>() {
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
                assert_that!(result.unwrap(), eq 8912);
            });

            barrier.wait();
            let sut_notifier = Sut::NotifierBuilder::new(&name).open().unwrap();
            std::thread::sleep(TIMEOUT);
            let counter_old = counter.load(Ordering::SeqCst);
            sut_notifier.notify(8912).unwrap();
            t.join().unwrap();

            assert_that!(counter_old, eq 0);
            assert_that!(counter.load(Ordering::SeqCst), eq 1);
        });
    }

    /// windows sporadically instantly wakes up in a timed receive operation
    #[cfg(not(target_os = "windows"))]
    #[test]
    fn timed_wait_blocks_until_notification_arrives<Sut: Event<u64>>() {
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
                assert_that!(result.unwrap(), eq 8912);
            });

            barrier.wait();
            let sut_notifier = Sut::NotifierBuilder::new(&name).open().unwrap();
            std::thread::sleep(TIMEOUT);
            let counter_old = counter.load(Ordering::SeqCst);
            sut_notifier.notify(8912).unwrap();
            t.join().unwrap();

            assert_that!(counter_old, eq 0);
            assert_that!(counter.load(Ordering::SeqCst), eq 1);
        });
    }

    #[test]
    fn list_events_works<Sut: Event<u64>>() {
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
    fn custom_suffix_keeps_events_separated<Sut: Event<u64>>() {
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
    fn defaults_for_configuration_are_set_correctly<Sut: Event<u64>>() {
        let config = <Sut as NamedConceptMgmt>::Configuration::default();
        assert_that!(*config.get_suffix(), eq DEFAULT_SUFFIX);
        assert_that!(*config.get_path_hint(), eq DEFAULT_PATH_HINT);
    }

    #[instantiate_tests(<iceoryx2_cal::event::unix_datagram_socket::Event<u64>>)]
    mod unix_datagram {}

    #[instantiate_tests(<iceoryx2_cal::event::process_local::Event<u64>>)]
    mod process_local {}
}
