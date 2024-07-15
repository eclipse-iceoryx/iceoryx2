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
    use iceoryx2::node::{NodeState, NodeView};
    use iceoryx2::prelude::*;
    use iceoryx2::service::Service;
    use iceoryx2_bb_posix::unique_system_id::UniqueSystemId;
    use iceoryx2_bb_testing::assert_that;

    fn generate_name() -> ServiceName {
        ServiceName::new(&format!(
            "node_death_tests_{}",
            UniqueSystemId::new().unwrap().value()
        ))
        .unwrap()
    }

    struct TestDetails<S: Service> {
        node: Node<S>,
        node_name: NodeName,
    }

    trait Test {
        type Service: Service;

        fn generate_node_name(i: usize, prefix: &str) -> NodeName {
            NodeName::new(&(prefix.to_string() + &i.to_string())).unwrap()
        }

        fn create_test_node() -> TestDetails<Self::Service> {
            static COUNTER: AtomicU32 = AtomicU32::new(0);
            let node_name = Self::generate_node_name(0, "toby or no toby");
            let fake_node_id = ((u32::MAX - COUNTER.fetch_add(0, Ordering::Relaxed)) as u128) << 96;
            let fake_node_id =
                unsafe { core::mem::transmute::<u128, UniqueSystemId>(fake_node_id) };

            let node = unsafe {
                NodeBuilder::new()
                    .name(&node_name)
                    .__internal_create_with_custom_node_id::<Self::Service>(fake_node_id)
                    .unwrap()
            };

            TestDetails { node, node_name }
        }

        fn staged_death(node: &mut Node<Self::Service>);
    }

    struct ZeroCopy;

    impl Test for ZeroCopy {
        type Service = iceoryx2::service::zero_copy::Service;

        fn staged_death(node: &mut Node<Self::Service>) {
            use iceoryx2_cal::monitoring::testing::__InternalMonitoringTokenTestable;
            let monitor = unsafe { __internal_node_staged_death(node) };
            monitor.staged_death();
        }
    }

    #[test]
    fn dead_node_is_marked_as_dead_and_can_be_cleaned_up<S: Test>() {
        let mut sut = S::create_test_node();
        S::staged_death(&mut sut.node);

        let mut node_list = vec![];
        Node::<S::Service>::list(Config::global_config(), |node_state| {
            node_list.push(node_state);
            CallbackProgression::Continue
        })
        .unwrap();
        assert_that!(node_list, len 1);

        if let Some(NodeState::Dead(state)) = node_list.pop() {
            assert_that!(*state.details().as_ref().unwrap().name(), eq sut.node_name);
            assert_that!(state.remove_stale_resources(), eq Ok(true));
        } else {
            assert_that!(true, eq false);
        }

        node_list.clear();
        Node::<S::Service>::list(Config::global_config(), |node_state| {
            node_list.push(node_state);
            CallbackProgression::Continue
        })
        .unwrap();
        assert_that!(node_list, len 0);
    }

    #[test]
    fn dead_node_is_removed_from_pub_sub_service<S: Test>() {
        const NUMBER_OF_BAD_NODES: usize = 2;
        const NUMBER_OF_GOOD_NODES: usize = 3;
        const NUMBER_OF_SERVICES: usize = 14;
        const NUMBER_OF_PUBLISHERS: usize = NUMBER_OF_BAD_NODES + NUMBER_OF_GOOD_NODES;
        const NUMBER_OF_SUBSCRIBERS: usize = NUMBER_OF_BAD_NODES + NUMBER_OF_GOOD_NODES;

        let mut bad_nodes = vec![];
        let mut good_nodes = vec![];

        for _ in 0..NUMBER_OF_BAD_NODES {
            bad_nodes.push(S::create_test_node().node);
        }

        for _ in 0..NUMBER_OF_GOOD_NODES {
            good_nodes.push(NodeBuilder::new().create::<S::Service>().unwrap());
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
    }

    #[instantiate_tests(<ZeroCopy>)]
    mod zero_copy {}
}
