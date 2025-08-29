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
mod reactor {
    use iceoryx2_bb_posix::file_descriptor::FileDescriptorBased;
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_cal::event::unix_datagram_socket::*;
    use iceoryx2_cal::event::{Listener, ListenerBuilder, Notifier, NotifierBuilder};
    use iceoryx2_cal::reactor::{Reactor, *};
    use iceoryx2_cal::testing::{generate_isolated_config, generate_name};

    use core::sync::atomic::{AtomicU64, Ordering};
    use core::time::Duration;
    use std::sync::{Barrier, Mutex};
    use std::time::Instant;

    const TIMEOUT: Duration = Duration::from_millis(50);
    const INFINITE_TIMEOUT: Duration = Duration::from_secs(3600 * 24);
    const NUMBER_OF_ATTACHMENTS: usize = 32;

    struct NotifierListenerPair {
        notifier: unix_datagram_socket::Notifier,
        listener: unix_datagram_socket::Listener,
    }

    impl NotifierListenerPair {
        fn new() -> Self {
            let name = generate_name();
            let config = generate_isolated_config::<unix_datagram_socket::EventImpl>();
            let listener = unix_datagram_socket::ListenerBuilder::new(&name)
                .config(&config)
                .create()
                .unwrap();
            let notifier = unix_datagram_socket::NotifierBuilder::new(&name)
                .config(&config)
                .open()
                .unwrap();

            Self { listener, notifier }
        }
    }

    #[test]
    fn attach_and_detach_works<Sut: Reactor>() {
        let sut = <<Sut as Reactor>::Builder>::new().create().unwrap();
        let config = generate_isolated_config::<unix_datagram_socket::EventImpl>();

        let mut listeners = vec![];
        let mut guards = vec![];
        for _ in 0..NUMBER_OF_ATTACHMENTS {
            let name = generate_name();
            listeners.push(
                unix_datagram_socket::ListenerBuilder::new(&name)
                    .config(&config)
                    .create()
                    .unwrap(),
            );
        }

        assert_that!(sut.is_empty(), eq true);
        for i in 0..NUMBER_OF_ATTACHMENTS {
            assert_that!(sut.len(), eq i);
            guards.push(sut.attach(&listeners[i]));
            assert_that!(sut.is_empty(), eq false);
        }

        for i in 0..NUMBER_OF_ATTACHMENTS {
            assert_that!(sut.len(), eq NUMBER_OF_ATTACHMENTS - i);
            assert_that!(sut.is_empty(), eq false);
            guards.pop();
        }
        assert_that!(sut.is_empty(), eq true);
        assert_that!(sut.len(), eq 0);
    }

    #[test]
    fn attach_the_same_attachment_twice_fails<Sut: Reactor>() {
        let sut = <<Sut as Reactor>::Builder>::new().create().unwrap();
        let config = generate_isolated_config::<unix_datagram_socket::EventImpl>();

        let name = generate_name();
        let listener = unix_datagram_socket::ListenerBuilder::new(&name)
            .config(&config)
            .create()
            .unwrap();

        let _guard = sut.attach(&listener).unwrap();
        let result = sut.attach(&listener);

        assert_that!(result.err(), eq Some(ReactorAttachError::AlreadyAttached));
    }

    #[test]
    fn try_wait_does_not_block_when_triggered_single<Sut: Reactor>() {
        let sut = <<Sut as Reactor>::Builder>::new().create().unwrap();

        let attachment = NotifierListenerPair::new();
        attachment.notifier.notify(TriggerId::new(123)).unwrap();

        let _guard = sut.attach(&attachment.listener);

        let mut triggered_fds = vec![];
        assert_that!(
            sut.try_wait(|fd| triggered_fds.push(unsafe { fd.native_handle() })),
            eq Ok(1)
        );

        assert_that!(triggered_fds, len 1);
        assert_that!(triggered_fds[0], eq unsafe { attachment. listener.file_descriptor().native_handle() });
    }

    #[test]
    fn timed_wait_does_not_block_when_triggered_single<Sut: Reactor>() {
        let sut = <<Sut as Reactor>::Builder>::new().create().unwrap();

        let attachment = NotifierListenerPair::new();
        attachment.notifier.notify(TriggerId::new(123)).unwrap();

        let _guard = sut.attach(&attachment.listener);

        let mut triggered_fds = vec![];
        assert_that!(
            sut.timed_wait(
                |fd| triggered_fds.push(unsafe { fd.native_handle() }),
                INFINITE_TIMEOUT
            ),
            eq Ok(1)
        );

        assert_that!(triggered_fds, len 1);
        assert_that!(triggered_fds[0], eq unsafe { attachment. listener.file_descriptor().native_handle() });
    }

    #[test]
    fn blocking_wait_does_not_block_when_triggered_single<Sut: Reactor>() {
        let sut = <<Sut as Reactor>::Builder>::new().create().unwrap();

        let attachment = NotifierListenerPair::new();
        attachment.notifier.notify(TriggerId::new(123)).unwrap();

        let _guard = sut.attach(&attachment.listener);

        let mut triggered_fds = vec![];
        assert_that!(
            sut.blocking_wait(|fd| triggered_fds.push(unsafe { fd.native_handle() }),),
            eq Ok(1)
        );

        assert_that!(triggered_fds, len 1);
        assert_that!(triggered_fds[0], eq unsafe { attachment. listener.file_descriptor().native_handle() });
    }

    #[test]
    fn try_wait_activates_as_long_as_there_is_data_to_read<Sut: Reactor>() {
        let sut = <<Sut as Reactor>::Builder>::new().create().unwrap();

        let attachment = NotifierListenerPair::new();
        attachment.notifier.notify(TriggerId::new(123)).unwrap();

        let _guard = sut.attach(&attachment.listener);

        for _ in 0..4 {
            let mut triggered_fds = vec![];
            assert_that!(
                sut.try_wait(|fd| triggered_fds.push(unsafe { fd.native_handle() })),
                eq Ok(1)
            );

            assert_that!(triggered_fds, len 1);
            assert_that!(triggered_fds[0], eq unsafe { attachment. listener.file_descriptor().native_handle() });
        }

        attachment.listener.try_wait_one().unwrap();
        let mut triggered_fds = vec![];
        assert_that!(
            sut.try_wait(|fd| triggered_fds.push(unsafe { fd.native_handle() })),
            is_ok
        );

        assert_that!(triggered_fds, is_empty);
    }

    #[test]
    fn timed_wait_activates_as_long_as_there_is_data_to_read<Sut: Reactor>() {
        let sut = <<Sut as Reactor>::Builder>::new().create().unwrap();

        let attachment = NotifierListenerPair::new();
        attachment.notifier.notify(TriggerId::new(123)).unwrap();

        let _guard = sut.attach(&attachment.listener);

        for _ in 0..4 {
            let mut triggered_fds = vec![];
            assert_that!(
                sut.timed_wait(
                    |fd| triggered_fds.push(unsafe { fd.native_handle() }),
                    INFINITE_TIMEOUT
                ),
                eq Ok(1)
            );

            assert_that!(triggered_fds, len 1);
            assert_that!(triggered_fds[0], eq unsafe { attachment. listener.file_descriptor().native_handle() });
        }

        attachment.listener.try_wait_one().unwrap();
        let mut triggered_fds = vec![];
        assert_that!(
            sut.try_wait(|fd| triggered_fds.push(unsafe { fd.native_handle() })),
            is_ok
        );

        assert_that!(triggered_fds, is_empty);
    }

    #[test]
    fn blocking_wait_activates_as_long_as_there_is_data_to_read<Sut: Reactor>() {
        let sut = <<Sut as Reactor>::Builder>::new().create().unwrap();

        let attachment = NotifierListenerPair::new();
        attachment.notifier.notify(TriggerId::new(123)).unwrap();

        let _guard = sut.attach(&attachment.listener);

        for _ in 0..4 {
            let mut triggered_fds = vec![];
            assert_that!(
                sut.blocking_wait(|fd| triggered_fds.push(unsafe { fd.native_handle() }),),
                eq Ok(1)
            );

            assert_that!(triggered_fds, len 1);
            assert_that!(triggered_fds[0], eq unsafe { attachment. listener.file_descriptor().native_handle() });
        }

        attachment.listener.try_wait_one().unwrap();
        let mut triggered_fds = vec![];
        assert_that!(
            sut.try_wait(|fd| triggered_fds.push(unsafe { fd.native_handle() })),
            is_ok
        );

        assert_that!(triggered_fds, is_empty);
    }

    #[test]
    fn try_wait_does_not_block_when_triggered_many<Sut: Reactor>() {
        let sut = <<Sut as Reactor>::Builder>::new().create().unwrap();

        let mut attachments = vec![];
        for _ in 0..NUMBER_OF_ATTACHMENTS {
            let attachment = NotifierListenerPair::new();
            attachment.notifier.notify(TriggerId::new(123)).unwrap();
            attachments.push(attachment);
        }

        let mut guards = vec![];
        for i in 0..NUMBER_OF_ATTACHMENTS {
            guards.push(sut.attach(&attachments[i].listener).unwrap());
        }

        let mut triggered_fds = vec![];
        assert_that!(
            sut.try_wait(|fd| triggered_fds.push(unsafe { fd.native_handle() })),
            eq Ok(NUMBER_OF_ATTACHMENTS)
        );

        assert_that!(triggered_fds, len NUMBER_OF_ATTACHMENTS);
        for i in 0..NUMBER_OF_ATTACHMENTS {
            assert_that!(triggered_fds, contains unsafe { attachments[i].listener.file_descriptor().native_handle() } );
        }
    }

    #[test]
    fn timed_wait_does_not_block_when_triggered_many<Sut: Reactor>() {
        let sut = <<Sut as Reactor>::Builder>::new().create().unwrap();

        let mut attachments = vec![];
        for _ in 0..NUMBER_OF_ATTACHMENTS {
            let attachment = NotifierListenerPair::new();
            attachment.notifier.notify(TriggerId::new(123)).unwrap();
            attachments.push(attachment);
        }

        let mut guards = vec![];
        for i in 0..NUMBER_OF_ATTACHMENTS {
            guards.push(sut.attach(&attachments[i].listener).unwrap());
        }

        let mut triggered_fds = vec![];
        assert_that!(
            sut.timed_wait(
                |fd| triggered_fds.push(unsafe { fd.native_handle() }),
                INFINITE_TIMEOUT
            ),
            eq Ok(NUMBER_OF_ATTACHMENTS)
        );

        assert_that!(triggered_fds, len NUMBER_OF_ATTACHMENTS);
        for i in 0..NUMBER_OF_ATTACHMENTS {
            assert_that!(triggered_fds, contains unsafe { attachments[i].listener.file_descriptor().native_handle() } );
        }
    }

    #[test]
    fn blocking_wait_does_not_block_when_triggered_many<Sut: Reactor>() {
        let sut = <<Sut as Reactor>::Builder>::new().create().unwrap();

        let mut attachments = vec![];
        for _ in 0..NUMBER_OF_ATTACHMENTS {
            let attachment = NotifierListenerPair::new();
            attachment.notifier.notify(TriggerId::new(123)).unwrap();
            attachments.push(attachment);
        }

        let mut guards = vec![];
        for i in 0..NUMBER_OF_ATTACHMENTS {
            guards.push(sut.attach(&attachments[i].listener).unwrap());
        }

        let mut triggered_fds = vec![];
        assert_that!(
            sut.blocking_wait(|fd| triggered_fds.push(unsafe { fd.native_handle() })),
            eq Ok(NUMBER_OF_ATTACHMENTS)
        );

        assert_that!(triggered_fds, len NUMBER_OF_ATTACHMENTS);
        for i in 0..NUMBER_OF_ATTACHMENTS {
            assert_that!(triggered_fds, contains unsafe { attachments[i].listener.file_descriptor().native_handle() } );
        }
    }

    #[test]
    fn timed_wait_blocks_for_at_least_timeout<Sut: Reactor>() {
        let sut = <<Sut as Reactor>::Builder>::new().create().unwrap();

        let attachment = NotifierListenerPair::new();

        let _guard = sut.attach(&attachment.listener);

        let mut triggered_fds = vec![];
        let start = Instant::now();
        assert_that!(
            sut.timed_wait(
                |fd| triggered_fds.push(unsafe { fd.native_handle() }),
                TIMEOUT
            ),
            eq Ok(0)
        );
        assert_that!(start.elapsed(), time_at_least TIMEOUT);

        assert_that!(triggered_fds, len 0);
    }

    #[test]
    fn try_wait_triggers_until_all_data_is_consumed<Sut: Reactor>() {
        let sut = <<Sut as Reactor>::Builder>::new().create().unwrap();

        let mut attachments = vec![];
        for _ in 0..NUMBER_OF_ATTACHMENTS {
            let attachment = NotifierListenerPair::new();
            attachment.notifier.notify(TriggerId::new(123)).unwrap();
            attachments.push(attachment);
        }

        let mut guards = vec![];
        for i in 0..NUMBER_OF_ATTACHMENTS {
            guards.push(sut.attach(&attachments[i].listener).unwrap());
        }

        for n in 0..NUMBER_OF_ATTACHMENTS {
            let mut triggered_fds = vec![];
            assert_that!(
                sut.try_wait(|fd| triggered_fds.push(unsafe { fd.native_handle() })),
                is_ok
            );

            assert_that!(triggered_fds, len NUMBER_OF_ATTACHMENTS - n);
            for i in n..NUMBER_OF_ATTACHMENTS {
                assert_that!(triggered_fds, contains unsafe { attachments[i].listener.file_descriptor().native_handle() } );
            }

            attachments[n].listener.try_wait_one().unwrap();
        }

        let mut triggered_fds = vec![];
        assert_that!(
            sut.try_wait(|fd| triggered_fds.push(unsafe { fd.native_handle() })),
            is_ok
        );

        assert_that!(triggered_fds, len 0);
    }

    #[test]
    fn timed_wait_triggers_until_all_data_is_consumed<Sut: Reactor>() {
        let sut = <<Sut as Reactor>::Builder>::new().create().unwrap();

        let mut attachments = vec![];
        for _ in 0..NUMBER_OF_ATTACHMENTS {
            let attachment = NotifierListenerPair::new();
            attachment.notifier.notify(TriggerId::new(123)).unwrap();
            attachments.push(attachment);
        }

        let mut guards = vec![];
        for i in 0..NUMBER_OF_ATTACHMENTS {
            guards.push(sut.attach(&attachments[i].listener).unwrap());
        }

        for n in 0..NUMBER_OF_ATTACHMENTS {
            let mut triggered_fds = vec![];
            assert_that!(
                sut.timed_wait(
                    |fd| triggered_fds.push(unsafe { fd.native_handle() }),
                    INFINITE_TIMEOUT
                ),
                is_ok
            );

            assert_that!(triggered_fds, len NUMBER_OF_ATTACHMENTS - n);
            for i in n..NUMBER_OF_ATTACHMENTS {
                assert_that!(triggered_fds, contains unsafe { attachments[i].listener.file_descriptor().native_handle() } );
            }

            attachments[n].listener.try_wait_one().unwrap();
        }

        let mut triggered_fds = vec![];
        assert_that!(
            sut.try_wait(|fd| triggered_fds.push(unsafe { fd.native_handle() })),
            is_ok
        );

        assert_that!(triggered_fds, len 0);
    }

    #[test]
    fn blocking_wait_triggers_until_all_data_is_consumed<Sut: Reactor>() {
        let sut = <<Sut as Reactor>::Builder>::new().create().unwrap();

        let mut attachments = vec![];
        for _ in 0..NUMBER_OF_ATTACHMENTS {
            let attachment = NotifierListenerPair::new();
            attachment.notifier.notify(TriggerId::new(123)).unwrap();
            attachments.push(attachment);
        }

        let mut guards = vec![];
        for i in 0..NUMBER_OF_ATTACHMENTS {
            guards.push(sut.attach(&attachments[i].listener).unwrap());
        }

        for n in 0..NUMBER_OF_ATTACHMENTS {
            let mut triggered_fds = vec![];
            assert_that!(
                sut.blocking_wait(|fd| triggered_fds.push(unsafe { fd.native_handle() })),
                is_ok
            );

            assert_that!(triggered_fds, len NUMBER_OF_ATTACHMENTS - n);
            for i in n..NUMBER_OF_ATTACHMENTS {
                assert_that!(triggered_fds, contains unsafe { attachments[i].listener.file_descriptor().native_handle() } );
            }

            attachments[n].listener.try_wait_one().unwrap();
        }

        let mut triggered_fds = vec![];
        assert_that!(
            sut.try_wait(|fd| triggered_fds.push(unsafe { fd.native_handle() })),
            is_ok
        );

        assert_that!(triggered_fds, len 0);
    }

    #[test]
    fn timed_wait_blocks_until_triggered<Sut: Reactor>() {
        let name = generate_name();
        let barrier = Barrier::new(2);
        let counter = AtomicU64::new(0);
        let config = Mutex::new(generate_isolated_config::<unix_datagram_socket::EventImpl>());

        std::thread::scope(|s| {
            let t = s.spawn(|| {
                let sut = <<Sut as Reactor>::Builder>::new().create().unwrap();
                let listener = unix_datagram_socket::ListenerBuilder::new(&name)
                    .config(&config.lock().unwrap())
                    .create()
                    .unwrap();
                let _guard = sut.attach(&listener);
                barrier.wait();

                let mut triggered_fds = vec![];
                let timed_wait_result = sut.timed_wait(
                    |fd| triggered_fds.push(unsafe { fd.native_handle() }),
                    INFINITE_TIMEOUT,
                );

                counter.fetch_add(1, Ordering::Relaxed);

                assert_that!(triggered_fds, len 1);
                assert_that!(timed_wait_result, is_ok);
            });

            barrier.wait();
            std::thread::sleep(TIMEOUT);
            let counter_old = counter.load(Ordering::Relaxed);

            let notifier = unix_datagram_socket::NotifierBuilder::new(&name)
                .config(&config.lock().unwrap())
                .open()
                .unwrap();
            notifier.notify(TriggerId::new(123)).unwrap();
            t.join().unwrap();

            assert_that!(counter_old, eq 0);
        });
    }

    #[test]
    fn blocking_wait_blocks_until_triggered<Sut: Reactor>() {
        let name = generate_name();
        let barrier = Barrier::new(2);
        let counter = AtomicU64::new(0);
        let config = Mutex::new(generate_isolated_config::<unix_datagram_socket::EventImpl>());

        std::thread::scope(|s| {
            let t = s.spawn(|| {
                let sut = <<Sut as Reactor>::Builder>::new().create().unwrap();
                let listener = unix_datagram_socket::ListenerBuilder::new(&name)
                    .config(&config.lock().unwrap())
                    .create()
                    .unwrap();
                let _guard = sut.attach(&listener);
                barrier.wait();

                let mut triggered_fds = vec![];
                let blocking_wait_result =
                    sut.blocking_wait(|fd| triggered_fds.push(unsafe { fd.native_handle() }));

                counter.fetch_add(1, Ordering::Relaxed);

                assert_that!(triggered_fds, len 1);
                assert_that!(blocking_wait_result, is_ok);
            });

            barrier.wait();
            std::thread::sleep(TIMEOUT);
            let counter_old = counter.load(Ordering::Relaxed);

            let notifier = unix_datagram_socket::NotifierBuilder::new(&name)
                .config(&config.lock().unwrap())
                .open()
                .unwrap();
            notifier.notify(TriggerId::new(123)).unwrap();
            t.join().unwrap();

            assert_that!(counter_old, eq 0);
        });
    }

    #[instantiate_tests(<iceoryx2_cal::reactor::posix_select::Reactor>)]
    mod posix_select {}
}
