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

#[generic_tests::define]
mod waitset {
    use std::time::{Duration, Instant};

    use iceoryx2::port::listener::Listener;
    use iceoryx2::port::notifier::Notifier;
    use iceoryx2::port::waitset::{WaitSetAttachmentError, WaitSetRunError};
    use iceoryx2::prelude::{WaitSetBuilder, *};
    use iceoryx2::testing::*;
    use iceoryx2_bb_posix::config::test_directory;
    use iceoryx2_bb_posix::directory::Directory;
    use iceoryx2_bb_posix::file::Permission;
    use iceoryx2_bb_posix::unix_datagram_socket::{
        UnixDatagramReceiver, UnixDatagramSender, UnixDatagramSenderBuilder,
    };
    use iceoryx2_bb_posix::{
        file_descriptor_set::SynchronousMultiplexing, unique_system_id::UniqueSystemId,
        unix_datagram_socket::UnixDatagramReceiverBuilder,
    };
    use iceoryx2_bb_system_types::file_path::*;
    use iceoryx2_bb_system_types::path::*;
    use iceoryx2_bb_testing::watchdog::Watchdog;
    use iceoryx2_bb_testing::{assert_that, test_fail};
    use iceoryx2_cal::event::Event;

    const TIMEOUT: Duration = Duration::from_millis(100);

    fn generate_name() -> ServiceName {
        ServiceName::new(&format!(
            "waitset_tests_{}",
            UniqueSystemId::new().unwrap().value()
        ))
        .unwrap()
    }

    fn generate_uds_name() -> FilePath {
        let mut path = test_directory();
        Directory::create(&path, Permission::OWNER_ALL).unwrap();
        let _ = path.add_path_entry(
            &Path::new(
                &format!("waitset_tests_{}", UniqueSystemId::new().unwrap().value()).as_bytes(),
            )
            .unwrap(),
        );

        FilePath::new(path.as_bytes()).unwrap()
    }

    fn create_event<S: Service>(node: &Node<S>) -> (Listener<S>, Notifier<S>) {
        let service_name = generate_name();
        let service = node
            .service_builder(&service_name)
            .event()
            .open_or_create()
            .unwrap();
        (
            service.listener_builder().create().unwrap(),
            service.notifier_builder().create().unwrap(),
        )
    }

    fn create_socket() -> (UnixDatagramReceiver, UnixDatagramSender) {
        let uds_name = generate_uds_name();

        let receiver = UnixDatagramReceiverBuilder::new(&uds_name)
            .create()
            .unwrap();

        let sender = UnixDatagramSenderBuilder::new(&uds_name).create().unwrap();

        (receiver, sender)
    }

    #[test]
    fn calling_run_on_empty_waitset_fails<S: Service>() {
        let sut = WaitSetBuilder::new().create::<S>().unwrap();
        let result = sut.wait_and_process_once(|_| CallbackProgression::Continue);

        assert_that!(result.err(), eq Some(WaitSetRunError::NoAttachments));
    }

    #[test]
    fn attach_multiple_notifications_works<S: Service>()
    where
        <S::Event as Event>::Listener: SynchronousMultiplexing,
    {
        const LISTENER_LIMIT: usize = 16;
        const EXTERNAL_LIMIT: usize = 16;

        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<S>().unwrap();
        let sut = WaitSetBuilder::new().create::<S>().unwrap();
        let mut listeners = vec![];
        let mut sockets = vec![];
        let mut guards = vec![];

        for _ in 0..LISTENER_LIMIT {
            let (listener, _) = create_event::<S>(&node);
            listeners.push(listener);
        }

        for _ in 0..EXTERNAL_LIMIT {
            let (receiver, _) = create_socket();

            sockets.push(receiver);
        }

        assert_that!(sut.is_empty(), eq true);
        for (n, listener) in listeners.iter().enumerate() {
            assert_that!(sut.len(), eq n);
            guards.push(sut.attach_notification(listener).unwrap());
            assert_that!(sut.len(), eq n + 1);
            assert_that!(sut.is_empty(), eq false);
        }

        for (n, socket) in sockets.iter().enumerate() {
            assert_that!(sut.len(), eq n + listeners.len());
            guards.push(sut.attach_notification(socket).unwrap());
            assert_that!(sut.len(), eq n + 1 + listeners.len());
        }

        guards.clear();
        assert_that!(sut.is_empty(), eq true);
        assert_that!(sut.len(), eq 0);
    }

    #[test]
    fn attaching_same_notification_twice_fails<S: Service>()
    where
        <S::Event as Event>::Listener: SynchronousMultiplexing,
    {
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<S>().unwrap();
        let sut = WaitSetBuilder::new().create::<S>().unwrap();

        let (listener, _) = create_event::<S>(&node);
        let (receiver, _) = create_socket();

        let _guard = sut.attach_notification(&listener);
        assert_that!(sut.attach_notification(&listener).err(), eq Some(WaitSetAttachmentError::AlreadyAttached));

        let _guard = sut.attach_notification(&receiver);
        assert_that!(sut.attach_notification(&receiver).err(), eq Some(WaitSetAttachmentError::AlreadyAttached));
    }

    #[test]
    fn attaching_same_deadline_twice_fails<S: Service>()
    where
        <S::Event as Event>::Listener: SynchronousMultiplexing,
    {
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<S>().unwrap();
        let sut = WaitSetBuilder::new().create::<S>().unwrap();

        let (listener, _) = create_event::<S>(&node);
        let (receiver, _) = create_socket();

        let _guard = sut.attach_deadline(&listener, TIMEOUT);
        assert_that!(sut.attach_deadline(&listener, TIMEOUT).err(), eq Some(WaitSetAttachmentError::AlreadyAttached));

        let _guard = sut.attach_deadline(&receiver, TIMEOUT);
        assert_that!(sut.attach_deadline(&receiver, TIMEOUT).err(), eq Some(WaitSetAttachmentError::AlreadyAttached));
    }

    #[test]
    fn run_lists_all_notifications<S: Service>()
    where
        <S::Event as Event>::Listener: SynchronousMultiplexing,
    {
        set_log_level(LogLevel::Debug);
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<S>().unwrap();
        let sut = WaitSetBuilder::new().create::<S>().unwrap();

        let (listener_1, notifier_1) = create_event::<S>(&node);
        let (listener_2, _notifier_2) = create_event::<S>(&node);
        let (receiver_1, sender_1) = create_socket();
        let (receiver_2, _sender_2) = create_socket();

        let listener_1_guard = sut.attach_notification(&listener_1).unwrap();
        let listener_2_guard = sut.attach_notification(&listener_2).unwrap();
        let receiver_1_guard = sut.attach_notification(&receiver_1).unwrap();
        let receiver_2_guard = sut.attach_notification(&receiver_2).unwrap();

        notifier_1.notify().unwrap();
        sender_1.try_send(b"bla").unwrap();

        let mut listener_1_triggered = false;
        let mut listener_2_triggered = false;
        let mut receiver_1_triggered = false;
        let mut receiver_2_triggered = false;

        sut.wait_and_process_once(|attachment_id| {
            if attachment_id.has_event_from(&listener_1_guard) {
                listener_1_triggered = true;
            } else if attachment_id.has_event_from(&listener_2_guard) {
                listener_2_triggered = true;
            } else if attachment_id.has_event_from(&receiver_1_guard) {
                receiver_1_triggered = true;
            } else if attachment_id.has_event_from(&receiver_2_guard) {
                receiver_2_triggered = true;
            } else {
                test_fail!("only attachments shall trigger");
            }

            CallbackProgression::Continue
        })
        .unwrap();

        assert_that!(listener_1_triggered, eq true);
        assert_that!(receiver_1_triggered, eq true);
    }

    #[test]
    fn run_with_tick_interval_blocks_for_at_least_timeout<S: Service>()
    where
        <S::Event as Event>::Listener: SynchronousMultiplexing,
    {
        let _watchdog = Watchdog::new();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<S>().unwrap();
        let sut = WaitSetBuilder::new().create::<S>().unwrap();

        let (listener, _) = create_event::<S>(&node);
        let _guard = sut.attach_notification(&listener);
        let tick_guard = sut.attach_interval(TIMEOUT).unwrap();

        let mut callback_called = false;
        let start = Instant::now();
        sut.wait_and_process_once(|id| {
            callback_called = true;
            assert_that!(id.has_event_from(&tick_guard), eq true);
            assert_that!(id.has_missed_deadline(&tick_guard), eq false);
            CallbackProgression::Continue
        })
        .unwrap();

        assert_that!(callback_called, eq true);
        assert_that!(start.elapsed(), time_at_least TIMEOUT);
    }

    #[test]
    fn run_with_deadline_blocks_for_at_least_timeout<S: Service>()
    where
        <S::Event as Event>::Listener: SynchronousMultiplexing,
    {
        let _watchdog = Watchdog::new();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<S>().unwrap();
        let sut = WaitSetBuilder::new().create::<S>().unwrap();

        let (listener, _) = create_event::<S>(&node);
        let guard = sut.attach_deadline(&listener, TIMEOUT).unwrap();

        let start = Instant::now();
        sut.wait_and_process_once(|id| {
            assert_that!(id.has_missed_deadline(&guard), eq true);
            CallbackProgression::Continue
        })
        .unwrap();

        assert_that!(start.elapsed(), time_at_least TIMEOUT);
    }

    #[test]
    fn run_lists_all_deadlines<S: Service>()
    where
        <S::Event as Event>::Listener: SynchronousMultiplexing,
    {
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<S>().unwrap();
        let sut = WaitSetBuilder::new().create::<S>().unwrap();

        let (listener_1, notifier_1) = create_event::<S>(&node);
        let (listener_2, _notifier_2) = create_event::<S>(&node);
        let (receiver_1, sender_1) = create_socket();
        let (receiver_2, _sender_2) = create_socket();

        let listener_1_guard = sut.attach_deadline(&listener_1, TIMEOUT * 1000).unwrap();
        let listener_2_guard = sut
            .attach_deadline(&listener_2, Duration::from_nanos(1))
            .unwrap();
        let receiver_1_guard = sut.attach_deadline(&receiver_1, TIMEOUT * 1000).unwrap();
        let receiver_2_guard = sut
            .attach_deadline(&receiver_2, Duration::from_nanos(1))
            .unwrap();

        std::thread::sleep(TIMEOUT);

        notifier_1.notify().unwrap();
        sender_1.try_send(b"bla").unwrap();

        let mut listener_1_triggered = false;
        let mut listener_2_triggered = false;
        let mut receiver_1_triggered = false;
        let mut receiver_2_triggered = false;

        sut.wait_and_process_once(|attachment_id| {
            if attachment_id.has_event_from(&listener_1_guard) {
                listener_1_triggered = true;
            } else if attachment_id.has_missed_deadline(&listener_2_guard) {
                listener_2_triggered = true;
            } else if attachment_id.has_event_from(&receiver_1_guard) {
                receiver_1_triggered = true;
            } else if attachment_id.has_missed_deadline(&receiver_2_guard) {
                receiver_2_triggered = true;
            } else {
                test_fail!("only attachments shall trigger");
            }

            CallbackProgression::Continue
        })
        .unwrap();

        assert_that!(listener_1_triggered, eq true);
        assert_that!(listener_2_triggered, eq true);
        assert_that!(receiver_1_triggered, eq true);
        assert_that!(receiver_2_triggered, eq true);
    }

    #[test]
    fn run_lists_all_ticks<S: Service>()
    where
        <S::Event as Event>::Listener: SynchronousMultiplexing,
    {
        let sut = WaitSetBuilder::new().create::<S>().unwrap();

        let tick_1_guard = sut.attach_interval(Duration::from_nanos(1)).unwrap();
        let tick_2_guard = sut.attach_interval(Duration::from_nanos(1)).unwrap();
        let tick_3_guard = sut.attach_interval(TIMEOUT * 1000).unwrap();
        let tick_4_guard = sut.attach_interval(TIMEOUT * 1000).unwrap();

        std::thread::sleep(TIMEOUT);

        let mut tick_1_triggered = false;
        let mut tick_2_triggered = false;
        let mut tick_3_triggered = false;
        let mut tick_4_triggered = false;

        sut.wait_and_process_once(|attachment_id| {
            if attachment_id.has_event_from(&tick_1_guard) {
                tick_1_triggered = true;
            } else if attachment_id.has_event_from(&tick_2_guard) {
                tick_2_triggered = true;
            } else if attachment_id.has_event_from(&tick_3_guard) {
                tick_3_triggered = true;
            } else if attachment_id.has_event_from(&tick_4_guard) {
                tick_4_triggered = true;
            } else {
                test_fail!("only attachments shall trigger");
            }

            CallbackProgression::Continue
        })
        .unwrap();

        assert_that!(tick_1_triggered, eq true);
        assert_that!(tick_2_triggered, eq true);
        assert_that!(tick_3_triggered, eq false);
        assert_that!(tick_4_triggered, eq false);
    }

    #[test]
    fn wait_and_process_stops_when_requested<S: Service>()
    where
        <S::Event as Event>::Listener: SynchronousMultiplexing,
    {
        let sut = WaitSetBuilder::new().create::<S>().unwrap();

        let _tick_1_guard = sut.attach_interval(Duration::from_nanos(1)).unwrap();
        let _tick_2_guard = sut.attach_interval(Duration::from_nanos(1)).unwrap();
        let _tick_3_guard = sut.attach_interval(TIMEOUT * 1000).unwrap();
        let _tick_4_guard = sut.attach_interval(TIMEOUT * 1000).unwrap();

        std::thread::sleep(TIMEOUT);

        let mut counter = 0;

        sut.wait_and_process(|_| {
            counter += 1;
            CallbackProgression::Stop
        })
        .unwrap();

        assert_that!(counter, eq 1);
    }

    #[test]
    fn run_lists_mixed<S: Service>()
    where
        <S::Event as Event>::Listener: SynchronousMultiplexing,
    {
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<S>().unwrap();
        let sut = WaitSetBuilder::new().create::<S>().unwrap();

        let (listener_1, notifier_1) = create_event::<S>(&node);
        let (listener_2, _notifier_2) = create_event::<S>(&node);
        let (listener_3, notifier_3) = create_event::<S>(&node);
        let (listener_4, _notifier_4) = create_event::<S>(&node);

        let tick_1_guard = sut.attach_interval(Duration::from_nanos(1)).unwrap();
        let tick_2_guard = sut.attach_interval(TIMEOUT * 1000).unwrap();
        let notification_1_guard = sut.attach_notification(&listener_1).unwrap();
        let notification_2_guard = sut.attach_notification(&listener_2).unwrap();
        let deadline_1_guard = sut.attach_deadline(&listener_3, TIMEOUT * 1000).unwrap();
        let deadline_2_guard = sut
            .attach_deadline(&listener_4, Duration::from_nanos(1))
            .unwrap();

        std::thread::sleep(TIMEOUT);

        notifier_1.notify().unwrap();
        notifier_3.notify().unwrap();

        let mut tick_1_triggered = false;
        let mut tick_2_triggered = false;
        let mut notification_1_triggered = false;
        let mut notification_2_triggered = false;
        let mut deadline_1_triggered = false;
        let mut deadline_2_triggered = false;
        let mut deadline_1_missed = false;
        let mut deadline_2_missed = false;

        sut.wait_and_process_once(|attachment_id| {
            if attachment_id.has_event_from(&tick_1_guard) {
                tick_1_triggered = true;
            } else if attachment_id.has_event_from(&tick_2_guard) {
                tick_2_triggered = true;
            } else if attachment_id.has_event_from(&notification_1_guard) {
                notification_1_triggered = true;
            } else if attachment_id.has_event_from(&notification_2_guard) {
                notification_2_triggered = true;
            } else if attachment_id.has_event_from(&deadline_1_guard) {
                deadline_1_triggered = true;
            } else if attachment_id.has_event_from(&deadline_2_guard) {
                deadline_2_triggered = true;
            } else if attachment_id.has_missed_deadline(&deadline_1_guard) {
                deadline_1_missed = true;
            } else if attachment_id.has_missed_deadline(&deadline_2_guard) {
                deadline_2_missed = true;
            } else {
                test_fail!("only attachments shall trigger");
            }

            CallbackProgression::Continue
        })
        .unwrap();

        assert_that!(tick_1_triggered, eq true);
        assert_that!(tick_2_triggered, eq false);
        assert_that!(notification_1_triggered, eq true);
        assert_that!(notification_2_triggered, eq false);
        assert_that!(deadline_1_triggered, eq true);
        assert_that!(deadline_2_triggered, eq false);
        assert_that!(deadline_1_missed, eq false);
        assert_that!(deadline_2_missed, eq true);
    }

    #[test]
    fn missed_deadline_and_then_notify_is_not_reported<S: Service>()
    where
        <S::Event as Event>::Listener: SynchronousMultiplexing,
    {
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<S>().unwrap();
        let sut = WaitSetBuilder::new().create::<S>().unwrap();

        let (listener_1, notifier_1) = create_event::<S>(&node);

        let deadline_1_guard = sut.attach_deadline(&listener_1, TIMEOUT).unwrap();

        std::thread::sleep(TIMEOUT + TIMEOUT / 10);
        notifier_1.notify().unwrap();

        // first we get informed by the waitset that we missed a deadline
        let mut missed_deadline = false;
        let mut received_event = false;

        sut.wait_and_process_once(|attachment_id| {
            if attachment_id.has_event_from(&deadline_1_guard) {
                received_event = true;
            } else if attachment_id.has_missed_deadline(&deadline_1_guard) {
                missed_deadline = true;
            } else {
                test_fail!("only attachments shall trigger");
            }

            CallbackProgression::Continue
        })
        .unwrap();

        assert_that!(missed_deadline, eq false);
        assert_that!(received_event, eq true);
    }

    #[instantiate_tests(<iceoryx2::service::ipc::Service>)]
    mod ipc {}

    #[instantiate_tests(<iceoryx2::service::local::Service>)]
    mod local {}
}
