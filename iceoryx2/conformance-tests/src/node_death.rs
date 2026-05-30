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

extern crate alloc;

use core::marker::PhantomData;
use core::time::Duration;

use alloc::vec;
use alloc::vec::Vec;
use iceoryx2::config::Config;
use iceoryx2::node::{CleanupState, NodeState};
use iceoryx2::prelude::*;
use iceoryx2::testing::*;
use iceoryx2_bb_elementary_traits::testing::abandonable::Abandonable;
use iceoryx2_bb_testing::watchdog::Watchdog;
use iceoryx2_bb_testing::{assert_that, test_fail, test_requires};
use iceoryx2_bb_testing_macros::conformance_test;
use iceoryx2_bb_testing_macros::conformance_tests;
use iceoryx2_cal::dynamic_storage::DynamicStorage;

pub struct Test<Service: iceoryx2::service::Service> {
    config: Config,
    _watchdog: Watchdog,
    _data: PhantomData<Service>,
}

impl<Service: iceoryx2::service::Service> Drop for Test<Service> {
    fn drop(&mut self) {
        Self::cleanup_dead_nodes(&self.config);
        unsafe { remove_global_mgmt_segment::<Service>(&self.config).unwrap() };
    }
}

impl<Service: iceoryx2::service::Service> Test<Service> {
    fn new() -> Self {
        let _watchdog = Watchdog::new();
        let mut config = generate_isolated_config();
        config.global.node.cleanup_dead_nodes_on_creation = false;
        config.global.node.cleanup_dead_nodes_on_destruction = false;
        config.global.service.cleanup_dead_nodes_on_open = false;

        Self {
            config,
            _watchdog,
            _data: PhantomData,
        }
    }

    fn config(&self) -> &Config {
        &self.config
    }

    fn config_mut(&mut self) -> &mut Config {
        &mut self.config
    }

    fn abandon_contents<T: Abandonable>(&self, mut contents: Vec<T>) {
        while let Some(element) = contents.pop() {
            T::abandon(element);
        }
    }

    fn create_node(&self) -> Node<Service> {
        NodeBuilder::new()
            .config(self.config())
            .create::<Service>()
            .unwrap()
    }

    fn number_of_nodes(&self) -> usize {
        self.list_nodes().len()
    }

    fn list_nodes(&self) -> Vec<NodeState<Service>> {
        let mut node_list = vec![];
        Node::<Service>::list(self.config(), |node_state| {
            node_list.push(node_state);
            CallbackProgression::Continue
        })
        .unwrap();

        node_list
    }

    fn cleanup_dead_nodes(config: &Config) {
        Node::<Service>::list(config, |node_state| {
            if let NodeState::Dead(state) = node_state {
                state
                    .blocking_remove_stale_resources(Duration::MAX)
                    .unwrap();
            }

            CallbackProgression::Continue
        })
        .unwrap();
    }
}

#[allow(clippy::module_inception)]
#[conformance_tests]
pub mod node_death {
    use iceoryx2_bb_elementary_traits::testing::abandonable::Abandonable;
    use iceoryx2_bb_posix::unique_system_id::UniqueSystemId;
    use iceoryx2_cal::static_storage::StaticStorage;

    use super::*;

    fn does_support_persistency<S: iceoryx2::service::Service>() -> bool {
        <S as Service>::DynamicStorage::<
                    iceoryx2::service::dynamic_config::DynamicConfig,
                >::does_support_persistency()
    }

    #[conformance_test]
    pub fn dead_node_is_marked_as_dead_and_can_be_cleaned_up<S: iceoryx2::service::Service>() {
        let test = Test::<S>::new();

        const NUMBER_OF_DEAD_NODES_LIMIT: usize = 5;

        for i in 1..NUMBER_OF_DEAD_NODES_LIMIT {
            for _ in 0..i {
                let sut = test.create_node();
                sut.abandon();
            }

            let mut node_list = test.list_nodes();
            assert_that!(node_list, len i);

            for _ in 0..i {
                if let Some(NodeState::Dead(state)) = node_list.pop() {
                    let result = state.try_remove_stale_resources();
                    assert_that!(result, is_ok);
                } else {
                    test_fail!("all nodes shall be dead");
                }
            }

            assert_that!(test.number_of_nodes(), eq 0);
        }
    }

    #[conformance_test]
    pub fn dead_node_is_removed_from_pub_sub_service<S: iceoryx2::service::Service>() {
        test_requires!(does_support_persistency::<S>());

        let test = Test::<S>::new();

        const NUMBER_OF_BAD_NODES: usize = 3;
        const NUMBER_OF_GOOD_NODES: usize = 4;
        const NUMBER_OF_SERVICES: usize = 5;
        const NUMBER_OF_PUBLISHERS: usize = NUMBER_OF_BAD_NODES + NUMBER_OF_GOOD_NODES;
        const NUMBER_OF_SUBSCRIBERS: usize = NUMBER_OF_BAD_NODES + NUMBER_OF_GOOD_NODES;

        let mut bad_nodes = vec![];
        let mut good_nodes = vec![];

        for _ in 0..NUMBER_OF_BAD_NODES {
            bad_nodes.push(test.create_node());
        }

        for _ in 0..NUMBER_OF_GOOD_NODES {
            good_nodes.push(test.create_node());
        }

        let mut bad_services = vec![];
        let mut bad_publishers = vec![];
        let mut bad_subscribers = vec![];
        let mut good_publishers = vec![];
        let mut good_subscribers = vec![];
        let mut good_services = vec![];

        for _ in 0..NUMBER_OF_SERVICES {
            let service_name = generate_service_name();

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

                bad_services.push(service);
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
                good_services.push(service);
            }
        }

        test.abandon_contents(bad_nodes);
        test.abandon_contents(bad_services);
        test.abandon_contents(bad_publishers);
        test.abandon_contents(bad_subscribers);

        assert_that!(Node::<S>::try_cleanup_dead_nodes(test.config()), eq CleanupState { cleanups: NUMBER_OF_BAD_NODES as _, failed_cleanups: 0});

        for service in &good_services {
            assert_that!(service.dynamic_config().number_of_publishers(), eq NUMBER_OF_PUBLISHERS - NUMBER_OF_BAD_NODES);
            assert_that!(service.dynamic_config().number_of_subscribers(), eq NUMBER_OF_SUBSCRIBERS - NUMBER_OF_BAD_NODES);
        }
    }

    #[conformance_test]
    pub fn dead_node_is_removed_from_event_service<S: iceoryx2::service::Service>() {
        let test = Test::<S>::new();

        const NUMBER_OF_BAD_NODES: usize = 3;
        const NUMBER_OF_GOOD_NODES: usize = 4;
        const NUMBER_OF_SERVICES: usize = 5;
        const NUMBER_OF_NOTIFIERS: usize = NUMBER_OF_BAD_NODES + NUMBER_OF_GOOD_NODES;
        const NUMBER_OF_LISTENERS: usize = NUMBER_OF_BAD_NODES + NUMBER_OF_GOOD_NODES;

        let mut bad_nodes = vec![];
        let mut good_nodes = vec![];

        for _ in 0..NUMBER_OF_BAD_NODES {
            bad_nodes.push(test.create_node());
        }

        for _ in 0..NUMBER_OF_GOOD_NODES {
            good_nodes.push(test.create_node());
        }

        let mut bad_services = vec![];
        let mut bad_notifiers = vec![];
        let mut bad_listeners = vec![];
        let mut good_services = vec![];
        let mut good_notifiers = vec![];
        let mut good_listeners = vec![];

        for _ in 0..NUMBER_OF_SERVICES {
            let service_name = generate_service_name();

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
                bad_services.push(service);
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
                good_services.push(service);
            }
        }

        test.abandon_contents(bad_nodes);
        test.abandon_contents(bad_notifiers);
        test.abandon_contents(bad_listeners);
        test.abandon_contents(bad_services);

        assert_that!(Node::<S>::try_cleanup_dead_nodes(test.config()), eq CleanupState { cleanups: NUMBER_OF_BAD_NODES as _, failed_cleanups: 0});

        for service in &good_services {
            assert_that!(service.dynamic_config().number_of_notifiers(), eq NUMBER_OF_NOTIFIERS - NUMBER_OF_BAD_NODES);
            assert_that!(service.dynamic_config().number_of_listeners(), eq NUMBER_OF_LISTENERS - NUMBER_OF_BAD_NODES);
        }
    }

    #[conformance_test]
    pub fn notifier_of_dead_node_emits_death_event_when_configured<
        S: iceoryx2::service::Service,
    >() {
        let test = Test::<S>::new();

        let service_name = generate_service_name();
        let notifier_dead_event = EventId::new(8);

        let dead_node = test.create_node();
        let node = test.create_node();

        let dead_service = dead_node
            .service_builder(&service_name)
            .event()
            .notifier_dead_event(notifier_dead_event)
            .notifier_created_event(EventId::new(0))
            .notifier_dropped_event(EventId::new(0))
            .create()
            .unwrap();
        let dead_notifier = dead_service.notifier_builder().create().unwrap();

        let service = node.service_builder(&service_name).event().open().unwrap();
        let listener = service.listener_builder().create().unwrap();

        dead_node.abandon();
        dead_notifier.abandon();
        dead_service.abandon();

        assert_that!(Node::<S>::try_cleanup_dead_nodes(test.config()), eq CleanupState { cleanups: 1, failed_cleanups: 0});

        let mut received_events = 0;
        listener
            .try_wait_all(|event| {
                assert_that!(event, eq notifier_dead_event);
                received_events += 1;
            })
            .unwrap();

        assert_that!(received_events, eq 1);
    }

    #[conformance_test]
    pub fn dead_node_is_removed_from_request_response_service<S: iceoryx2::service::Service>() {
        test_requires!(does_support_persistency::<S>());

        let test = Test::<S>::new();

        const NUMBER_OF_BAD_NODES: usize = 2;
        const NUMBER_OF_GOOD_NODES: usize = 3;
        const NUMBER_OF_SERVICES: usize = 4;
        const NUMBER_OF_CLIENTS: usize = NUMBER_OF_BAD_NODES + NUMBER_OF_GOOD_NODES;
        const NUMBER_OF_SERVERS: usize = NUMBER_OF_BAD_NODES + NUMBER_OF_GOOD_NODES;

        let mut bad_nodes = vec![];
        let mut bad_services = vec![];
        let mut bad_clients = vec![];
        let mut bad_servers = vec![];
        let mut good_services = vec![];
        let mut good_clients = vec![];
        let mut good_servers = vec![];
        let mut good_nodes = vec![];

        for _ in 0..NUMBER_OF_BAD_NODES {
            bad_nodes.push(test.create_node());
        }

        for _ in 0..NUMBER_OF_GOOD_NODES {
            good_nodes.push(test.create_node());
        }

        for _ in 0..NUMBER_OF_SERVICES {
            let service_name = generate_service_name();

            for node in &bad_nodes {
                let service = node
                    .service_builder(&service_name)
                    .request_response::<u64, u64>()
                    .max_clients(NUMBER_OF_CLIENTS)
                    .max_servers(NUMBER_OF_SERVERS)
                    .open_or_create()
                    .unwrap();
                bad_clients.push(service.client_builder().create().unwrap());
                bad_servers.push(service.server_builder().create().unwrap());
                bad_services.push(service);
            }

            for node in &good_nodes {
                let service = node
                    .service_builder(&service_name)
                    .request_response::<u64, u64>()
                    .max_clients(NUMBER_OF_CLIENTS)
                    .max_servers(NUMBER_OF_SERVERS)
                    .open_or_create()
                    .unwrap();
                good_clients.push(service.client_builder().create().unwrap());
                good_servers.push(service.server_builder().create().unwrap());
                good_services.push(service);
            }
        }

        test.abandon_contents(bad_nodes);
        test.abandon_contents(bad_services);
        test.abandon_contents(bad_servers);
        test.abandon_contents(bad_clients);

        assert_that!(Node::<S>::try_cleanup_dead_nodes(test.config()), eq CleanupState { cleanups: NUMBER_OF_BAD_NODES as _, failed_cleanups: 0});

        for service in &good_services {
            assert_that!(service.dynamic_config().number_of_clients(), eq NUMBER_OF_CLIENTS - NUMBER_OF_BAD_NODES);
            assert_that!(service.dynamic_config().number_of_servers(), eq NUMBER_OF_SERVERS - NUMBER_OF_BAD_NODES);
        }
    }

    #[conformance_test]
    pub fn dead_node_is_removed_from_blackboard_service<S: iceoryx2::service::Service>() {
        let test = Test::<S>::new();

        const NUMBER_OF_BAD_NODES: usize = 3;
        const NUMBER_OF_GOOD_NODES: usize = 4;
        const NUMBER_OF_SERVICES: usize = 5;
        const NUMBER_OF_READERS: usize = NUMBER_OF_BAD_NODES + NUMBER_OF_GOOD_NODES;

        let mut bad_services = vec![];
        let mut bad_nodes = vec![];
        let mut bad_readers = vec![];
        let mut good_services = vec![];
        let mut good_nodes = vec![];
        let mut good_readers = vec![];

        for _ in 0..NUMBER_OF_BAD_NODES {
            bad_nodes.push(test.create_node());
        }

        for _ in 0..NUMBER_OF_GOOD_NODES {
            good_nodes.push(test.create_node());
        }

        for _ in 0..NUMBER_OF_SERVICES {
            let service_name = generate_service_name();
            let _service = bad_nodes[0]
                .service_builder(&service_name)
                .blackboard_creator::<u64>()
                .max_readers(NUMBER_OF_READERS)
                .add_with_default::<u64>(0)
                .create()
                .unwrap();

            for node in &bad_nodes {
                let service = node
                    .service_builder(&service_name)
                    .blackboard_opener::<u64>()
                    .max_readers(NUMBER_OF_READERS)
                    .open()
                    .unwrap();
                bad_readers.push(service.reader_builder().create().unwrap());
                bad_services.push(service);
            }

            for node in &good_nodes {
                let service = node
                    .service_builder(&service_name)
                    .blackboard_opener::<u64>()
                    .max_readers(NUMBER_OF_READERS)
                    .open()
                    .unwrap();
                good_readers.push(service.reader_builder().create().unwrap());
                good_services.push(service);
            }
        }

        test.abandon_contents(bad_nodes);
        test.abandon_contents(bad_services);
        test.abandon_contents(bad_readers);

        assert_that!(Node::<S>::try_cleanup_dead_nodes(test.config()), eq CleanupState { cleanups: NUMBER_OF_BAD_NODES as _, failed_cleanups: 0});

        for service in &good_services {
            assert_that!(service.dynamic_config().number_of_readers(), eq NUMBER_OF_READERS - NUMBER_OF_BAD_NODES);
        }
    }

    #[conformance_test]
    pub fn opened_blackboard_can_be_accessed_after_creator_node_crash<
        S: iceoryx2::service::Service,
    >() {
        let test = Test::<S>::new();
        let service_name = generate_service_name();

        let bad_node = test.create_node();
        let bad_service = bad_node
            .service_builder(&service_name)
            .blackboard_creator::<u64>()
            .add_with_default::<u64>(0)
            .create()
            .unwrap();
        let bad_writer = bad_service.writer_builder().create().unwrap();

        let good_node = NodeBuilder::new()
            .config(test.config())
            .create::<S>()
            .unwrap();
        let good_service = good_node
            .service_builder(&service_name)
            .blackboard_opener::<u64>()
            .open()
            .unwrap();
        let reader = good_service.reader_builder().create().unwrap();

        bad_node.abandon();
        bad_writer.abandon();
        bad_service.abandon();

        assert_that!(Node::<S>::try_cleanup_dead_nodes(test.config()), eq CleanupState { cleanups: 1, failed_cleanups: 0});

        assert_that!(good_service.dynamic_config().number_of_readers(), eq 1);
        assert_that!(good_service.dynamic_config().number_of_writers(), eq 0);
        assert_that!(*reader.entry::<u64>(&0).unwrap().get(), eq 0);

        let writer = good_service.writer_builder().create().unwrap();
        let entry_handle_mut = writer.entry::<u64>(&0).unwrap();
        entry_handle_mut.update_with_copy(1);

        assert_that!(good_service.dynamic_config().number_of_readers(), eq 1);
        assert_that!(good_service.dynamic_config().number_of_writers(), eq 1);
        assert_that!(*reader.entry::<u64>(&0).unwrap().get(), eq 1);
    }

    #[conformance_test]
    pub fn event_service_is_removed_when_last_node_dies<S: iceoryx2::service::Service>() {
        let test = Test::<S>::new();
        let service_name = generate_service_name();

        let bad_node = test.create_node();
        let bad_service = bad_node
            .service_builder(&service_name)
            .event()
            .open_or_create()
            .unwrap();
        bad_node.abandon();
        bad_service.abandon();

        assert_that!(
            S::list(test.config(), |service_details| {
                assert_that!(*service_details.static_details.name(), eq service_name);
                CallbackProgression::Continue
            }),
            is_ok
        );

        assert_that!(Node::<S>::try_cleanup_dead_nodes(test.config()), eq CleanupState { cleanups: 1, failed_cleanups: 0});

        assert_that!(
            S::list(test.config(), |_| {
                test_fail!("after the cleanup there shall be no more services");
            }),
            is_ok
        );
    }

    #[conformance_test]
    pub fn pubsub_service_is_removed_when_last_node_dies<S: iceoryx2::service::Service>() {
        let test = Test::<S>::new();
        let service_name = generate_service_name();

        let bad_node = test.create_node();
        let bad_service = bad_node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .open_or_create()
            .unwrap();
        bad_node.abandon();
        bad_service.abandon();

        assert_that!(
            S::list(test.config(), |service_details| {
                assert_that!(*service_details.static_details.name(), eq service_name);
                CallbackProgression::Continue
            }),
            is_ok
        );

        assert_that!(Node::<S>::try_cleanup_dead_nodes(test.config()), eq CleanupState { cleanups: 1, failed_cleanups: 0});

        assert_that!(
            S::list(test.config(), |_| {
                test_fail!("after the cleanup there shall be no more services");
            }),
            is_ok
        );
    }

    #[conformance_test]
    pub fn request_response_service_is_removed_when_last_node_dies<
        S: iceoryx2::service::Service,
    >() {
        let test = Test::<S>::new();
        let service_name = generate_service_name();

        let bad_node = test.create_node();
        let bad_service = bad_node
            .service_builder(&service_name)
            .request_response::<u64, u64>()
            .open_or_create()
            .unwrap();
        bad_node.abandon();
        bad_service.abandon();

        assert_that!(
            S::list(test.config(), |service_details| {
                assert_that!(*service_details.static_details.name(), eq service_name);
                CallbackProgression::Continue
            }),
            is_ok
        );

        assert_that!(Node::<S>::try_cleanup_dead_nodes(test.config()), eq CleanupState { cleanups: 1, failed_cleanups: 0});

        assert_that!(
            S::list(test.config(), |_| {
                test_fail!("after the cleanup there shall be no more services");
            }),
            is_ok
        );
    }

    #[conformance_test]
    pub fn blackboard_service_is_removed_when_last_node_dies<S: iceoryx2::service::Service>() {
        let test = Test::<S>::new();
        let service_name = generate_service_name();

        let bad_node = test.create_node();
        let bad_service = bad_node
            .service_builder(&service_name)
            .blackboard_creator::<u64>()
            .add_with_default::<u64>(0)
            .create()
            .unwrap();
        bad_node.abandon();
        bad_service.abandon();

        assert_that!(
            S::list(test.config(), |service_details| {
                assert_that!(*service_details.static_details.name(), eq service_name);
                CallbackProgression::Continue
            }),
            is_ok
        );

        assert_that!(Node::<S>::try_cleanup_dead_nodes(test.config()), eq CleanupState { cleanups: 1, failed_cleanups: 0});

        assert_that!(
            S::list(test.config(), |_| {
                test_fail!("after the cleanup there shall be no more services");
            }),
            is_ok
        );
    }

    #[conformance_test]
    pub fn writer_and_reader_resources_are_removed_after_crash<S: iceoryx2::service::Service>() {
        let test = Test::<S>::new();
        let service_name = generate_service_name();

        let good_node = test.create_node();
        let good_service = good_node
            .service_builder(&service_name)
            .blackboard_creator::<u64>()
            .max_readers(1)
            .add_with_default::<u64>(0)
            .create()
            .unwrap();

        let bad_node = test.create_node();
        let bad_service = bad_node
            .service_builder(&service_name)
            .blackboard_opener::<u64>()
            .open()
            .unwrap();
        let bad_writer = bad_service.writer_builder().create().unwrap();
        let bad_reader = bad_service.reader_builder().create().unwrap();

        bad_node.abandon();
        bad_writer.abandon();
        bad_reader.abandon();
        bad_service.abandon();

        assert_that!(Node::<S>::try_cleanup_dead_nodes(test.config()), eq CleanupState { cleanups: 1, failed_cleanups: 0});

        let writer = good_service.writer_builder().create();
        assert_that!(writer, is_ok);
        let reader = good_service.reader_builder().create();
        assert_that!(reader, is_ok);
    }

    // test disabled on Windows as the state files cannot be removed after simulated node death
    #[cfg(not(target_os = "windows"))]
    #[conformance_test]
    pub fn blackboard_resources_are_removed_when_key_has_user_defined_name<
        S: iceoryx2::service::Service,
    >() {
        let test = Test::<S>::new();
        let service_name = generate_service_name();

        #[repr(C)]
        #[derive(ZeroCopySend, Debug, Clone, Copy, PartialEq, Eq, Hash)]
        #[type_name("SoSpecial")]
        struct SpecialKey(u64);

        let bad_node = test.create_node();
        let bad_service = bad_node
            .service_builder(&service_name)
            .blackboard_creator::<SpecialKey>()
            .add_with_default::<u64>(SpecialKey(0))
            .create()
            .unwrap();
        bad_node.abandon();
        bad_service.abandon();

        assert_that!(
            S::list(test.config(), |service_details| {
                assert_that!(*service_details.static_details.name(), eq service_name);
                CallbackProgression::Continue
            }),
            is_ok
        );

        assert_that!(Node::<S>::try_cleanup_dead_nodes(test.config()), eq CleanupState { cleanups: 1, failed_cleanups: 0});

        assert_that!(
            S::list(test.config(), |_| {
                test_fail!("after the cleanup there shall be no more services");
            }),
            is_ok
        );

        let node = test.create_node();
        let service = node
            .service_builder(&service_name)
            .blackboard_creator::<SpecialKey>()
            .add_with_default::<u64>(SpecialKey(0))
            .create();
        assert_that!(service, is_ok);
    }

    // test disabled on Windows as the state files cannot be removed after simulated node death
    #[cfg(not(target_os = "windows"))]
    #[conformance_test]
    pub fn blackboard_resources_are_removed_when_last_node_dies<S: iceoryx2::service::Service>() {
        let test = Test::<S>::new();
        let service_name = generate_service_name();

        let bad_node = test.create_node();
        let bad_service = bad_node
            .service_builder(&service_name)
            .blackboard_creator::<u64>()
            .add_with_default::<u64>(0)
            .create()
            .unwrap();
        bad_node.abandon();
        bad_service.abandon();

        assert_that!(
            S::list(test.config(), |service_details| {
                assert_that!(*service_details.static_details.name(), eq service_name);
                CallbackProgression::Continue
            }),
            is_ok
        );

        assert_that!(Node::<S>::try_cleanup_dead_nodes(test.config()), eq CleanupState { cleanups: 1, failed_cleanups: 0});

        assert_that!(
            S::list(test.config(), |_| {
                test_fail!("after the cleanup there shall be no more services");
            }),
            is_ok
        );

        let node = test.create_node();
        let service = node
            .service_builder(&service_name)
            .blackboard_creator::<u64>()
            .add_with_default::<u64>(0)
            .create();
        assert_that!(service, is_ok);
    }

    #[conformance_test]
    pub fn node_cleanup_option_works_on_node_creation<S: iceoryx2::service::Service>() {
        let test = Test::<S>::new();

        let bad_node = test.create_node();
        bad_node.abandon();

        assert_that!(test.number_of_nodes(), eq 1);

        let node_without_cleanup = test.create_node();

        assert_that!(test.number_of_nodes(), eq 2);

        let mut config = test.config().clone();
        config.global.node.cleanup_dead_nodes_on_creation = true;
        let node_with_cleanup = NodeBuilder::new().config(&config).create::<S>().unwrap();

        assert_that!(test.number_of_nodes(), eq 2);

        drop(node_with_cleanup);
        drop(node_without_cleanup);

        assert_that!(test.number_of_nodes(), eq 0);
    }

    #[conformance_test]
    pub fn node_cleanup_option_works_on_node_destruction<S: iceoryx2::service::Service>() {
        let mut test = Test::<S>::new();
        test.config_mut()
            .global
            .node
            .cleanup_dead_nodes_on_destruction = true;

        let node_with_cleanup = test.create_node();

        let mut config = test.config().clone();
        config.global.node.cleanup_dead_nodes_on_destruction = false;
        let node_without_cleanup = NodeBuilder::new().config(&config).create::<S>().unwrap();

        let bad_node = test.create_node();
        bad_node.abandon();

        assert_that!(test.number_of_nodes(), eq 3);

        drop(node_without_cleanup);

        assert_that!(test.number_of_nodes(), eq 2);

        drop(node_with_cleanup);

        assert_that!(test.number_of_nodes(), eq 0);
    }

    pub fn node_cleanup_on_service_connection_works<
        S: iceoryx2::service::Service,
        T: Abandonable,
        F: FnMut(&Node<S>) -> T,
    >(
        test: Test<S>,
        total_number_of_nodes: usize,
        mut service_builder: F,
    ) {
        test_requires!(does_support_persistency::<S>());

        let bad_node = test.create_node();
        let bad_service = service_builder(&bad_node);
        bad_node.abandon();
        bad_service.abandon();

        let sut = test.create_node();
        let _service = service_builder(&sut);

        let number_of_nodes = test.number_of_nodes();
        Test::<S>::cleanup_dead_nodes(test.config());
        assert_that!(number_of_nodes, eq total_number_of_nodes);
    }

    #[conformance_test]
    pub fn publish_subscribe_node_cleanup_on_open_works_when_enabled<
        S: iceoryx2::service::Service,
    >() {
        let mut test = Test::<S>::new();
        test.config_mut().global.service.cleanup_dead_nodes_on_open = true;
        const NUMBER_OF_CONNECTED_NODES_AFTER_CONNECTION: usize = 1;
        let service_name = generate_service_name();

        node_cleanup_on_service_connection_works::<S, _, _>(
            test,
            NUMBER_OF_CONNECTED_NODES_AFTER_CONNECTION,
            |sut| {
                sut.service_builder(&service_name)
                    .publish_subscribe::<u64>()
                    .open_or_create()
                    .unwrap()
            },
        );
    }

    #[conformance_test]
    pub fn publish_subscribe_no_node_cleanup_on_open_when_disabled<
        S: iceoryx2::service::Service,
    >() {
        let test = Test::<S>::new();
        const NUMBER_OF_CONNECTED_NODES_AFTER_CONNECTION: usize = 2;
        let service_name = generate_service_name();

        node_cleanup_on_service_connection_works::<S, _, _>(
            test,
            NUMBER_OF_CONNECTED_NODES_AFTER_CONNECTION,
            |sut| {
                sut.service_builder(&service_name)
                    .publish_subscribe::<u64>()
                    .open_or_create()
                    .unwrap()
            },
        );
    }

    #[conformance_test]
    pub fn request_response_node_cleanup_on_open_works_when_enabled<
        S: iceoryx2::service::Service,
    >() {
        let mut test = Test::<S>::new();
        test.config_mut().global.service.cleanup_dead_nodes_on_open = true;
        const NUMBER_OF_CONNECTED_NODES_AFTER_CONNECTION: usize = 1;
        let service_name = generate_service_name();

        node_cleanup_on_service_connection_works::<S, _, _>(
            test,
            NUMBER_OF_CONNECTED_NODES_AFTER_CONNECTION,
            |sut| {
                sut.service_builder(&service_name)
                    .request_response::<u64, u64>()
                    .open_or_create()
                    .unwrap()
            },
        );
    }

    #[conformance_test]
    pub fn request_response_no_node_cleanup_on_open_when_disabled<S: iceoryx2::service::Service>() {
        let test = Test::<S>::new();
        const NUMBER_OF_CONNECTED_NODES_AFTER_CONNECTION: usize = 2;
        let service_name = generate_service_name();

        node_cleanup_on_service_connection_works::<S, _, _>(
            test,
            NUMBER_OF_CONNECTED_NODES_AFTER_CONNECTION,
            |sut| {
                sut.service_builder(&service_name)
                    .request_response::<u64, u64>()
                    .open_or_create()
                    .unwrap()
            },
        );
    }

    #[conformance_test]
    pub fn event_node_cleanup_on_open_works_when_enabled<S: iceoryx2::service::Service>() {
        let mut test = Test::<S>::new();
        test.config_mut().global.service.cleanup_dead_nodes_on_open = true;
        const NUMBER_OF_CONNECTED_NODES_AFTER_CONNECTION: usize = 1;
        let service_name = generate_service_name();

        node_cleanup_on_service_connection_works::<S, _, _>(
            test,
            NUMBER_OF_CONNECTED_NODES_AFTER_CONNECTION,
            |sut| {
                sut.service_builder(&service_name)
                    .event()
                    .open_or_create()
                    .unwrap()
            },
        );
    }

    #[conformance_test]
    pub fn event_no_node_cleanup_on_open_when_disabled<S: iceoryx2::service::Service>() {
        let test = Test::<S>::new();
        const NUMBER_OF_CONNECTED_NODES_AFTER_CONNECTION: usize = 2;
        let service_name = generate_service_name();

        node_cleanup_on_service_connection_works::<S, _, _>(
            test,
            NUMBER_OF_CONNECTED_NODES_AFTER_CONNECTION,
            |sut| {
                sut.service_builder(&service_name)
                    .event()
                    .open_or_create()
                    .unwrap()
            },
        );
    }

    #[conformance_test]
    pub fn blackboard_node_cleanup_on_open_works_when_enabled<S: iceoryx2::service::Service>() {
        let mut test = Test::<S>::new();
        test.config_mut().global.service.cleanup_dead_nodes_on_open = true;
        const NUMBER_OF_CONNECTED_NODES_AFTER_CONNECTION: usize = 2;
        let service_name = generate_service_name();

        let sut = test.create_node();
        let _service = sut
            .service_builder(&service_name)
            .blackboard_creator::<u64>()
            .add_with_default::<u64>(0)
            .create()
            .unwrap();

        node_cleanup_on_service_connection_works::<S, _, _>(
            test,
            NUMBER_OF_CONNECTED_NODES_AFTER_CONNECTION,
            |sut| {
                sut.service_builder(&service_name)
                    .blackboard_opener::<u64>()
                    .open()
                    .unwrap()
            },
        );
    }

    #[conformance_test]
    pub fn blackboard_no_node_cleanup_on_open_when_disabled<S: iceoryx2::service::Service>() {
        let test = Test::<S>::new();
        const NUMBER_OF_CONNECTED_NODES_AFTER_CONNECTION: usize = 3;
        let service_name = generate_service_name();

        let sut = test.create_node();
        let _service = sut
            .service_builder(&service_name)
            .blackboard_creator::<u64>()
            .add_with_default::<u64>(0)
            .create()
            .unwrap();

        node_cleanup_on_service_connection_works::<S, _, _>(
            test,
            NUMBER_OF_CONNECTED_NODES_AFTER_CONNECTION,
            |sut| {
                sut.service_builder(&service_name)
                    .blackboard_opener::<u64>()
                    .open()
                    .unwrap()
            },
        );
    }

    #[conformance_test]
    pub fn dead_node_is_removed_when_stale_service_tags_are_present<
        S: iceoryx2::service::Service,
    >() {
        const MAX_NUMBER_OF_TAGS: usize = 10;

        let test = Test::<S>::new();
        let config = test.config().clone();

        for number_of_service_tags in 1..=MAX_NUMBER_OF_TAGS {
            let sut = test.create_node();
            let node_id = *sut.id();

            let mut service_tags = vec![];

            for _ in 0..number_of_service_tags {
                let service_name = generate_service_name();
                let service_hash =
                    generate_service_hash::<S>(&service_name, MessagingPattern::PublishSubscribe);
                create_service_tag(&sut, &service_hash)
                    .unwrap()
                    .unwrap()
                    .release_ownership();
                service_tags.push((service_hash, config.clone(), node_id));
            }

            sut.abandon();

            let cleanup_results = Node::<S>::try_cleanup_dead_nodes(&config);
            assert_that!(cleanup_results.cleanups, eq 1);
            assert_that!(cleanup_results.failed_cleanups, eq 0);

            let node_state = get_node_state::<S>(&node_id, &config).unwrap();
            assert_that!(node_state, is_none);

            for tag in service_tags {
                assert_that!(does_service_tag_exist::<S>(&tag.0, &tag.1, &tag.2).unwrap(), eq false);
            }
        }
    }

    #[conformance_test]
    pub fn dead_node_is_removed_when_stale_port_tags_are_present<S: iceoryx2::service::Service>() {
        const MAX_NUMBER_OF_TAGS: usize = 1;

        let test = Test::<S>::new();
        let config = test.config().clone();

        for number_of_service_tags in 1..=MAX_NUMBER_OF_TAGS {
            let sut = test.create_node();
            let node_id = *sut.id();

            let mut port_tags = vec![];

            for _ in 0..number_of_service_tags {
                let port_id = UniqueSystemId::new().unwrap().value();
                create_port_tag(&sut, port_id).unwrap().release_ownership();
                port_tags.push((port_id, config.clone(), node_id));
            }

            sut.abandon();

            let cleanup_results = Node::<S>::try_cleanup_dead_nodes(&config);
            assert_that!(cleanup_results.cleanups, eq 1);
            assert_that!(cleanup_results.failed_cleanups, eq 0);

            let node_state = get_node_state::<S>(&node_id, &config).unwrap();
            assert_that!(node_state, is_none);
            assert_that!(test.number_of_nodes(), eq 0);

            for tag in port_tags {
                assert_that!(does_port_tag_exist::<S>(tag.0, &tag.1, &tag.2).unwrap(), eq false);
            }
        }
    }

    #[conformance_test]
    pub fn many_dead_nodes_with_many_stale_tags_are_cleaned_up<S: iceoryx2::service::Service>() {
        const NUMBER_OF_NODES: usize = 4;
        const MAX_NUMBER_OF_TAGS: usize = 4;

        let test = Test::<S>::new();
        let config = test.config().clone();

        for number_of_tags in 1..=MAX_NUMBER_OF_TAGS {
            let mut port_tags = vec![];
            let mut service_tags = vec![];
            let mut node_ids = vec![];
            for _ in 0..NUMBER_OF_NODES {
                let sut = test.create_node();
                let node_id = *sut.id();
                node_ids.push(node_id);

                for _ in 0..number_of_tags {
                    let service_name = generate_service_name();
                    let service_hash = generate_service_hash::<S>(
                        &service_name,
                        MessagingPattern::PublishSubscribe,
                    );
                    create_service_tag(&sut, &service_hash)
                        .unwrap()
                        .unwrap()
                        .release_ownership();
                    service_tags.push((service_hash, config.clone(), node_id));

                    let port_id = UniqueSystemId::new().unwrap().value();
                    create_port_tag(&sut, port_id).unwrap().release_ownership();
                    port_tags.push((port_id, config.clone(), node_id));
                }

                sut.abandon();
            }

            let cleanup_results = Node::<S>::try_cleanup_dead_nodes(&config);
            assert_that!(cleanup_results.cleanups, eq NUMBER_OF_NODES as u64);
            assert_that!(cleanup_results.failed_cleanups, eq 0);
            assert_that!(test.number_of_nodes(), eq 0);

            for node_id in node_ids {
                let node_state = get_node_state::<S>(&node_id, &config).unwrap();
                assert_that!(node_state, is_none);
            }

            for tag in service_tags {
                assert_that!(does_service_tag_exist::<S>(&tag.0, &tag.1, &tag.2).unwrap(), eq false);
            }

            for tag in port_tags {
                assert_that!(does_port_tag_exist::<S>(tag.0, &tag.1, &tag.2).unwrap(), eq false);
            }
        }
    }

    #[conformance_test]
    pub fn dead_node_is_removed_from_service_when_stale_tags_are_present<
        S: iceoryx2::service::Service,
    >() {
        const MAX_NUMBER_OF_TAGS: usize = 10;

        let test = Test::<S>::new();
        let config = test.config().clone();

        for number_of_service_tags in 1..=MAX_NUMBER_OF_TAGS {
            let bad_node = test.create_node();
            let good_node = test.create_node();
            let service_name = generate_service_name();
            let bad_service = bad_node
                .service_builder(&service_name)
                .event()
                .create()
                .unwrap();
            let good_service = good_node
                .service_builder(&service_name)
                .event()
                .open()
                .unwrap();
            let node_id = *bad_node.id();

            let mut service_tags = vec![];
            let mut port_tags = vec![];

            for _ in 0..number_of_service_tags {
                let service_name = generate_service_name();
                let service_hash =
                    generate_service_hash::<S>(&service_name, MessagingPattern::PublishSubscribe);
                create_service_tag(&bad_node, &service_hash)
                    .unwrap()
                    .unwrap()
                    .release_ownership();
                service_tags.push((service_hash, config.clone(), node_id));

                let port_id = UniqueSystemId::new().unwrap().value();
                create_port_tag(&bad_node, port_id)
                    .unwrap()
                    .release_ownership();
                port_tags.push((port_id, config.clone(), node_id));
            }

            bad_service.abandon();
            bad_node.abandon();

            let cleanup_results = good_service.try_cleanup_dead_nodes();
            assert_that!(cleanup_results.cleanups, eq 1);
            assert_that!(cleanup_results.failed_cleanups, eq 0);

            let node_state = get_node_state::<S>(&node_id, &config).unwrap();
            assert_that!(node_state, is_none);

            let mut counter = 0;
            good_service
                .nodes(|node_state| {
                    counter += 1;
                    assert_that!(node_state.node_id(), eq good_node.id());
                    CallbackProgression::Continue
                })
                .unwrap();
            assert_that!(counter, eq 1);

            for tag in service_tags {
                assert_that!(does_service_tag_exist::<S>(&tag.0, &tag.1, &tag.2).unwrap(), eq false);
            }

            for tag in port_tags {
                assert_that!(does_port_tag_exist::<S>(tag.0, &tag.1, &tag.2).unwrap(), eq false);
            }
        }
    }
}
