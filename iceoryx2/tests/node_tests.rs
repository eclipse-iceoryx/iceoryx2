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
    use iceoryx2::config::Config;
    use iceoryx2::node::NodeState;
    use iceoryx2::prelude::*;
    use iceoryx2::service::Service;
    use iceoryx2_bb_testing::assert_that;

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

        let generate_node_name =
            |i: usize| NodeName::new(&("give me a bit".to_string() + &i.to_string())).unwrap();

        let mut nodes = vec![];
        let mut node_names = vec![];
        for i in 0..NUMBER_OF_NODES {
            let node_name = generate_node_name(i);
            nodes.push(NodeBuilder::new().name(&node_name).create::<S>().unwrap());
            node_names.push(node_name);
        }

        let node_list = Node::<S>::list().unwrap();
        for node in node_list {
            match node {
                NodeState::<S>::Alive(state) => {
                    assert_that!(
                        node_names,
                        contains * state.details().as_ref().unwrap().name()
                    )
                }
                NodeState::<S>::Dead(state) => {
                    assert_that!(
                        node_names,
                        contains * state.details().as_ref().unwrap().name()
                    )
                }
            }
        }
    }

    #[instantiate_tests(<iceoryx2::service::zero_copy::Service>)]
    mod zero_copy {}

    #[instantiate_tests(<iceoryx2::service::process_local::Service>)]
    mod process_local {}
}
