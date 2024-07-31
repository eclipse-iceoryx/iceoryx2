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
mod node_death_tests {
    use std::sync::atomic::{AtomicU32, Ordering};

    use iceoryx2::config::Config;
    use iceoryx2::node::testing::__internal_node_staged_death;
    use iceoryx2::node::{CleanupState, NodeState};
    use iceoryx2::prelude::*;
    use iceoryx2::service::Service;
    use iceoryx2_bb_log::{set_log_level, LogLevel};
    use iceoryx2_bb_posix::unique_system_id::UniqueSystemId;
    use iceoryx2_bb_testing::watchdog::Watchdog;
    use iceoryx2_bb_testing::{assert_that, test_fail};

    fn generate_name() -> ServiceName {
        ServiceName::new(&format!(
            "node_death_tests_{}",
            UniqueSystemId::new().unwrap().value()
        ))
        .unwrap()
    }

    struct TestDetails<S: Service> {
        node: Node<S>,
    }

    trait Test {
        type Service: Service;

        fn generate_node_name(i: usize, prefix: &str) -> NodeName {
            NodeName::new(&(prefix.to_string() + &i.to_string())).unwrap()
        }

        fn create_test_node(config: &Config) -> TestDetails<Self::Service> {
            static COUNTER: AtomicU32 = AtomicU32::new(0);
            let node_name = Self::generate_node_name(0, "toby or no toby");
            let fake_node_id = ((u32::MAX - COUNTER.fetch_add(1, Ordering::Relaxed)) as u128) << 96;
            let fake_node_id =
                unsafe { core::mem::transmute::<u128, UniqueSystemId>(fake_node_id) };

            let node = unsafe {
                NodeBuilder::new()
                    .name(&node_name)
                    .config(config)
                    .__internal_create_with_custom_node_id::<Self::Service>(fake_node_id)
                    .unwrap()
            };

            TestDetails { node }
        }

        fn staged_death(node: &mut Node<Self::Service>);
    }

    struct ZeroCopy;

    impl Test for ZeroCopy {
        type Service = iceoryx2::service::ipc::Service;

        fn staged_death(node: &mut Node<Self::Service>) {
            use iceoryx2_cal::monitoring::testing::__InternalMonitoringTokenTestable;
            let monitor = unsafe { __internal_node_staged_death(node) };
            monitor.staged_death();
        }
    }

    #[test]
    fn dead_node_is_marked_as_dead_and_can_be_cleaned_up<S: Test>() {
        const NUMBER_OF_DEAD_NODES_LIMIT: usize = 5;
        let mut config = Config::global_config().clone();
        config.global.node.cleanup_dead_nodes_on_creation = false;

        for i in 1..NUMBER_OF_DEAD_NODES_LIMIT {
            for _ in 0..i {
                let mut sut = S::create_test_node(&config);
                S::staged_death(&mut sut.node);
                core::mem::forget(sut.node);
            }

            let mut node_list = vec![];
            Node::<S::Service>::list(&config, |node_state| {
                node_list.push(node_state);
                CallbackProgression::Continue
            })
            .unwrap();
            assert_that!(node_list, len i);

            for _ in 0..i {
                if let Some(NodeState::Dead(state)) = node_list.pop() {
                    assert_that!(state.remove_stale_resources(), eq Ok(true));
                } else {
                    test_fail!("all nodes shall be dead");
                }
            }

            node_list.clear();
            Node::<S::Service>::list(&config, |node_state| {
                node_list.push(node_state);
                CallbackProgression::Continue
            })
            .unwrap();
            assert_that!(node_list, len 0);
        }
    }

    #[test]
    fn dead_node_is_removed_from_pub_sub_service<S: Test>() {
        let _watchdog = Watchdog::new();
        const NUMBER_OF_BAD_NODES: usize = 3;
        const NUMBER_OF_GOOD_NODES: usize = 4;
        const NUMBER_OF_SERVICES: usize = 5;
        const NUMBER_OF_PUBLISHERS: usize = NUMBER_OF_BAD_NODES + NUMBER_OF_GOOD_NODES;
        const NUMBER_OF_SUBSCRIBERS: usize = NUMBER_OF_BAD_NODES + NUMBER_OF_GOOD_NODES;

        let mut config = Config::global_config().clone();
        config.global.node.cleanup_dead_nodes_on_creation = false;

        let mut bad_nodes = vec![];
        let mut good_nodes = vec![];

        for _ in 0..NUMBER_OF_BAD_NODES {
            bad_nodes.push(S::create_test_node(&config).node);
        }

        for _ in 0..NUMBER_OF_GOOD_NODES {
            good_nodes.push(
                NodeBuilder::new()
                    .config(&config)
                    .create::<S::Service>()
                    .unwrap(),
            );
        }

        let mut services = vec![];
        let mut bad_publishers = vec![];
        let mut bad_subscribers = vec![];
        let mut good_publishers = vec![];
        let mut good_subscribers = vec![];

        for _ in 0..NUMBER_OF_SERVICES {
            let service_name = generate_name();

            for node in &bad_nodes {
                let service = node
                    .service_builder(&service_name)
                    .publish_subscribe::<u64>()
                    .max_publishers(NUMBER_OF_PUBLISHERS)
                    .max_subscribers(NUMBER_OF_SUBSCRIBERS)
                    .open_or_create()
                    .unwrap();
                bad_publishers.push(service.publisher_builder().create().unwrap());
                bad_subscribers.push(service.subscriber_builder().create().unwrap());

                services.push(service);
            }

            for node in &good_nodes {
                let service = node
                    .service_builder(&service_name)
                    .publish_subscribe::<u64>()
                    .max_publishers(NUMBER_OF_PUBLISHERS)
                    .max_subscribers(NUMBER_OF_SUBSCRIBERS)
                    .open_or_create()
                    .unwrap();
                good_publishers.push(service.publisher_builder().create().unwrap());
                good_subscribers.push(service.subscriber_builder().create().unwrap());
                services.push(service);
            }
        }

        for _ in 0..NUMBER_OF_BAD_NODES {
            let mut node = bad_nodes.pop().unwrap();
            S::staged_death(&mut node);
        }

        core::mem::forget(bad_publishers);
        core::mem::forget(bad_subscribers);

        assert_that!(Node::<S::Service>::cleanup_dead_nodes(Config::global_config()), eq CleanupState { cleanups: NUMBER_OF_BAD_NODES, failed_cleanups: 0});

        for service in &services {
            assert_that!(service.dynamic_config().number_of_publishers(), eq NUMBER_OF_PUBLISHERS - NUMBER_OF_BAD_NODES);
            assert_that!(service.dynamic_config().number_of_subscribers(), eq NUMBER_OF_SUBSCRIBERS - NUMBER_OF_BAD_NODES);
        }
    }

    #[test]
    fn dead_node_is_removed_from_event_service<S: Test>() {
        set_log_level(LogLevel::Error);
        let _watchdog = Watchdog::new();
        const NUMBER_OF_BAD_NODES: usize = 3;
        const NUMBER_OF_GOOD_NODES: usize = 4;
        const NUMBER_OF_SERVICES: usize = 5;
        const NUMBER_OF_NOTIFIERS: usize = NUMBER_OF_BAD_NODES + NUMBER_OF_GOOD_NODES;
        const NUMBER_OF_LISTENERS: usize = NUMBER_OF_BAD_NODES + NUMBER_OF_GOOD_NODES;

        let mut config = Config::global_config().clone();
        config.global.node.cleanup_dead_nodes_on_creation = false;

        let mut bad_nodes = vec![];
        let mut good_nodes = vec![];

        for _ in 0..NUMBER_OF_BAD_NODES {
            bad_nodes.push(S::create_test_node(&config).node);
        }

        for _ in 0..NUMBER_OF_GOOD_NODES {
            good_nodes.push(
                NodeBuilder::new()
                    .config(&config)
                    .create::<S::Service>()
                    .unwrap(),
            );
        }

        let mut services = vec![];
        let mut bad_notifiers = vec![];
        let mut bad_listeners = vec![];
        let mut good_notifiers = vec![];
        let mut good_listeners = vec![];

        for _ in 0..NUMBER_OF_SERVICES {
            let service_name = generate_name();

            for node in &bad_nodes {
                let service = node
                    .service_builder(&service_name)
                    .event()
                    .max_listeners(NUMBER_OF_LISTENERS)
                    .max_notifiers(NUMBER_OF_NOTIFIERS)
                    .open_or_create()
                    .unwrap();
                bad_notifiers.push(service.notifier_builder().create().unwrap());
                bad_listeners.push(service.listener_builder().create().unwrap());

                services.push(service);
            }

            for node in &good_nodes {
                let service = node
                    .service_builder(&service_name)
                    .event()
                    .max_listeners(NUMBER_OF_LISTENERS)
                    .max_notifiers(NUMBER_OF_NOTIFIERS)
                    .open_or_create()
                    .unwrap();
                good_notifiers.push(service.notifier_builder().create().unwrap());
                good_listeners.push(service.listener_builder().create().unwrap());

                services.push(service);
            }
        }

        for _ in 0..NUMBER_OF_BAD_NODES {
            let mut node = bad_nodes.pop().unwrap();
            S::staged_death(&mut node);
        }

        core::mem::forget(bad_notifiers);
        core::mem::forget(bad_listeners);

        assert_that!(Node::<S::Service>::cleanup_dead_nodes(Config::global_config()), eq CleanupState { cleanups: NUMBER_OF_BAD_NODES, failed_cleanups: 0});

        for service in &services {
            assert_that!(service.dynamic_config().number_of_notifiers(), eq NUMBER_OF_NOTIFIERS - NUMBER_OF_BAD_NODES);
            assert_that!(service.dynamic_config().number_of_listeners(), eq NUMBER_OF_LISTENERS - NUMBER_OF_BAD_NODES);
        }
    }

    #[test]
    fn event_service_is_removed_when_last_node_dies<S: Test>() {
        let service_name = generate_name();
        let mut config = Config::global_config().clone();
        config.global.node.cleanup_dead_nodes_on_creation = false;

        let mut sut = S::create_test_node(&config).node;
        core::mem::forget(
            sut.service_builder(&service_name)
                .event()
                .open_or_create()
                .unwrap(),
        );
        S::staged_death(&mut sut);

        assert_that!(
            S::Service::list(&config, |service_details| {
                assert_that!(*service_details.static_details.name(), eq service_name);
                CallbackProgression::Continue
            }),
            is_ok
        );

        assert_that!(Node::<S::Service>::cleanup_dead_nodes(Config::global_config()), eq CleanupState { cleanups: 1, failed_cleanups: 0});

        assert_that!(
            S::Service::list(&config, |_| {
                test_fail!("after the cleanup there shall be no more services");
            }),
            is_ok
        );
    }

    #[test]
    fn pubsub_service_is_removed_when_last_node_dies<S: Test>() {
        let service_name = generate_name();
        let mut config = Config::global_config().clone();
        config.global.node.cleanup_dead_nodes_on_creation = false;

        let mut sut = S::create_test_node(&config).node;
        core::mem::forget(
            sut.service_builder(&service_name)
                .publish_subscribe::<u64>()
                .open_or_create()
                .unwrap(),
        );
        S::staged_death(&mut sut);

        assert_that!(
            S::Service::list(&config, |service_details| {
                assert_that!(*service_details.static_details.name(), eq service_name);
                CallbackProgression::Continue
            }),
            is_ok
        );

        assert_that!(Node::<S::Service>::cleanup_dead_nodes(Config::global_config()), eq CleanupState { cleanups: 1, failed_cleanups: 0});

        assert_that!(
            S::Service::list(&config, |_| {
                test_fail!("after the cleanup there shall be no more services");
            }),
            is_ok
        );
    }

    #[test]
    fn node_cleanup_option_works_on_node_creation<S: Test>() {
        let mut config = Config::global_config().clone();
        config.global.node.cleanup_dead_nodes_on_creation = false;

        let mut sut = S::create_test_node(&config);
        S::staged_death(&mut sut.node);
        core::mem::forget(sut.node);

        let number_of_nodes = || {
            let mut counter = 0;
            Node::<S::Service>::list(&config, |_| {
                counter += 1;
                CallbackProgression::Continue
            })
            .unwrap();
            counter
        };

        assert_that!(number_of_nodes(), eq 1);

        let node_without_cleanup = NodeBuilder::new()
            .config(&config)
            .create::<S::Service>()
            .unwrap();

        assert_that!(number_of_nodes(), eq 2);

        let mut config = Config::global_config().clone();
        config.global.node.cleanup_dead_nodes_on_creation = true;
        let node_with_cleanup = NodeBuilder::new()
            .config(&config)
            .create::<S::Service>()
            .unwrap();

        assert_that!(number_of_nodes(), eq 2);

        drop(node_with_cleanup);
        drop(node_without_cleanup);

        assert_that!(number_of_nodes(), eq 0);
    }

    #[test]
    fn node_cleanup_option_works_on_node_destruction<S: Test>() {
        let mut config = Config::global_config().clone();
        config.global.node.cleanup_dead_nodes_on_destruction = true;
        let node_with_cleanup = NodeBuilder::new()
            .config(&config)
            .create::<S::Service>()
            .unwrap();

        let mut config = Config::global_config().clone();
        config.global.node.cleanup_dead_nodes_on_destruction = false;
        let node_without_cleanup = NodeBuilder::new()
            .config(&config)
            .create::<S::Service>()
            .unwrap();

        let mut sut = S::create_test_node(&config);
        S::staged_death(&mut sut.node);
        core::mem::forget(sut.node);

        let number_of_nodes = || {
            let mut counter = 0;
            Node::<S::Service>::list(&config, |_| {
                counter += 1;
                CallbackProgression::Continue
            })
            .unwrap();
            counter
        };

        assert_that!(number_of_nodes(), eq 3);

        drop(node_without_cleanup);

        assert_that!(number_of_nodes(), eq 2);

        drop(node_with_cleanup);

        assert_that!(number_of_nodes(), eq 0);
    }

    #[instantiate_tests(<ZeroCopy>)]
    mod ipc {}
}
