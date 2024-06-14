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
    use std::collections::HashSet;

    use iceoryx2::config::Config;
    use iceoryx2::node::NodeState;
    use iceoryx2::prelude::*;
    use iceoryx2::service::Service;
    use iceoryx2_bb_posix::unique_system_id::UniqueSystemId;
    use iceoryx2_bb_testing::assert_that;

    #[derive(Debug, Eq, PartialEq)]
    struct Details {
        name: NodeName,
        id: u128,
        config: Config,
    }

    impl Details {
        fn new(name: &NodeName, id: &UniqueSystemId, config: &Config) -> Self {
            Self {
                name: name.clone(),
                id: id.value(),
                config: config.clone(),
            }
        }

        fn from_node<S: Service>(node: &Node<S>) -> Self {
            Self::new(node.name(), node.id(), node.config())
        }
    }

    fn assert_node_presence<S: Service>(node_details: &Vec<Details>) {
        let node_list = Node::<S>::list().unwrap();
        for node in node_list {
            match node {
                NodeState::<S>::Alive(state) => {
                    let state_details = state.details().as_ref().unwrap();
                    let triple =
                        Details::new(state_details.name(), state.id(), state_details.config());

                    assert_that!(
                        *node_details,
                        contains triple
                    )
                }
                NodeState::<S>::Dead(state) => {
                    let state_details = state.details().as_ref().unwrap();
                    let triple =
                        Details::new(state_details.name(), state.id(), state_details.config());

                    assert_that!(
                        *node_details,
                        contains triple
                    )
                }
            }
        }
    }

    fn generate_node_name(i: usize, prefix: &str) -> NodeName {
        NodeName::new(&(prefix.to_string() + &i.to_string())).unwrap()
    }

    #[test]
    fn node_without_name_can_be_created<S: Service>() {
        let sut = NodeBuilder::new().create::<S>().unwrap();

        assert_that!(*sut.name(), eq NodeName::new("").unwrap());
    }

    #[test]
    fn node_with_name_can_be_created<S: Service>() {
        let node_name = NodeName::new("photons taste like chicken").unwrap();
        let sut = NodeBuilder::new().name(&node_name).create::<S>().unwrap();

        assert_that!(*sut.name(), eq node_name);
    }

    #[test]
    fn multiple_nodes_with_the_same_name_can_be_created<S: Service>() {
        const NUMBER_OF_NODES: usize = 16;
        let node_name = NodeName::new("but what does an electron taste like?").unwrap();

        let mut nodes = vec![];
        for _ in 0..NUMBER_OF_NODES {
            nodes.push(NodeBuilder::new().name(&node_name).create::<S>().unwrap());
        }

        for node in nodes {
            assert_that!(*node.name(), eq node_name);
        }
    }

    #[test]
    fn without_custom_config_global_config_is_used<S: Service>() {
        let sut = NodeBuilder::new().create::<S>().unwrap();

        assert_that!(*sut.config(), eq * Config::get_global_config());
    }

    #[test]
    fn nodes_can_be_listed<S: Service>() {
        const NUMBER_OF_NODES: usize = 16;

        let mut nodes = vec![];
        let mut node_details = vec![];
        for i in 0..NUMBER_OF_NODES {
            let node_name = generate_node_name(i, "give me a bit");
            let node = NodeBuilder::new().name(&node_name).create::<S>().unwrap();
            node_details.push(Details::from_node(&node));
            nodes.push(node);
        }

        assert_node_presence::<S>(&node_details);
    }

    #[test]
    fn when_node_goes_out_of_scope_it_cleans_up<S: Service>() {
        const NUMBER_OF_NODES: usize = 16;

        let mut nodes = vec![];
        let mut node_details = vec![];
        for i in 0..NUMBER_OF_NODES {
            let node_name = generate_node_name(i, "gravity should be illegal");
            let node = NodeBuilder::new().name(&node_name).create::<S>().unwrap();
            node_details.push(Details::from_node(&node));
            nodes.push(node);
        }

        for _ in 0..NUMBER_OF_NODES {
            nodes.pop();
            node_details.pop();
            assert_node_presence::<S>(&node_details);
        }
    }

    #[test]
    fn id_is_unique<S: Service>() {
        const NUMBER_OF_NODES: usize = 16;

        let mut nodes = vec![];
        let mut node_ids = HashSet::new();
        for i in 0..NUMBER_OF_NODES {
            let node_name = generate_node_name(
                i,
                "its a bird, its a plane, no its the mountain goat jumping through the code",
            );
            nodes.push(NodeBuilder::new().name(&node_name).create::<S>().unwrap());
            assert_that!(node_ids.insert(nodes.last().unwrap().id().value()), eq true);
        }
    }

    #[instantiate_tests(<iceoryx2::service::zero_copy::Service>)]
    mod zero_copy {}

    #[instantiate_tests(<iceoryx2::service::process_local::Service>)]
    mod process_local {}
}
