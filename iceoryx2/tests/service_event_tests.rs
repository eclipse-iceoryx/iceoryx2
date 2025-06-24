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
mod service_event {
    use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
    use core::time::Duration;
    use std::collections::HashSet;
    use std::sync::Barrier;
    use std::time::Instant;

    use iceoryx2::port::listener::{Listener, ListenerCreateError};
    use iceoryx2::port::notifier::{NotifierCreateError, NotifierNotifyError};
    use iceoryx2::prelude::*;
    use iceoryx2::service::builder::event::{EventCreateError, EventOpenError};
    use iceoryx2::testing::*;
    use iceoryx2_bb_posix::unique_system_id::UniqueSystemId;
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_bb_testing::watchdog::Watchdog;

    const TIMEOUT: Duration = Duration::from_millis(50);

    fn generate_name() -> ServiceName {
        ServiceName::new(&format!(
            "service_tests_{}",
            UniqueSystemId::new().unwrap().value()
        ))
        .unwrap()
    }

    #[test]
    fn creating_non_existing_service_works<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node.service_builder(&service_name).event().create();

        assert_that!(sut, is_ok);
        let sut = sut.unwrap();
        assert_that!(*sut.name(), eq service_name);
    }

    #[test]
    fn creating_same_service_twice_fails<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node.service_builder(&service_name).event().create();
        assert_that!(sut, is_ok);

        let sut2 = node.service_builder(&service_name).event().create();
        assert_that!(sut2, is_err);
        assert_that!(
            sut2.err().unwrap(), eq
            EventCreateError::AlreadyExists
        );
    }

    #[test]
    fn recreate_after_drop_works<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node.service_builder(&service_name).event().create();
        assert_that!(sut, is_ok);

        drop(sut);

        let sut2 = node.service_builder(&service_name).event().create();
        assert_that!(sut2, is_ok);
    }

    #[test]
    fn open_fails_when_service_does_not_exist<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node.service_builder(&service_name).event().open();
        assert_that!(sut, is_err);
        assert_that!(sut.err().unwrap(), eq EventOpenError::DoesNotExist);
    }

    #[test]
    fn open_succeeds_when_service_does_exist<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node.service_builder(&service_name).event().create();
        assert_that!(sut, is_ok);

        let sut2 = node.service_builder(&service_name).event().open();
        assert_that!(sut2, is_ok);
    }

    #[test]
    fn open_fails_when_service_does_not_satisfy_opener_notifier_requirements<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .event()
            .max_notifiers(2)
            .create();
        assert_that!(sut, is_ok);

        let sut2 = node
            .service_builder(&service_name)
            .event()
            .max_notifiers(3)
            .open();

        assert_that!(sut2, is_err);
        assert_that!(
            sut2.err().unwrap(), eq
            EventOpenError::DoesNotSupportRequestedAmountOfNotifiers
        );

        let sut2 = node
            .service_builder(&service_name)
            .event()
            .max_notifiers(1)
            .open_or_create();
        assert_that!(sut2, is_ok);
    }

    #[test]
    fn open_fails_when_service_does_not_satisfy_opener_listener_requirements<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .event()
            .max_listeners(2)
            .create();
        assert_that!(sut, is_ok);

        let sut2 = node
            .service_builder(&service_name)
            .event()
            .max_listeners(3)
            .open();

        assert_that!(sut2, is_err);
        assert_that!(
            sut2.err().unwrap(), eq
            EventOpenError::DoesNotSupportRequestedAmountOfListeners
        );

        let sut2 = node
            .service_builder(&service_name)
            .event()
            .max_listeners(1)
            .open();
        assert_that!(sut2, is_ok);
    }

    #[test]
    fn set_max_nodes_to_zero_adjusts_it_to_one<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .event()
            .max_nodes(0)
            .create()
            .unwrap();

        assert_that!(sut.static_config().max_nodes(), eq 1);
    }

    #[test]
    fn set_max_listeners_to_zero_adjusts_it_to_one<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .event()
            .max_listeners(0)
            .create()
            .unwrap();

        assert_that!(sut.static_config().max_listeners(), eq 1);
    }

    #[test]
    fn set_max_notifiers_to_zero_adjusts_it_to_one<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .event()
            .max_notifiers(0)
            .create()
            .unwrap();

        assert_that!(sut.static_config().max_notifiers(), eq 1);
    }

    #[test]
    fn open_fails_when_service_does_not_satisfy_opener_node_requirements<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .event()
            .max_nodes(2)
            .create();
        assert_that!(sut, is_ok);

        let sut2 = node
            .service_builder(&service_name)
            .event()
            .max_nodes(3)
            .open();

        assert_that!(sut2, is_err);
        assert_that!(
            sut2.err().unwrap(), eq
            EventOpenError::DoesNotSupportRequestedAmountOfNodes
        );

        let sut2 = node
            .service_builder(&service_name)
            .event()
            .max_nodes(1)
            .open();
        assert_that!(sut2, is_ok);
    }

    #[test]
    fn open_fails_when_service_does_not_satisfy_event_id_requirements<Sut: Service>() {
        let service_name = generate_name();
        const EVENT_ID_MAX_VALUE: usize = 78;
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let _sut = node
            .service_builder(&service_name)
            .event()
            .event_id_max_value(EVENT_ID_MAX_VALUE)
            .create();

        let sut2 = node
            .service_builder(&service_name)
            .event()
            .event_id_max_value(EVENT_ID_MAX_VALUE + 1)
            .open();

        assert_that!(sut2, is_err);
        assert_that!(sut2.err().unwrap(), eq EventOpenError::DoesNotSupportRequestedMaxEventId);

        let sut2 = node
            .service_builder(&service_name)
            .event()
            .event_id_max_value(EVENT_ID_MAX_VALUE)
            .open();

        assert_that!(sut2, is_ok);
    }

    #[test]
    fn open_uses_predefined_settings_when_nothing_is_specified<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .event()
            .max_nodes(7)
            .max_notifiers(4)
            .max_listeners(5)
            .notifier_dead_event(EventId::new(8))
            .notifier_dropped_event(EventId::new(9))
            .notifier_created_event(EventId::new(10))
            .create()
            .unwrap();
        assert_that!(sut.static_config().max_nodes(), eq 7);
        assert_that!(sut.static_config().max_notifiers(), eq 4);
        assert_that!(sut.static_config().max_listeners(), eq 5);
        assert_that!(sut.static_config().notifier_dead_event(), eq Some(EventId::new(8)));
        assert_that!(sut.static_config().notifier_dropped_event(), eq Some(EventId::new(9)));
        assert_that!(sut.static_config().notifier_created_event(), eq Some(EventId::new(10)));

        let sut2 = node.service_builder(&service_name).event().open().unwrap();
        assert_that!(sut2.static_config().max_nodes(), eq 7);
        assert_that!(sut2.static_config().max_notifiers(), eq 4);
        assert_that!(sut2.static_config().max_listeners(), eq 5);
        assert_that!(sut2.static_config().notifier_dead_event(), eq Some(EventId::new(8)));
        assert_that!(sut2.static_config().notifier_dropped_event(), eq Some(EventId::new(9)));
        assert_that!(sut2.static_config().notifier_created_event(), eq Some(EventId::new(10)));
    }

    #[test]
    fn settings_can_be_modified_via_custom_config<Sut: Service>() {
        let service_name = generate_name();
        let mut custom_config = generate_isolated_config();
        custom_config.defaults.event.max_nodes = 13;
        custom_config.defaults.event.max_notifiers = 9;
        custom_config.defaults.event.max_listeners = 10;
        let node = NodeBuilder::new()
            .config(&custom_config)
            .create::<Sut>()
            .unwrap();

        let sut = node
            .service_builder(&service_name)
            .event()
            .create()
            .unwrap();
        assert_that!(sut.static_config().max_nodes(), eq 13);
        assert_that!(sut.static_config().max_notifiers(), eq 9);
        assert_that!(sut.static_config().max_listeners(), eq 10);

        let sut2 = node.service_builder(&service_name).event().open().unwrap();
        assert_that!(sut2.static_config().max_nodes(), eq 13);
        assert_that!(sut2.static_config().max_notifiers(), eq 9);
        assert_that!(sut2.static_config().max_listeners(), eq 10);
    }

    #[test]
    fn simple_communication_works_listener_created_first<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let event_id = EventId::new(32);

        let sut = node
            .service_builder(&service_name)
            .event()
            .create()
            .unwrap();

        let sut2 = node.service_builder(&service_name).event().open().unwrap();

        let listener = sut.listener_builder().create().unwrap();
        let notifier = sut2
            .notifier_builder()
            .default_event_id(event_id)
            .create()
            .unwrap();

        assert_that!(notifier.notify(), is_ok);

        let mut received_events = 0;
        for event in listener.try_wait_one().unwrap().iter() {
            assert_that!(*event, eq event_id);
            received_events += 1;
        }
        assert_that!(received_events, eq 1);
    }

    #[test]
    fn simple_communication_works_notifier_created_first<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let event_id = EventId::new(23);

        let sut = node
            .service_builder(&service_name)
            .event()
            .create()
            .unwrap();

        let sut2 = node.service_builder(&service_name).event().open().unwrap();

        let notifier = sut2
            .notifier_builder()
            .default_event_id(event_id)
            .create()
            .unwrap();
        let listener = sut.listener_builder().create().unwrap();

        assert_that!(notifier.notify(), is_ok);

        let mut received_events = 0;
        for event in listener.try_wait_one().unwrap().iter() {
            assert_that!(*event, eq event_id);
            received_events += 1;
        }
        assert_that!(received_events, eq 1);
    }

    #[test]
    fn notifier_emits_create_and_dropped_event_id<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let sut = node
            .service_builder(&service_name)
            .event()
            .disable_notifier_created_event()
            .disable_notifier_dropped_event()
            .create()
            .unwrap();

        let sut2 = node.service_builder(&service_name).event().open().unwrap();

        let listener = sut.listener_builder().create().unwrap();
        let notifier = sut2.notifier_builder().create().unwrap();

        let mut received_events = 0;
        for _ in listener.try_wait_one().unwrap().iter() {
            received_events += 1;
        }
        assert_that!(received_events, eq 0);

        drop(notifier);

        let mut received_events = 0;
        for _ in listener.try_wait_one().unwrap().iter() {
            received_events += 1;
        }
        assert_that!(received_events, eq 0);
    }

    #[test]
    fn notifier_emits_nothing_when_no_events_are_configured<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let notifier_created = EventId::new(31);
        let notifier_dropped = EventId::new(28);

        let sut = node
            .service_builder(&service_name)
            .event()
            .notifier_created_event(notifier_created)
            .notifier_dropped_event(notifier_dropped)
            .create()
            .unwrap();

        let sut2 = node.service_builder(&service_name).event().open().unwrap();

        let listener = sut.listener_builder().create().unwrap();
        let notifier = sut2.notifier_builder().create().unwrap();

        let mut received_events = 0;
        for event in listener.try_wait_one().unwrap().iter() {
            assert_that!(*event, eq notifier_created);
            received_events += 1;
        }
        assert_that!(received_events, eq 1);

        drop(notifier);

        let mut received_events = 0;
        for event in listener.try_wait_one().unwrap().iter() {
            assert_that!(*event, eq notifier_dropped);
            received_events += 1;
        }
        assert_that!(received_events, eq 1);
    }

    #[test]
    fn communication_with_max_notifiers_and_listeners_single_notification<Sut: Service>() {
        const MAX_LISTENERS: usize = 4;
        const MAX_NOTIFIERS: usize = 6;
        const NUMBER_OF_ITERATIONS: u64 = 128;
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let sut = node
            .service_builder(&service_name)
            .event()
            .max_notifiers(MAX_NOTIFIERS)
            .max_listeners(MAX_LISTENERS)
            .create()
            .unwrap();

        let mut listeners = vec![];
        let mut notifiers = vec![];

        for _ in 0..MAX_LISTENERS {
            listeners.push(sut.listener_builder().create().unwrap());
        }

        for i in 0..MAX_NOTIFIERS {
            notifiers.push(
                sut.notifier_builder()
                    .default_event_id(EventId::new(i + 3))
                    .create()
                    .unwrap(),
            );
        }

        for _ in 0..NUMBER_OF_ITERATIONS {
            for (i, notifier) in notifiers.iter().enumerate() {
                assert_that!(notifier.notify(), is_ok);

                for listener in &mut listeners {
                    let mut received_events = 0;
                    for event in listener.try_wait_one().unwrap().iter() {
                        assert_that!(*event, eq EventId::new(i + 3));
                        received_events += 1;
                    }
                    assert_that!(received_events, eq 1);
                }
            }
        }
    }

    #[test]
    fn communication_with_max_notifiers_and_listeners_multi_notification<Sut: Service>() {
        const MAX_LISTENERS: usize = 5;
        const MAX_NOTIFIERS: usize = 7;
        const NUMBER_OF_ITERATIONS: u64 = 128;
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let sut = node
            .service_builder(&service_name)
            .event()
            .max_notifiers(MAX_NOTIFIERS)
            .max_listeners(MAX_LISTENERS)
            .create()
            .unwrap();

        let mut listeners = vec![];
        let mut notifiers = vec![];

        for _ in 0..MAX_LISTENERS {
            listeners.push(sut.listener_builder().create().unwrap());
        }

        for i in 0..MAX_NOTIFIERS {
            notifiers.push(
                sut.notifier_builder()
                    .default_event_id(EventId::new(i))
                    .create()
                    .unwrap(),
            );
        }

        for _ in 0..NUMBER_OF_ITERATIONS {
            for notifier in &notifiers {
                assert_that!(notifier.notify(), is_ok);
            }

            for listener in &mut listeners {
                let mut received_events = 0;

                let mut received_event_ids = [false; MAX_NOTIFIERS];
                while let Some(event) = listener.try_wait_one().unwrap() {
                    assert_that!(received_event_ids[event.as_value()], eq false);
                    received_event_ids[event.as_value()] = true;
                    received_events += 1;
                }
                assert_that!(received_events, eq MAX_NOTIFIERS);
            }
        }
    }

    #[test]
    fn number_of_notifiers_works<Sut: Service>() {
        let service_name = generate_name();
        const MAX_NOTIFIERS: usize = 8;
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let sut = node
            .service_builder(&service_name)
            .event()
            .max_notifiers(MAX_NOTIFIERS)
            .create()
            .unwrap();

        let sut2 = node.service_builder(&service_name).event().open().unwrap();

        let mut notifiers = vec![];

        for i in 0..MAX_NOTIFIERS / 2 {
            notifiers.push(sut.notifier_builder().create().unwrap());
            assert_that!(sut.dynamic_config().number_of_notifiers(), eq 2 * i + 1);
            assert_that!(sut2.dynamic_config().number_of_notifiers(), eq 2 * i + 1);
            assert_that!(sut.dynamic_config().number_of_listeners(), eq 0);
            assert_that!(sut2.dynamic_config().number_of_listeners(), eq 0);

            notifiers.push(sut2.notifier_builder().create().unwrap());
            assert_that!(sut.dynamic_config().number_of_notifiers(), eq 2 * i + 2);
            assert_that!(sut2.dynamic_config().number_of_notifiers(), eq 2 * i + 2);
            assert_that!(sut.dynamic_config().number_of_listeners(), eq 0);
            assert_that!(sut2.dynamic_config().number_of_listeners(), eq 0);
        }

        let notifier = sut.notifier_builder().create();
        assert_that!(notifier, is_err);
        assert_that!(notifier.err().unwrap(), eq NotifierCreateError::ExceedsMaxSupportedNotifiers);

        for i in 0..MAX_NOTIFIERS {
            notifiers.pop();
            assert_that!(sut.dynamic_config().number_of_notifiers(), eq MAX_NOTIFIERS - i - 1);
            assert_that!(sut2.dynamic_config().number_of_notifiers(), eq MAX_NOTIFIERS - i - 1);
        }
    }

    #[test]
    fn number_of_listeners_works<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        const MAX_LISTENERS: usize = 8;

        let sut = node
            .service_builder(&service_name)
            .event()
            .max_listeners(MAX_LISTENERS)
            .create()
            .unwrap();

        let sut2 = node.service_builder(&service_name).event().open().unwrap();

        let mut listeners = vec![];

        for i in 0..MAX_LISTENERS / 2 {
            listeners.push(sut.listener_builder().create().unwrap());
            assert_that!(sut.dynamic_config().number_of_listeners(), eq 2 * i + 1);
            assert_that!(sut2.dynamic_config().number_of_listeners(), eq 2 * i + 1);
            assert_that!(sut.dynamic_config().number_of_notifiers(), eq 0);
            assert_that!(sut2.dynamic_config().number_of_notifiers(), eq 0);

            listeners.push(sut2.listener_builder().create().unwrap());
            assert_that!(sut.dynamic_config().number_of_listeners(), eq 2 * i + 2);
            assert_that!(sut2.dynamic_config().number_of_listeners(), eq 2 * i + 2);
            assert_that!(sut.dynamic_config().number_of_notifiers(), eq 0);
            assert_that!(sut2.dynamic_config().number_of_notifiers(), eq 0);
        }

        let listener = sut.listener_builder().create();
        assert_that!(listener, is_err);
        assert_that!(listener.err().unwrap(), eq ListenerCreateError::ExceedsMaxSupportedListeners);

        for i in 0..MAX_LISTENERS {
            listeners.pop();
            assert_that!(sut.dynamic_config().number_of_listeners(), eq MAX_LISTENERS - i - 1);
            assert_that!(sut2.dynamic_config().number_of_listeners(), eq MAX_LISTENERS - i - 1);
        }
    }

    #[test]
    fn number_of_nodes_works<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let main_node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        const MAX_NODES: usize = 8;

        let _sut = main_node
            .service_builder(&service_name)
            .event()
            .max_nodes(MAX_NODES)
            .create()
            .unwrap();

        let mut nodes = vec![];
        let mut services = vec![];

        for _ in 1..MAX_NODES {
            let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
            let service = node.service_builder(&service_name).event().open();
            assert_that!(service, is_ok);
            nodes.push(node);
            services.push(service);
        }

        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let service = node.service_builder(&service_name).event().open();
        assert_that!(service, is_err);
        assert_that!(service.err().unwrap(), eq EventOpenError::ExceedsMaxNumberOfNodes);

        nodes.pop();
        services.pop();

        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let service = node.service_builder(&service_name).event().open();
        assert_that!(service, is_ok);
    }

    #[test]
    fn max_event_id_works<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        const EVENT_ID_MAX_VALUE: usize = 78;

        let sut = node
            .service_builder(&service_name)
            .event()
            .event_id_max_value(EVENT_ID_MAX_VALUE)
            .create()
            .unwrap();

        let sut2 = node.service_builder(&service_name).event().open().unwrap();

        let listener = sut.listener_builder().create().unwrap();
        let notifier = sut2.notifier_builder().create().unwrap();

        for i in 0..=EVENT_ID_MAX_VALUE {
            assert_that!(notifier
                .notify_with_custom_event_id(EventId::new(i))
                .unwrap(), eq 1);
            assert_that!(listener.try_wait_one().unwrap(), eq Some(EventId::new(i)));
        }

        let result = notifier.notify_with_custom_event_id(EventId::new(EVENT_ID_MAX_VALUE + 1));
        assert_that!(result, is_err);
        assert_that!(result.err().unwrap(), eq NotifierNotifyError::EventIdOutOfBounds);
    }

    #[test]
    fn concurrent_reconnecting_notifier_can_trigger_waiting_listener<Sut: Service>() {
        let _watch_dog = Watchdog::new_with_timeout(Duration::from_secs(120));

        let number_of_listener_threads = 2;
        let number_of_notifier_threads = 2;
        const NUMBER_OF_ITERATIONS: usize = 100;
        const EVENT_ID: EventId = EventId::new(8);

        let keep_running = AtomicBool::new(true);
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let barrier = Barrier::new(number_of_notifier_threads + number_of_listener_threads);

        let sut = node
            .service_builder(&service_name)
            .event()
            .max_listeners(number_of_listener_threads)
            .max_notifiers(number_of_notifier_threads)
            .create()
            .unwrap();

        std::thread::scope(|s| {
            let mut listener_threads = vec![];
            for _ in 0..number_of_listener_threads {
                listener_threads.push(s.spawn(|| {
                    let listener = sut.listener_builder().create().unwrap();
                    barrier.wait();

                    let mut counter = 0;
                    while counter < NUMBER_OF_ITERATIONS {
                        let event_ids = listener.blocking_wait_one().unwrap();
                        if let Some(id) = event_ids {
                            counter += 1;
                            assert_that!(id, eq EVENT_ID);
                        }
                    }
                }));
            }

            for _ in 0..number_of_notifier_threads {
                s.spawn(|| {
                    barrier.wait();

                    while keep_running.load(Ordering::Relaxed) {
                        let notifier = sut.notifier_builder().create().unwrap();
                        assert_that!(notifier.notify_with_custom_event_id(EVENT_ID), is_ok);
                    }
                });
            }

            for thread in listener_threads {
                thread.join().unwrap();
            }

            keep_running.store(false, Ordering::Relaxed);
        });
    }

    #[test]
    fn concurrent_reconnecting_listener_can_wait_for_triggering_notifiers<Sut: Service>() {
        let _watch_dog = Watchdog::new_with_timeout(Duration::from_secs(120));

        let number_of_listener_threads = 2;
        let number_of_notifier_threads = 2;
        const NUMBER_OF_ITERATIONS: usize = 100;
        const EVENT_ID: EventId = EventId::new(8);

        let keep_running = AtomicBool::new(true);
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let barrier = Barrier::new(number_of_listener_threads + number_of_notifier_threads);

        let sut = node
            .service_builder(&service_name)
            .event()
            .max_listeners(number_of_listener_threads * 2)
            .max_notifiers(number_of_notifier_threads)
            .create()
            .unwrap();

        std::thread::scope(|s| {
            let mut listener_threads = vec![];
            for _ in 0..number_of_listener_threads {
                listener_threads.push(s.spawn(|| {
                    barrier.wait();

                    let mut counter = 0;
                    let mut listener = sut.listener_builder().create().unwrap();
                    while counter < NUMBER_OF_ITERATIONS {
                        let event_ids = listener.blocking_wait_one().unwrap();
                        if let Some(id) = event_ids {
                            counter += 1;
                            assert_that!(id, eq EVENT_ID);
                            listener = sut.listener_builder().create().unwrap();
                        }
                    }
                }));
            }

            for _ in 0..number_of_notifier_threads {
                s.spawn(|| {
                    let notifier = sut.notifier_builder().create().unwrap();
                    barrier.wait();

                    while keep_running.load(Ordering::Relaxed) {
                        assert_that!(notifier.notify_with_custom_event_id(EVENT_ID), is_ok);
                    }
                });
            }

            for thread in listener_threads {
                thread.join().unwrap();
            }

            keep_running.store(false, Ordering::Relaxed);
        });
    }

    #[test]
    fn service_persists_when_service_object_is_dropped_but_endpoints_are_still_alive<
        Sut: Service,
    >() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let event_id = EventId::new(12);

        let sut = node
            .service_builder(&service_name)
            .event()
            .create()
            .unwrap();

        let notifier = sut
            .notifier_builder()
            .default_event_id(event_id)
            .create()
            .unwrap();
        let listener = sut.listener_builder().create().unwrap();

        assert_that!(Sut::does_exist(&service_name, &config, MessagingPattern::Event), eq Ok(true));
        drop(sut);
        assert_that!(Sut::does_exist(&service_name, &config, MessagingPattern::Event), eq Ok(true));

        assert_that!(notifier.notify(), eq Ok(1));

        let mut received_events = 0;
        for event in listener.try_wait_one().unwrap().iter() {
            assert_that!(*event, eq event_id);
            received_events += 1;
        }
        assert_that!(received_events, eq 1);
    }

    #[test]
    fn ports_of_dropped_service_block_new_service_creation<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .event()
            .create()
            .unwrap();

        let listener = sut.listener_builder().create().unwrap();
        let notifier = sut.notifier_builder().create().unwrap();

        drop(sut);

        assert_that!(node.service_builder(&service_name)
            .event()
            .create().err().unwrap(),
            eq EventCreateError::AlreadyExists);

        drop(listener);

        assert_that!(node.service_builder(&service_name)
            .event()
            .create().err().unwrap(),
            eq EventCreateError::AlreadyExists);

        drop(notifier);

        assert_that!(node.service_builder(&service_name).event().create(), is_ok);
    }

    #[test]
    fn service_can_be_opened_when_there_is_a_notifier<Sut: Service>() {
        let event_id = EventId::new(76);
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .event()
            .create()
            .unwrap();
        let listener = sut.listener_builder().create().unwrap();
        let notifier = sut.notifier_builder().create().unwrap();

        drop(sut);
        let sut = node.service_builder(&service_name).event().open();
        assert_that!(sut, is_ok);
        drop(sut);
        let sut = node.service_builder(&service_name).event().create();
        assert_that!(sut.err().unwrap(), eq EventCreateError::AlreadyExists);
        drop(listener);

        let sut = node.service_builder(&service_name).event().open().unwrap();
        let listener = sut.listener_builder().create().unwrap();
        notifier.notify_with_custom_event_id(event_id).unwrap();
        let notification = listener.try_wait_one().unwrap();
        assert_that!(notification, eq Some(event_id));

        drop(listener);
        drop(sut);
        drop(notifier);

        let sut = node.service_builder(&service_name).event().open();
        assert_that!(sut.err().unwrap(), eq EventOpenError::DoesNotExist);
        let sut = node.service_builder(&service_name).event().create();
        assert_that!(sut, is_ok);
    }

    #[test]
    fn service_can_be_opened_when_there_is_a_listener<Sut: Service>() {
        let event_id = EventId::new(93);
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let sut = node
            .service_builder(&service_name)
            .event()
            .create()
            .unwrap();
        let listener = sut.listener_builder().create().unwrap();
        let notifier = sut.notifier_builder().create().unwrap();

        drop(sut);
        let sut = node.service_builder(&service_name).event().open();
        assert_that!(sut, is_ok);
        drop(sut);
        let sut = node.service_builder(&service_name).event().create();
        assert_that!(sut.err().unwrap(), eq EventCreateError::AlreadyExists);
        drop(notifier);

        let sut = node.service_builder(&service_name).event().open().unwrap();
        let notifier = sut.notifier_builder().create().unwrap();
        notifier.notify_with_custom_event_id(event_id).unwrap();
        let notification = listener.try_wait_one().unwrap();
        assert_that!(notification, eq Some(event_id));

        drop(notifier);
        drop(sut);
        drop(listener);

        let sut = node.service_builder(&service_name).event().open();
        assert_that!(sut.err().unwrap(), eq EventOpenError::DoesNotExist);
        let sut = node.service_builder(&service_name).event().create();
        assert_that!(sut, is_ok);
    }

    #[test]
    fn try_wait_does_not_block<Sut: Service>() {
        let _watch_dog = Watchdog::new();
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let sut = node
            .service_builder(&service_name)
            .event()
            .create()
            .unwrap();
        let listener = sut.listener_builder().create().unwrap();

        assert_that!(listener.try_wait_one(), is_ok);
    }

    #[test]
    fn timed_wait_blocks_for_at_least_timeout<Sut: Service>() {
        let _watch_dog = Watchdog::new();
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let sut = node
            .service_builder(&service_name)
            .event()
            .create()
            .unwrap();

        let listener = sut.listener_builder().create().unwrap();

        let now = Instant::now();
        assert_that!(listener.timed_wait_one(TIMEOUT), is_ok);
        assert_that!(now.elapsed(), time_at_least TIMEOUT);
    }

    fn wait_blocks_until_notification<Sut: Service, F: FnMut(&Listener<Sut>) + Send>(
        mut wait_call: F,
    ) {
        let _watch_dog = Watchdog::new();
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let sut = node
            .service_builder(&service_name)
            .event()
            .create()
            .unwrap();
        let notifier = sut.notifier_builder().create().unwrap();
        let counter = AtomicU64::new(0);
        let barrier = Barrier::new(2);

        std::thread::scope(|s| {
            let t = s.spawn(|| {
                let listener = sut.listener_builder().create().unwrap();
                barrier.wait();
                wait_call(&listener);
                counter.fetch_add(1, Ordering::Relaxed);
            });

            barrier.wait();
            std::thread::sleep(TIMEOUT);
            assert_that!(counter.load(Ordering::Relaxed), eq 0);

            assert_that!(notifier.notify_with_custom_event_id(EventId::new(13)).unwrap(), eq 1);
            t.join().unwrap();
            assert_that!(counter.load(Ordering::Relaxed), eq 1);
        });
    }

    #[test]
    fn timed_wait_blocks_until_notification<Sut: Service>() {
        wait_blocks_until_notification(|l: &Listener<Sut>| {
            let id = l.timed_wait_one(TIMEOUT * 1000).unwrap();
            assert_that!(id, eq Some(EventId::new(13)));
        })
    }

    #[test]
    fn blocking_wait_blocks_until_notification<Sut: Service>() {
        wait_blocks_until_notification(|l: &Listener<Sut>| {
            let id = l.blocking_wait_one().unwrap();
            assert_that!(id, eq Some(EventId::new(13)));
        })
    }

    #[test]
    fn try_wait_collects_all_notifications<Sut: Service>() {
        const NUMBER_OF_NOTIFICATIONS: usize = 8;
        wait_collects_all_notifications(NUMBER_OF_NOTIFICATIONS, |l: &Listener<Sut>, ids| {
            while let Some(id) = l.try_wait_one().unwrap() {
                assert_that!(ids.insert(id), eq true);
            }
        });
    }

    #[test]
    fn timed_wait_collects_all_notifications<Sut: Service>() {
        const NUMBER_OF_NOTIFICATIONS: usize = 8;
        wait_collects_all_notifications(NUMBER_OF_NOTIFICATIONS, |l: &Listener<Sut>, ids| {
            for _ in 0..NUMBER_OF_NOTIFICATIONS {
                let id = l.timed_wait_one(TIMEOUT).unwrap().unwrap();
                assert_that!(ids.insert(id), eq true);
            }
        });
    }

    #[test]
    fn blocking_wait_collects_all_notifications<Sut: Service>() {
        const NUMBER_OF_NOTIFICATIONS: usize = 8;
        wait_collects_all_notifications(NUMBER_OF_NOTIFICATIONS, |l: &Listener<Sut>, ids| {
            for _ in 0..NUMBER_OF_NOTIFICATIONS {
                let id = l.blocking_wait_one().unwrap().unwrap();
                assert_that!(ids.insert(id), eq true);
            }
        });
    }

    #[test]
    fn try_wait_all_does_not_block<Sut: Service>() {
        let _watch_dog = Watchdog::new();
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let sut = node
            .service_builder(&service_name)
            .event()
            .create()
            .unwrap();

        let listener = sut.listener_builder().create().unwrap();

        let mut callback_called = false;
        assert_that!(listener.try_wait_all(|_| callback_called = true), is_ok);
        assert_that!(callback_called, eq false);
    }

    #[test]
    fn timed_wait_all_blocks_for_at_least_timeout<Sut: Service>() {
        let _watch_dog = Watchdog::new();
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let sut = node
            .service_builder(&service_name)
            .event()
            .create()
            .unwrap();

        let listener = sut.listener_builder().create().unwrap();

        let now = Instant::now();
        let mut callback_called = false;
        assert_that!(
            listener.timed_wait_all(|_| callback_called = true, TIMEOUT),
            is_ok
        );
        assert_that!(now.elapsed(), time_at_least TIMEOUT);
        assert_that!(callback_called, eq false);
    }

    #[test]
    fn timed_wait_all_blocks_until_notification<Sut: Service>() {
        let mut callback_was_called = false;
        wait_blocks_until_notification(|l: &Listener<Sut>| {
            assert_that!(
                l.timed_wait_all(
                    |id| {
                        assert_that!(id, eq EventId::new(13));
                        callback_was_called = true;
                    },
                    TIMEOUT * 1000
                ),
                is_ok
            );
        });
        assert_that!(callback_was_called, eq true);
    }

    #[test]
    fn blocking_wait_all_blocks_until_notification<Sut: Service>() {
        let mut callback_was_called = false;
        wait_blocks_until_notification(|l: &Listener<Sut>| {
            assert_that!(
                l.blocking_wait_all(|id| {
                    assert_that!(id, eq EventId::new(13));
                    callback_was_called = true;
                }),
                is_ok
            );
        });
        assert_that!(callback_was_called, eq true);
    }

    fn wait_collects_all_notifications<
        Sut: Service,
        F: FnMut(&Listener<Sut>, &mut HashSet<EventId>),
    >(
        number_of_notifications: usize,
        mut wait_call: F,
    ) {
        let _watch_dog = Watchdog::new();
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<Sut>().unwrap();

        let sut = node
            .service_builder(&service_name)
            .event()
            .event_id_max_value(number_of_notifications)
            .create()
            .unwrap();
        let listener = sut.listener_builder().create().unwrap();
        let notifier = sut.notifier_builder().create().unwrap();

        for i in 0..number_of_notifications {
            assert_that!(notifier.notify_with_custom_event_id(EventId::new(i)).unwrap(), eq 1);
        }

        let mut id_set = HashSet::new();
        wait_call(&listener, &mut id_set);
    }

    #[test]
    fn try_wait_all_collects_all_notifications<Sut: Service>() {
        const NUMBER_OF_NOTIFICATIONS: usize = 8;
        wait_collects_all_notifications(NUMBER_OF_NOTIFICATIONS, |l: &Listener<Sut>, ids| {
            let result = l.try_wait_all(|id| assert_that!(ids.insert(id), eq true));
            assert_that!(result, is_ok);
        });
    }

    #[test]
    fn timed_wait_all_collects_all_notifications<Sut: Service>() {
        const NUMBER_OF_NOTIFICATIONS: usize = 8;
        wait_collects_all_notifications(NUMBER_OF_NOTIFICATIONS, |l: &Listener<Sut>, ids| {
            let result = l.timed_wait_all(|id| assert_that!(ids.insert(id), eq true), TIMEOUT);
            assert_that!(result, is_ok);
        });
    }

    #[test]
    fn blocking_wait_all_collects_all_notifications<Sut: Service>() {
        const NUMBER_OF_NOTIFICATIONS: usize = 8;
        wait_collects_all_notifications(NUMBER_OF_NOTIFICATIONS, |l: &Listener<Sut>, ids| {
            let result = l.blocking_wait_all(|id| assert_that!(ids.insert(id), eq true));
            assert_that!(result, is_ok);
        });
    }

    #[test]
    fn open_error_display_works<S: Service>() {
        assert_that!(
            format!("{}", EventOpenError::DoesNotExist), eq "EventOpenError::DoesNotExist");
        assert_that!(
            format!("{}", EventOpenError::InsufficientPermissions), eq "EventOpenError::InsufficientPermissions");
        assert_that!(
            format!("{}", EventOpenError::ServiceInCorruptedState), eq "EventOpenError::ServiceInCorruptedState");
        assert_that!(
            format!("{}", EventOpenError::IncompatibleMessagingPattern), eq "EventOpenError::IncompatibleMessagingPattern");
        assert_that!(
            format!("{}", EventOpenError::IncompatibleAttributes), eq "EventOpenError::IncompatibleAttributes");
        assert_that!(
            format!("{}", EventOpenError::InternalFailure), eq "EventOpenError::InternalFailure");
        assert_that!(
            format!("{}", EventOpenError::HangsInCreation), eq "EventOpenError::HangsInCreation");
        assert_that!(
            format!("{}", EventOpenError::DoesNotSupportRequestedAmountOfNotifiers), eq "EventOpenError::DoesNotSupportRequestedAmountOfNotifiers");
        assert_that!(
            format!("{}", EventOpenError::DoesNotSupportRequestedAmountOfListeners), eq "EventOpenError::DoesNotSupportRequestedAmountOfListeners");
        assert_that!(
            format!("{}", EventOpenError::DoesNotSupportRequestedMaxEventId), eq "EventOpenError::DoesNotSupportRequestedMaxEventId");
    }

    #[test]
    fn create_error_display_works<S: Service>() {
        assert_that!(
            format!("{}", EventCreateError::ServiceInCorruptedState), eq "EventCreateError::ServiceInCorruptedState");
        assert_that!(
            format!("{}", EventCreateError::InternalFailure), eq "EventCreateError::InternalFailure");
        assert_that!(
            format!("{}", EventCreateError::InsufficientPermissions), eq "EventCreateError::InsufficientPermissions");
        assert_that!(
            format!("{}", EventCreateError::IsBeingCreatedByAnotherInstance), eq "EventCreateError::IsBeingCreatedByAnotherInstance");
    }

    #[test]
    fn deadline_can_be_set<S: Service>() {
        const DEADLINE: Duration = Duration::from_secs(556);
        let service_name = generate_name();
        let mut config = generate_isolated_config();
        config.defaults.event.deadline = None;
        let node = NodeBuilder::new().config(&config).create::<S>().unwrap();

        let sut_create = node
            .service_builder(&service_name)
            .event()
            .deadline(DEADLINE)
            .create()
            .unwrap();
        let listener_create = sut_create.listener_builder().create().unwrap();
        let notifier_create = sut_create.notifier_builder().create().unwrap();

        let sut_open = node.service_builder(&service_name).event().open().unwrap();
        let listener_open = sut_open.listener_builder().create().unwrap();
        let notifier_open = sut_open.notifier_builder().create().unwrap();

        assert_that!(sut_create.static_config().deadline(), eq Some(DEADLINE));
        assert_that!(sut_open.static_config().deadline(), eq Some(DEADLINE));

        assert_that!(listener_create.deadline(), eq Some(DEADLINE));
        assert_that!(listener_open.deadline(), eq Some(DEADLINE));
        assert_that!(notifier_create.deadline(), eq Some(DEADLINE));
        assert_that!(notifier_open.deadline(), eq Some(DEADLINE));
    }

    #[test]
    fn deadline_can_be_disabled<S: Service>() {
        const DEADLINE: Duration = Duration::from_secs(556);
        let service_name = generate_name();
        let mut config = generate_isolated_config();
        config.defaults.event.deadline = Some(DEADLINE);
        let node = NodeBuilder::new().config(&config).create::<S>().unwrap();

        let sut_create = node
            .service_builder(&service_name)
            .event()
            .disable_deadline()
            .create()
            .unwrap();
        let listener_create = sut_create.listener_builder().create().unwrap();
        let notifier_create = sut_create.notifier_builder().create().unwrap();

        let sut_open = node.service_builder(&service_name).event().open().unwrap();
        let listener_open = sut_open.listener_builder().create().unwrap();
        let notifier_open = sut_open.notifier_builder().create().unwrap();

        assert_that!(sut_create.static_config().deadline(), eq None);
        assert_that!(sut_open.static_config().deadline(), eq None);

        assert_that!(listener_create.deadline(), eq None);
        assert_that!(listener_open.deadline(), eq None);
        assert_that!(notifier_create.deadline(), eq None);
        assert_that!(notifier_open.deadline(), eq None);
    }

    #[test]
    fn notifier_is_informed_when_deadline_was_missed<S: Service>() {
        const DEADLINE: Duration = Duration::from_nanos(1);
        const TIMEOUT: Duration = Duration::from_millis(10);
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<S>().unwrap();

        let sut_create = node
            .service_builder(&service_name)
            .event()
            .deadline(DEADLINE)
            .create()
            .unwrap();

        let listener = sut_create.listener_builder().create().unwrap();
        let notifier_create = sut_create.notifier_builder().create().unwrap();

        let sut_open = node.service_builder(&service_name).event().open().unwrap();
        let notifier_open = sut_open.notifier_builder().create().unwrap();

        std::thread::sleep(TIMEOUT);
        let result = notifier_create.notify();
        assert_that!(result.err(), eq Some(NotifierNotifyError::MissedDeadline));
        assert_that!(listener.try_wait_one().unwrap(), is_some);

        std::thread::sleep(TIMEOUT);
        let result = notifier_open.notify();
        assert_that!(result.err(), eq Some(NotifierNotifyError::MissedDeadline));
        assert_that!(listener.try_wait_one().unwrap(), is_some);
    }

    #[test]
    fn when_deadline_is_not_missed_notification_works<S: Service>() {
        const DEADLINE: Duration = Duration::from_secs(3600);
        const TIMEOUT: Duration = Duration::from_millis(10);
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<S>().unwrap();

        let sut_create = node
            .service_builder(&service_name)
            .event()
            .deadline(DEADLINE)
            .create()
            .unwrap();

        let listener = sut_create.listener_builder().create().unwrap();
        let notifier_create = sut_create.notifier_builder().create().unwrap();

        let sut_open = node.service_builder(&service_name).event().open().unwrap();
        let notifier_open = sut_open.notifier_builder().create().unwrap();

        std::thread::sleep(TIMEOUT);
        assert_that!(notifier_create.notify(), is_ok);
        assert_that!(listener.try_wait_one().unwrap(), is_some);

        std::thread::sleep(TIMEOUT);
        assert_that!(notifier_open.notify(), is_ok);
        assert_that!(listener.try_wait_one().unwrap(), is_some);
    }

    #[test]
    fn listing_all_notifiers_works<S: Service>() {
        const NUMBER_OF_NOTIFIERS: usize = 18;
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<S>().unwrap();

        let sut = node
            .service_builder(&service_name)
            .event()
            .max_notifiers(NUMBER_OF_NOTIFIERS)
            .create()
            .unwrap();

        let mut notifiers = vec![];

        for _ in 0..NUMBER_OF_NOTIFIERS {
            notifiers.push(sut.notifier_builder().create().unwrap());
        }

        let mut notifier_details = vec![];
        sut.dynamic_config().list_notifiers(|details| {
            notifier_details.push(details.notifier_id);
            CallbackProgression::Continue
        });

        assert_that!(notifier_details, len NUMBER_OF_NOTIFIERS);
        for notifier in notifiers {
            assert_that!(notifier_details, contains notifier.id());
        }
    }

    #[test]
    fn listing_all_notifiers_stops_on_request<S: Service>() {
        const NUMBER_OF_NOTIFIERS: usize = 11;
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<S>().unwrap();

        let sut = node
            .service_builder(&service_name)
            .event()
            .max_notifiers(NUMBER_OF_NOTIFIERS)
            .create()
            .unwrap();

        let mut notifiers = vec![];

        for _ in 0..NUMBER_OF_NOTIFIERS {
            notifiers.push(sut.notifier_builder().create().unwrap());
        }

        let mut counter = 0;
        sut.dynamic_config().list_notifiers(|_| {
            counter += 1;
            CallbackProgression::Stop
        });

        assert_that!(counter, eq 1);
    }

    #[test]
    fn listing_all_listeners_works<S: Service>() {
        const NUMBER_OF_LISTENERS: usize = 14;
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<S>().unwrap();

        let sut = node
            .service_builder(&service_name)
            .event()
            .max_listeners(NUMBER_OF_LISTENERS)
            .create()
            .unwrap();

        let mut listeners = vec![];

        for _ in 0..NUMBER_OF_LISTENERS {
            listeners.push(sut.listener_builder().create().unwrap());
        }

        let mut listener_details = vec![];
        sut.dynamic_config().list_listeners(|details| {
            listener_details.push(details.listener_id);
            CallbackProgression::Continue
        });

        assert_that!(listener_details, len NUMBER_OF_LISTENERS);
        for listener in listeners {
            assert_that!(listener_details, contains listener.id());
        }
    }

    #[test]
    fn listing_all_listeners_stops_on_request<S: Service>() {
        const NUMBER_OF_LISTENERS: usize = 11;
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<S>().unwrap();

        let sut = node
            .service_builder(&service_name)
            .event()
            .max_listeners(NUMBER_OF_LISTENERS)
            .create()
            .unwrap();

        let mut listeners = vec![];

        for _ in 0..NUMBER_OF_LISTENERS {
            listeners.push(sut.listener_builder().create().unwrap());
        }

        let mut counter = 0;
        sut.dynamic_config().list_listeners(|_| {
            counter += 1;
            CallbackProgression::Stop
        });

        assert_that!(counter, eq 1);
    }

    #[test]
    fn notifier_does_not_notify_listener_from_same_node_id_when_requested<Sut: Service>() {
        let service_name = generate_name();
        let config = generate_isolated_config();
        let node_1 = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let node_2 = NodeBuilder::new().config(&config).create::<Sut>().unwrap();
        let event_id = EventId::new(23);

        let sut_1 = node_1
            .service_builder(&service_name)
            .event()
            .create()
            .unwrap();

        let sut_2 = node_2
            .service_builder(&service_name)
            .event()
            .open()
            .unwrap();

        let notifier = sut_1
            .notifier_builder()
            .default_event_id(event_id)
            .create()
            .unwrap();
        let listener_1 = sut_1.listener_builder().create().unwrap();
        let listener_2 = sut_2.listener_builder().create().unwrap();

        assert_that!(notifier.__internal_notify(event_id, true), is_ok);

        let mut received_events = 0;
        for _ in listener_1.try_wait_one().unwrap().iter() {
            received_events += 1;
        }
        assert_that!(received_events, eq 0);

        let mut received_events = 0;
        for event in listener_2.try_wait_one().unwrap().iter() {
            assert_that!(*event, eq event_id);
            received_events += 1;
        }
        assert_that!(received_events, eq 1);
    }

    #[instantiate_tests(<iceoryx2::service::ipc::Service>)]
    mod ipc {}

    #[instantiate_tests(<iceoryx2::service::local::Service>)]
    mod local {}

    #[instantiate_tests(<iceoryx2::service::ipc_threadsafe::Service>)]
    mod ipc_threadsafe {}

    #[instantiate_tests(<iceoryx2::service::local_threadsafe::Service>)]
    mod local_threadsafe {}
}
