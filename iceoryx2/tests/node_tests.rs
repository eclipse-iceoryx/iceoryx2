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
mod node {
    use core::time::Duration;
    use std::collections::{HashSet, VecDeque};
    use std::sync::Barrier;

    use iceoryx2::config::Config;
    use iceoryx2::node::{
        NodeCleanupFailure, NodeCreationFailure, NodeId, NodeListFailure, NodeState, NodeView,
    };
    use iceoryx2::prelude::*;
    use iceoryx2::service::Service;
    use iceoryx2::testing::*;
    use iceoryx2_bb_posix::system_configuration::SystemInfo;
    use iceoryx2_bb_testing::watchdog::Watchdog;
    use iceoryx2_bb_testing::{assert_that, test_fail};

    #[derive(Debug, Eq, PartialEq)]
    struct Details {
        name: NodeName,
        id: NodeId,
        config: Config,
    }

    impl Details {
        fn new(name: &NodeName, id: &NodeId, config: &Config) -> Self {
            Self {
                name: name.clone(),
                id: id.clone(),
                config: config.clone(),
            }
        }

        fn from_node<S: Service>(node: &Node<S>) -> Self {
            Self::new(node.name(), node.id(), node.config())
        }
    }

    fn assert_node_presence<S: Service>(node_details: &VecDeque<Details>, config: &Config) {
        let mut node_list = vec![];
        Node::<S>::list(config, |node_state| {
            node_list.push(node_state);
            CallbackProgression::Continue
        })
        .unwrap();

        assert_that!(node_list, len node_details.len());
        for node in node_list {
            let view = match node {
                NodeState::<S>::Alive(ref view) => view as &dyn NodeView,
                NodeState::<S>::Dead(ref view) => view as &dyn NodeView,
                NodeState::<S>::Inaccessible(_) | NodeState::<S>::Undefined(_) => {
                    assert_that!(true, eq false);
                    panic!();
                }
            };

            let details = view.details().as_ref().unwrap();
            let triple = Details::new(details.name(), view.id(), details.config());

            assert_that!(
                *node_details,
                contains triple
            )
        }
    }

    fn generate_node_name(i: usize, prefix: &str) -> NodeName {
        NodeName::new(&(prefix.to_string() + &i.to_string())).unwrap()
    }

    #[test]
    fn node_without_name_can_be_created<S: Service>() {
        let config = generate_isolated_config();
        let sut = NodeBuilder::new().config(&config).create::<S>().unwrap();

        assert_that!(*sut.name(), eq NodeName::new("").unwrap());
    }

    #[test]
    fn node_with_name_can_be_created<S: Service>() {
        let config = generate_isolated_config();
        let node_name = NodeName::new("photons taste like chicken").unwrap();
        let sut = NodeBuilder::new()
            .config(&config)
            .name(&node_name)
            .create::<S>()
            .unwrap();

        assert_that!(*sut.name(), eq node_name);
    }

    #[test]
    fn multiple_nodes_with_the_same_name_can_be_created<S: Service>() {
        const NUMBER_OF_NODES: usize = 16;
        let config = generate_isolated_config();
        let node_name = NodeName::new("but what does an electron taste like?").unwrap();

        let mut nodes = vec![];
        for _ in 0..NUMBER_OF_NODES {
            nodes.push(
                NodeBuilder::new()
                    .config(&config)
                    .name(&node_name)
                    .create::<S>()
                    .unwrap(),
            );
        }

        for node in nodes {
            assert_that!(*node.name(), eq node_name);
        }
    }

    #[test]
    fn without_custom_config_global_config_is_used<S: Service>() {
        let sut = NodeBuilder::new().create::<S>().unwrap();

        assert_that!(*sut.config(), eq * Config::global_config());
    }

    #[test]
    fn nodes_can_be_listed<S: Service>() {
        const NUMBER_OF_NODES: usize = 16;
        let config = generate_isolated_config();

        let mut nodes = vec![];
        let mut node_details = VecDeque::new();
        for i in 0..NUMBER_OF_NODES {
            let node_name = generate_node_name(i, "give me a bit");
            let node = NodeBuilder::new()
                .config(&config)
                .name(&node_name)
                .create::<S>()
                .unwrap();
            node_details.push_back(Details::from_node(&node));
            nodes.push(node);
        }

        assert_node_presence::<S>(&node_details, &config);
    }

    #[test]
    fn when_node_goes_out_of_scope_it_cleans_up<S: Service>() {
        const NUMBER_OF_NODES: usize = 16;
        let config = generate_isolated_config();

        let mut nodes = vec![];
        let mut node_details = VecDeque::new();
        for i in 0..NUMBER_OF_NODES {
            let node_name = generate_node_name(i, "gravity should be illegal");
            let node = NodeBuilder::new()
                .config(&config)
                .name(&node_name)
                .create::<S>()
                .unwrap();
            node_details.push_back(Details::from_node(&node));
            nodes.push(node);
        }

        for _ in 0..NUMBER_OF_NODES {
            nodes.pop();
            node_details.pop_back();
            assert_node_presence::<S>(&node_details, &config);
        }
    }

    #[test]
    fn id_is_unique<S: Service>() {
        const NUMBER_OF_NODES: usize = 16;
        let config = generate_isolated_config();

        let mut nodes = vec![];
        let mut node_ids = HashSet::new();
        for i in 0..NUMBER_OF_NODES {
            let node_name = generate_node_name(
                i,
                "its a bird, its a plane, no its the mountain goat jumping through the code",
            );
            nodes.push(
                NodeBuilder::new()
                    .config(&config)
                    .name(&node_name)
                    .create::<S>()
                    .unwrap(),
            );
            assert_that!(node_ids.insert(nodes.last().unwrap().id().clone()), eq true);
        }
    }

    #[test]
    fn nodes_with_disjunct_config_are_separated<S: Service>() {
        const NUMBER_OF_NODES: usize = 16;

        let mut nodes_1 = VecDeque::new();
        let mut node_details_1 = VecDeque::new();
        let mut nodes_2 = VecDeque::new();
        let mut node_details_2 = VecDeque::new();

        let config = generate_isolated_config();
        let mut config_1 = config.clone();
        config_1.global.node.directory = Path::new(b"node2").unwrap();
        let mut config_2 = config.clone();
        config_2.global.node.directory = Path::new(b"bud_spencer").unwrap();

        for i in 0..NUMBER_OF_NODES {
            let node_name_1 = generate_node_name(i, "gravity should be illegal");
            let node_name_2 = generate_node_name(i, "i like to name it name it");
            let node_1 = NodeBuilder::new()
                .config(&config_1)
                .name(&node_name_1)
                .create::<S>()
                .unwrap();
            let node_2 = NodeBuilder::new()
                .config(&config_2)
                .name(&node_name_2)
                .create::<S>()
                .unwrap();

            node_details_1.push_back(Details::from_node(&node_1));
            node_details_2.push_back(Details::from_node(&node_2));
            nodes_1.push_back(node_1);
            nodes_2.push_back(node_2);
        }

        for _ in 0..NUMBER_OF_NODES {
            nodes_1.pop_back();
            nodes_2.pop_front();
            node_details_1.pop_back();
            node_details_2.pop_front();

            assert_node_presence::<S>(&node_details_1, &config_1);
            assert_node_presence::<S>(&node_details_2, &config_2);
        }
    }

    #[test]
    fn node_creation_failure_display_works<S: Service>() {
        assert_that!(
            format!("{}", NodeCreationFailure::InsufficientPermissions), eq "NodeCreationFailure::InsufficientPermissions");
        assert_that!(
            format!("{}", NodeCreationFailure::InternalError), eq "NodeCreationFailure::InternalError");
    }

    #[test]
    fn node_list_failure_display_works<S: Service>() {
        assert_that!(
            format!("{}", NodeListFailure::InsufficientPermissions), eq "NodeListFailure::InsufficientPermissions");
        assert_that!(
            format!("{}", NodeListFailure::Interrupt), eq "NodeListFailure::Interrupt");
        assert_that!(
            format!("{}", NodeListFailure::InternalError), eq "NodeListFailure::InternalError");
    }

    #[test]
    fn node_cleanup_failure_display_works<S: Service>() {
        assert_that!(
            format!("{}", NodeCleanupFailure::InsufficientPermissions), eq "NodeCleanupFailure::InsufficientPermissions");
        assert_that!(
            format!("{}", NodeCleanupFailure::Interrupt), eq "NodeCleanupFailure::Interrupt");
        assert_that!(
            format!("{}", NodeCleanupFailure::InternalError), eq "NodeCleanupFailure::InternalError");
    }

    #[test]
    fn concurrent_node_creation_and_listing_works<S: Service>() {
        let _watch_dog = Watchdog::new_with_timeout(Duration::from_secs(120));
        let number_of_creators = (SystemInfo::NumberOfCpuCores.value()).clamp(2, 1024);
        const NUMBER_OF_ITERATIONS: usize = 100;
        let barrier = Barrier::new(number_of_creators);
        let mut config = generate_isolated_config();
        config.global.node.cleanup_dead_nodes_on_creation = false;
        config.global.node.cleanup_dead_nodes_on_destruction = false;

        std::thread::scope(|s| {
            let mut threads = vec![];
            for _ in 0..number_of_creators {
                threads.push(s.spawn(|| {
                    barrier.wait();
                    for _ in 0..NUMBER_OF_ITERATIONS {
                        let node = NodeBuilder::new().config(&config).create::<S>().unwrap();

                        let mut found_self = false;
                        let result = Node::<S>::list(node.config(), |node_state| {
                            match node_state {
                                NodeState::Alive(view) => {
                                    if view.id() == node.id() {
                                        found_self = true;
                                    }
                                }
                                NodeState::Dead(view) => {
                                    if view.id() == node.id() {
                                        found_self = true;
                                    }
                                }
                                NodeState::Inaccessible(node_id) => {
                                    if node_id == *node.id() {
                                        found_self = true;
                                    }
                                }
                                NodeState::Undefined(_) => {
                                    assert_that!(true, eq false);
                                }
                            };

                            CallbackProgression::Continue
                        });

                        assert_that!(found_self, eq true);
                        assert_that!(result, is_ok);
                    }
                }));
            }

            for thread in threads {
                thread.join().unwrap();
            }
        });
    }

    #[test]
    fn node_listing_stops_when_callback_progression_signals_stop<S: Service>() {
        let config = generate_isolated_config();
        let node_1 = NodeBuilder::new().config(&config).create::<S>().unwrap();
        let _node_2 = NodeBuilder::new().config(&config).create::<S>().unwrap();

        let mut node_counter = 0;
        let result = Node::<S>::list(node_1.config(), |_| {
            node_counter += 1;
            CallbackProgression::Stop
        });

        assert_that!(result, is_ok);
        assert_that!(node_counter, eq 1);
    }

    #[test]
    fn i_am_not_dead<S: Service>() {
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<S>().unwrap();

        let mut nodes = vec![];
        let result = Node::<S>::list(node.config(), |node_state| {
            nodes.push(node_state);
            CallbackProgression::Continue
        });

        assert_that!(result, is_ok);
        assert_that!(nodes, len 1);

        if let NodeState::Alive(node_view) = &nodes[0] {
            assert_that!(node_view.id(), eq node.id());
        } else {
            test_fail!("Process internal nodes shall be always detected as alive.");
        }
    }

    #[test]
    fn signal_handling_mechanism_can_be_configured<S: Service>() {
        let config = generate_isolated_config();
        let node_1 = NodeBuilder::new()
            .signal_handling_mode(SignalHandlingMode::Disabled)
            .config(&config)
            .create::<S>()
            .unwrap();

        let node_2 = NodeBuilder::new()
            .signal_handling_mode(SignalHandlingMode::HandleTerminationRequests)
            .config(&config)
            .create::<S>()
            .unwrap();

        assert_that!(node_1.signal_handling_mode(), eq SignalHandlingMode::Disabled);
        assert_that!(node_2.signal_handling_mode(), eq SignalHandlingMode::HandleTerminationRequests);
    }

    #[test]
    fn by_default_termination_signals_are_handled<S: Service>() {
        let config = generate_isolated_config();
        let node = NodeBuilder::new().config(&config).create::<S>().unwrap();

        assert_that!(node.signal_handling_mode(), eq SignalHandlingMode::HandleTerminationRequests);
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
