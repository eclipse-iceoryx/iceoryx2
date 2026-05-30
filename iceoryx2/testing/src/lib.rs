// Copyright (c) 2026 Contributors to the Eclipse Foundation
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

#![cfg_attr(not(any(test, feature = "std")), no_std)]

extern crate alloc;

use alloc::vec::Vec;
use core::marker::PhantomData;
use core::time::Duration;
use iceoryx2::prelude::*;
pub use iceoryx2::testing::*;
use iceoryx2_bb_elementary_traits::testing::abandonable::Abandonable;
use iceoryx2_bb_testing::watchdog::Watchdog;

pub struct Test<Service: iceoryx2::service::Service> {
    config: Config,
    _watchdog: Watchdog,
    _data: PhantomData<Service>,
}

unsafe impl<S: iceoryx2::service::Service> Send for Test<S> {}
unsafe impl<S: iceoryx2::service::Service> Sync for Test<S> {}

impl<Service: iceoryx2::service::Service> Drop for Test<Service> {
    fn drop(&mut self) {
        Self::cleanup_dead_nodes(&self.config);
        unsafe { remove_global_mgmt_segment::<Service>(&self.config).unwrap() };
    }
}

impl<Service: iceoryx2::service::Service> Default for Test<Service> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Service: iceoryx2::service::Service> Test<Service> {
    pub fn new() -> Self {
        Self::new_with_custom_watchdog(Watchdog::new())
    }

    pub fn new_with_custom_watchdog(watchdog: Watchdog) -> Self {
        let mut config = generate_isolated_config();
        config.global.node.cleanup_dead_nodes_on_creation = false;
        config.global.node.cleanup_dead_nodes_on_destruction = false;
        config.global.service.cleanup_dead_nodes_on_open = false;

        Self {
            config,
            _watchdog: watchdog,
            _data: PhantomData,
        }
    }

    pub fn new_with_custom_config(config: Config) -> Self {
        Self {
            config,
            _watchdog: Watchdog::new(),
            _data: PhantomData,
        }
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn config_mut(&mut self) -> &mut Config {
        &mut self.config
    }

    pub fn abandon_contents<T: Abandonable>(&self, mut contents: Vec<T>) {
        while let Some(element) = contents.pop() {
            T::abandon(element);
        }
    }

    pub fn create_node(&self) -> Node<Service> {
        NodeBuilder::new()
            .config(self.config())
            .create::<Service>()
            .unwrap()
    }

    pub fn number_of_nodes(&self) -> usize {
        self.list_nodes().len()
    }

    pub fn list_nodes(&self) -> Vec<NodeState<Service>> {
        let mut node_list = Vec::new();
        Node::<Service>::list(self.config(), |node_state| {
            node_list.push(node_state);
            CallbackProgression::Continue
        })
        .unwrap();

        node_list
    }

    pub fn cleanup_dead_nodes(config: &Config) {
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
