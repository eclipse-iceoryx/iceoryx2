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

use core::time::Duration;

use alloc::string::ToString;
use alloc::vec;
use iceoryx2::config::Config;
use iceoryx2::identifiers::UniqueNodeId;
use iceoryx2::node::{CleanupState, NodeState};
use iceoryx2::prelude::*;
use iceoryx2::service::Service;
use iceoryx2::testing::*;
use iceoryx2_bb_concurrency::atomic::AtomicU32;
use iceoryx2_bb_concurrency::atomic::Ordering;
use iceoryx2_bb_testing::leakable::Leakable;
use iceoryx2_bb_testing::watchdog::Watchdog;
use iceoryx2_bb_testing::{assert_that, test_fail};
use iceoryx2_bb_testing_macros::conformance_test;
use iceoryx2_bb_testing_macros::conformance_tests;

pub trait Test {
    type Service: Service;

    fn new() -> Self;

    fn config(&self) -> &Config;

    fn config_mut(&mut self) -> &mut Config;

    fn leak_contents<T: Leakable>(mut contents: Vec<T>) {
        while let Some(element) = contents.pop() {
            T::leak(element);
        }
    }

    fn generate_node_name(i: usize, prefix: &str) -> NodeName {
        NodeName::new(&(prefix.to_string() + &i.to_string())).unwrap()
    }

    fn create_good_node(&self) -> Node<Self::Service> {
        NodeBuilder::new()
            .config(self.config())
            .create::<Self::Service>()
            .unwrap()
    }

    fn number_of_nodes(&self) -> usize {
        self.list_nodes().len()
    }

    fn list_nodes(&self) -> Vec<NodeState<Self::Service>> {
        let mut node_list = vec![];
        Node::<Self::Service>::list(self.config(), |node_state| {
            node_list.push(node_state);
            CallbackProgression::Continue
        })
        .unwrap();

        node_list
    }

    fn create_bad_node(&self) -> Node<Self::Service> {
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        let node_name = Self::generate_node_name(0, "toby or no toby");
        let fake_node_id = ((u32::MAX - COUNTER.fetch_add(1, Ordering::Relaxed)) as u128) << 96;
        let fake_node_id = unsafe { core::mem::transmute::<u128, UniqueNodeId>(fake_node_id) };

        unsafe {
            NodeBuilder::new()
                .name(&node_name)
                .config(self.config())
                .__internal_create_with_custom_node_id::<Self::Service>(fake_node_id)
                .unwrap()
        }
    }

    fn cleanup_dead_nodes(config: &Config) {
        Node::<Self::Service>::list(config, |node_state| {
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

pub struct ZeroCopy {
    config: Config,
    _watchdog: Watchdog,
}

impl Drop for ZeroCopy {
    fn drop(&mut self) {
        Self::cleanup_dead_nodes(&self.config);
    }
}

impl Test for ZeroCopy {
    type Service = iceoryx2::service::ipc::Service;

    fn new() -> Self {
        let _watchdog = Watchdog::new();
        let mut config = generate_isolated_config();
        config.global.node.cleanup_dead_nodes_on_creation = false;
        config.global.node.cleanup_dead_nodes_on_destruction = false;
        config.global.service.cleanup_dead_nodes_on_open = false;

        Self { config, _watchdog }
    }

    fn config(&self) -> &Config {
        &self.config
    }

    fn config_mut(&mut self) -> &mut Config {
        &mut self.config
    }
}

#[allow(clippy::module_inception)]
#[conformance_tests]
pub mod node_death {
    use iceoryx2_bb_testing::leakable::Leakable;

    use super::*;

    #[conformance_test]
    pub fn dead_node_is_marked_as_dead_and_can_be_cleaned_up<S: Test>() {
        let test = S::new();

        const NUMBER_OF_DEAD_NODES_LIMIT: usize = 5;

        for i in 1..NUMBER_OF_DEAD_NODES_LIMIT {
            for _ in 0..i {
                let sut = test.create_bad_node();
                Node::leak(sut);
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
    pub fn dead_node_is_removed_from_pub_sub_service<S: Test>() {
        let test = S::new();

        const NUMBER_OF_BAD_NODES: usize = 3;
        const NUMBER_OF_GOOD_NODES: usize = 4;
        const NUMBER_OF_SERVICES: usize = 5;
        const NUMBER_OF_PUBLISHERS: usize = NUMBER_OF_BAD_NODES + NUMBER_OF_GOOD_NODES;
        const NUMBER_OF_SUBSCRIBERS: usize = NUMBER_OF_BAD_NODES + NUMBER_OF_GOOD_NODES;

        let mut bad_nodes = vec![];
        let mut good_nodes = vec![];

        for _ in 0..NUMBER_OF_BAD_NODES {
            bad_nodes.push(test.create_bad_node());
        }

        for _ in 0..NUMBER_OF_GOOD_NODES {
            good_nodes.push(test.create_good_node());
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

        for _ in 0..NUMBER_OF_BAD_NODES {
            let node = bad_nodes.pop().unwrap();
            Node::leak(node);
        }

        S::leak_contents(bad_services);
        S::leak_contents(bad_publishers);
        S::leak_contents(bad_subscribers);

        assert_that!(Node::<S::Service>::try_cleanup_dead_nodes(test.config()), eq CleanupState { cleanups: NUMBER_OF_BAD_NODES as _, failed_cleanups: 0});

        for service in &good_services {
            assert_that!(service.dynamic_config().number_of_publishers(), eq NUMBER_OF_PUBLISHERS - NUMBER_OF_BAD_NODES);
            assert_that!(service.dynamic_config().number_of_subscribers(), eq NUMBER_OF_SUBSCRIBERS - NUMBER_OF_BAD_NODES);
        }
    }

    #[conformance_test]
    pub fn dead_node_is_removed_from_event_service<S: Test>() {
        let test = S::new();

        const NUMBER_OF_BAD_NODES: usize = 3;
        const NUMBER_OF_GOOD_NODES: usize = 4;
        const NUMBER_OF_SERVICES: usize = 5;
        const NUMBER_OF_NOTIFIERS: usize = NUMBER_OF_BAD_NODES + NUMBER_OF_GOOD_NODES;
        const NUMBER_OF_LISTENERS: usize = NUMBER_OF_BAD_NODES + NUMBER_OF_GOOD_NODES;

        let mut bad_nodes = vec![];
        let mut good_nodes = vec![];

        for _ in 0..NUMBER_OF_BAD_NODES {
            bad_nodes.push(test.create_bad_node());
        }

        for _ in 0..NUMBER_OF_GOOD_NODES {
            good_nodes.push(test.create_good_node());
        }

        let mut services = vec![];
        let mut bad_notifiers = vec![];
        let mut bad_listeners = vec![];
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
            let node = bad_nodes.pop().unwrap();
            Node::leak(node);
        }

        core::mem::forget(bad_notifiers);
        core::mem::forget(bad_listeners);

        assert_that!(Node::<S::Service>::try_cleanup_dead_nodes(test.config()), eq CleanupState { cleanups: NUMBER_OF_BAD_NODES as _, failed_cleanups: 0});

        for service in &services {
            assert_that!(service.dynamic_config().number_of_notifiers(), eq NUMBER_OF_NOTIFIERS - NUMBER_OF_BAD_NODES);
            assert_that!(service.dynamic_config().number_of_listeners(), eq NUMBER_OF_LISTENERS - NUMBER_OF_BAD_NODES);
        }
    }

    #[conformance_test]
    pub fn notifier_of_dead_node_emits_death_event_when_configured<S: Test>() {
        let test = S::new();

        let service_name = generate_service_name();
        let notifier_dead_event = EventId::new(8);

        let dead_node = test.create_bad_node();
        let node = test.create_good_node();

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

        Node::leak(dead_node);
        core::mem::forget(dead_notifier);

        assert_that!(Node::<S::Service>::try_cleanup_dead_nodes(test.config()), eq CleanupState { cleanups: 1, failed_cleanups: 0});

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
    pub fn dead_node_is_removed_from_request_response_service<S: Test>() {
        let test = S::new();

        const NUMBER_OF_BAD_NODES: usize = 2;
        const NUMBER_OF_GOOD_NODES: usize = 3;
        const NUMBER_OF_SERVICES: usize = 4;
        const NUMBER_OF_CLIENTS: usize = NUMBER_OF_BAD_NODES + NUMBER_OF_GOOD_NODES;
        const NUMBER_OF_SERVERS: usize = NUMBER_OF_BAD_NODES + NUMBER_OF_GOOD_NODES;

        let mut bad_nodes = vec![];
        let mut good_nodes = vec![];

        for _ in 0..NUMBER_OF_BAD_NODES {
            bad_nodes.push(test.create_bad_node());
        }

        for _ in 0..NUMBER_OF_GOOD_NODES {
            good_nodes.push(test.create_good_node());
        }

        let mut services = vec![];
        let mut bad_clients = vec![];
        let mut bad_servers = vec![];
        let mut good_clients = vec![];
        let mut good_servers = vec![];

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

                services.push(service);
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

                services.push(service);
            }
        }

        for _ in 0..NUMBER_OF_BAD_NODES {
            let node = bad_nodes.pop().unwrap();
            Node::leak(node);
        }

        core::mem::forget(bad_clients);
        core::mem::forget(bad_servers);

        assert_that!(Node::<S::Service>::try_cleanup_dead_nodes(test.config()), eq CleanupState { cleanups: NUMBER_OF_BAD_NODES as _, failed_cleanups: 0});

        for service in &services {
            assert_that!(service.dynamic_config().number_of_clients(), eq NUMBER_OF_CLIENTS - NUMBER_OF_BAD_NODES);
            assert_that!(service.dynamic_config().number_of_servers(), eq NUMBER_OF_SERVERS - NUMBER_OF_BAD_NODES);
        }
    }

    #[conformance_test]
    pub fn dead_node_is_removed_from_blackboard_service<S: Test>() {
        let test = S::new();

        const NUMBER_OF_BAD_NODES: usize = 3;
        const NUMBER_OF_GOOD_NODES: usize = 4;
        const NUMBER_OF_SERVICES: usize = 5;
        const NUMBER_OF_READERS: usize = NUMBER_OF_BAD_NODES + NUMBER_OF_GOOD_NODES;

        let mut bad_nodes = vec![];
        let mut good_nodes = vec![];

        for _ in 0..NUMBER_OF_BAD_NODES {
            bad_nodes.push(test.create_bad_node());
        }

        for _ in 0..NUMBER_OF_GOOD_NODES {
            good_nodes.push(test.create_good_node());
        }

        let mut services = vec![];
        let mut bad_readers = vec![];
        let mut good_readers = vec![];

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

                services.push(service);
            }

            for node in &good_nodes {
                let service = node
                    .service_builder(&service_name)
                    .blackboard_opener::<u64>()
                    .max_readers(NUMBER_OF_READERS)
                    .open()
                    .unwrap();
                good_readers.push(service.reader_builder().create().unwrap());

                services.push(service);
            }
        }

        for _ in 0..NUMBER_OF_BAD_NODES {
            let node = bad_nodes.pop().unwrap();
            Node::leak(node);
        }

        core::mem::forget(bad_readers);

        assert_that!(Node::<S::Service>::try_cleanup_dead_nodes(test.config()), eq CleanupState { cleanups: NUMBER_OF_BAD_NODES as _, failed_cleanups: 0});

        for service in &services {
            assert_that!(service.dynamic_config().number_of_readers(), eq NUMBER_OF_READERS - NUMBER_OF_BAD_NODES);
        }
    }

    #[conformance_test]
    pub fn opened_blackboard_can_be_accessed_after_creator_node_crash<S: Test>() {
        let test = S::new();
        let service_name = generate_service_name();

        let bad_node = test.create_bad_node();
        let bad_service = bad_node
            .service_builder(&service_name)
            .blackboard_creator::<u64>()
            .add_with_default::<u64>(0)
            .create()
            .unwrap();
        let writer = bad_service.writer_builder().create().unwrap();

        let good_node = NodeBuilder::new()
            .config(test.config())
            .create::<S::Service>()
            .unwrap();
        let good_service = good_node
            .service_builder(&service_name)
            .blackboard_opener::<u64>()
            .open()
            .unwrap();
        let reader = good_service.reader_builder().create().unwrap();

        Node::leak(bad_node);
        core::mem::forget(writer);
        core::mem::forget(bad_service);
        assert_that!(Node::<S::Service>::try_cleanup_dead_nodes(test.config()), eq CleanupState { cleanups: 1, failed_cleanups: 0});

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
    pub fn event_service_is_removed_when_last_node_dies<S: Test>() {
        let test = S::new();
        let service_name = generate_service_name();

        let sut = test.create_bad_node();
        core::mem::forget(
            sut.service_builder(&service_name)
                .event()
                .open_or_create()
                .unwrap(),
        );
        Node::leak(sut);

        assert_that!(
            S::Service::list(test.config(), |service_details| {
                assert_that!(*service_details.static_details.name(), eq service_name);
                CallbackProgression::Continue
            }),
            is_ok
        );

        assert_that!(Node::<S::Service>::try_cleanup_dead_nodes(test.config()), eq CleanupState { cleanups: 1, failed_cleanups: 0});

        assert_that!(
            S::Service::list(test.config(), |_| {
                test_fail!("after the cleanup there shall be no more services");
            }),
            is_ok
        );
    }

    #[conformance_test]
    pub fn pubsub_service_is_removed_when_last_node_dies<S: Test>() {
        let test = S::new();
        let service_name = generate_service_name();

        let sut = test.create_bad_node();
        core::mem::forget(
            sut.service_builder(&service_name)
                .publish_subscribe::<u64>()
                .open_or_create()
                .unwrap(),
        );
        Node::leak(sut);

        assert_that!(
            S::Service::list(test.config(), |service_details| {
                assert_that!(*service_details.static_details.name(), eq service_name);
                CallbackProgression::Continue
            }),
            is_ok
        );

        assert_that!(Node::<S::Service>::try_cleanup_dead_nodes(test.config()), eq CleanupState { cleanups: 1, failed_cleanups: 0});

        assert_that!(
            S::Service::list(test.config(), |_| {
                test_fail!("after the cleanup there shall be no more services");
            }),
            is_ok
        );
    }

    #[conformance_test]
    pub fn request_response_service_is_removed_when_last_node_dies<S: Test>() {
        let test = S::new();
        let service_name = generate_service_name();

        let sut = test.create_bad_node();
        core::mem::forget(
            sut.service_builder(&service_name)
                .request_response::<u64, u64>()
                .open_or_create()
                .unwrap(),
        );
        Node::leak(sut);

        assert_that!(
            S::Service::list(test.config(), |service_details| {
                assert_that!(*service_details.static_details.name(), eq service_name);
                CallbackProgression::Continue
            }),
            is_ok
        );

        assert_that!(Node::<S::Service>::try_cleanup_dead_nodes(test.config()), eq CleanupState { cleanups: 1, failed_cleanups: 0});

        assert_that!(
            S::Service::list(test.config(), |_| {
                test_fail!("after the cleanup there shall be no more services");
            }),
            is_ok
        );
    }

    #[conformance_test]
    pub fn blackboard_service_is_removed_when_last_node_dies<S: Test>() {
        let test = S::new();
        let service_name = generate_service_name();

        let sut = test.create_bad_node();
        core::mem::forget(
            sut.service_builder(&service_name)
                .blackboard_creator::<u64>()
                .add_with_default::<u64>(0)
                .create()
                .unwrap(),
        );
        Node::leak(sut);

        assert_that!(
            S::Service::list(test.config(), |service_details| {
                assert_that!(*service_details.static_details.name(), eq service_name);
                CallbackProgression::Continue
            }),
            is_ok
        );

        assert_that!(Node::<S::Service>::try_cleanup_dead_nodes(test.config()), eq CleanupState { cleanups: 1, failed_cleanups: 0});

        assert_that!(
            S::Service::list(test.config(), |_| {
                test_fail!("after the cleanup there shall be no more services");
            }),
            is_ok
        );
    }

    #[conformance_test]
    pub fn writer_and_reader_resources_are_removed_after_crash<S: Test>() {
        let test = S::new();
        let service_name = generate_service_name();

        let good_node = test.create_good_node();
        let good_service = good_node
            .service_builder(&service_name)
            .blackboard_creator::<u64>()
            .max_readers(1)
            .add_with_default::<u64>(0)
            .create()
            .unwrap();

        let bad_node = test.create_bad_node();
        let bad_service = bad_node
            .service_builder(&service_name)
            .blackboard_opener::<u64>()
            .open()
            .unwrap();
        let writer = bad_service.writer_builder().create().unwrap();
        let reader = bad_service.reader_builder().create().unwrap();

        Node::leak(bad_node);
        assert_that!(Node::<S::Service>::try_cleanup_dead_nodes(test.config()), eq CleanupState { cleanups: 1, failed_cleanups: 0});
        core::mem::forget(writer);
        core::mem::forget(reader);

        let writer = good_service.writer_builder().create();
        assert_that!(writer, is_ok);
        let reader = good_service.reader_builder().create();
        assert_that!(reader, is_ok);
    }

    // test disabled on Windows as the state files cannot be removed after simulated node death
    #[cfg(not(target_os = "windows"))]
    #[conformance_test]
    pub fn blackboard_resources_are_removed_when_key_has_user_defined_name<S: Test>() {
        let test = S::new();
        let service_name = generate_service_name();

        #[repr(C)]
        #[derive(ZeroCopySend, Debug, Clone, Copy, PartialEq, Eq, Hash)]
        #[type_name("SoSpecial")]
        struct SpecialKey(u64);

        let sut = test.create_bad_node();
        core::mem::forget(
            sut.service_builder(&service_name)
                .blackboard_creator::<SpecialKey>()
                .add_with_default::<u64>(SpecialKey(0))
                .create()
                .unwrap(),
        );
        Node::leak(sut);

        assert_that!(
            S::Service::list(test.config(), |service_details| {
                assert_that!(*service_details.static_details.name(), eq service_name);
                CallbackProgression::Continue
            }),
            is_ok
        );

        assert_that!(Node::<S::Service>::try_cleanup_dead_nodes(test.config()), eq CleanupState { cleanups: 1, failed_cleanups: 0});

        assert_that!(
            S::Service::list(test.config(), |_| {
                test_fail!("after the cleanup there shall be no more services");
            }),
            is_ok
        );

        let node = test.create_good_node();
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
    pub fn blackboard_resources_are_removed_when_last_node_dies<S: Test>() {
        let test = S::new();
        let service_name = generate_service_name();

        let sut = test.create_bad_node();
        core::mem::forget(
            sut.service_builder(&service_name)
                .blackboard_creator::<u64>()
                .add_with_default::<u64>(0)
                .create()
                .unwrap(),
        );
        Node::leak(sut);

        assert_that!(
            S::Service::list(test.config(), |service_details| {
                assert_that!(*service_details.static_details.name(), eq service_name);
                CallbackProgression::Continue
            }),
            is_ok
        );

        assert_that!(Node::<S::Service>::try_cleanup_dead_nodes(test.config()), eq CleanupState { cleanups: 1, failed_cleanups: 0});

        assert_that!(
            S::Service::list(test.config(), |_| {
                test_fail!("after the cleanup there shall be no more services");
            }),
            is_ok
        );

        let node = test.create_good_node();
        let service = node
            .service_builder(&service_name)
            .blackboard_creator::<u64>()
            .add_with_default::<u64>(0)
            .create();
        assert_that!(service, is_ok);
    }

    #[conformance_test]
    pub fn node_cleanup_option_works_on_node_creation<S: Test>() {
        let test = S::new();

        let sut = test.create_bad_node();
        Node::leak(sut);

        assert_that!(test.number_of_nodes(), eq 1);

        let node_without_cleanup = test.create_good_node();

        assert_that!(test.number_of_nodes(), eq 2);

        let mut config = test.config().clone();
        config.global.node.cleanup_dead_nodes_on_creation = true;
        let node_with_cleanup = NodeBuilder::new()
            .config(&config)
            .create::<S::Service>()
            .unwrap();

        assert_that!(test.number_of_nodes(), eq 2);

        drop(node_with_cleanup);
        drop(node_without_cleanup);

        assert_that!(test.number_of_nodes(), eq 0);
    }

    #[conformance_test]
    pub fn node_cleanup_option_works_on_node_destruction<S: Test>() {
        let mut test = S::new();
        test.config_mut()
            .global
            .node
            .cleanup_dead_nodes_on_destruction = true;

        let node_with_cleanup = test.create_good_node();

        let mut config = test.config().clone();
        config.global.node.cleanup_dead_nodes_on_destruction = false;
        let node_without_cleanup = NodeBuilder::new()
            .config(&config)
            .create::<S::Service>()
            .unwrap();

        let sut = test.create_bad_node();
        Node::leak(sut);

        assert_that!(test.number_of_nodes(), eq 3);

        drop(node_without_cleanup);

        assert_that!(test.number_of_nodes(), eq 2);

        drop(node_with_cleanup);

        assert_that!(test.number_of_nodes(), eq 0);
    }

    pub fn node_cleanup_on_service_connection_works<
        S: Test,
        T,
        F: FnMut(&Node<S::Service>) -> T,
    >(
        test: S,
        total_number_of_nodes: usize,
        mut service_builder: F,
    ) {
        let sut = test.create_bad_node();
        core::mem::forget(service_builder(&sut));
        Node::leak(sut);

        let sut = test.create_bad_node();
        let _service = service_builder(&sut);

        let number_of_nodes = test.number_of_nodes();
        S::cleanup_dead_nodes(test.config());
        assert_that!(number_of_nodes, eq total_number_of_nodes);
    }

    #[conformance_test]
    pub fn publish_subscribe_node_cleanup_on_open_works_when_enabled<S: Test>() {
        let mut test = S::new();
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
    pub fn publish_subscribe_no_node_cleanup_on_open_when_disabled<S: Test>() {
        let test = S::new();
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
    pub fn request_response_node_cleanup_on_open_works_when_enabled<S: Test>() {
        let mut test = S::new();
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
    pub fn request_response_no_node_cleanup_on_open_when_disabled<S: Test>() {
        let test = S::new();
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
    pub fn event_node_cleanup_on_open_works_when_enabled<S: Test>() {
        let mut test = S::new();
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
    pub fn event_no_node_cleanup_on_open_when_disabled<S: Test>() {
        let test = S::new();
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
    pub fn blackboard_node_cleanup_on_open_works_when_enabled<S: Test>() {
        let mut test = S::new();
        test.config_mut().global.service.cleanup_dead_nodes_on_open = true;
        const NUMBER_OF_CONNECTED_NODES_AFTER_CONNECTION: usize = 2;
        let service_name = generate_service_name();

        let sut = test.create_bad_node();
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
    pub fn blackboard_no_node_cleanup_on_open_when_disabled<S: Test>() {
        let test = S::new();
        const NUMBER_OF_CONNECTED_NODES_AFTER_CONNECTION: usize = 3;
        let service_name = generate_service_name();

        let sut = test.create_bad_node();
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
}
