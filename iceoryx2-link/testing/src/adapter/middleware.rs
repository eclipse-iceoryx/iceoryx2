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

use alloc::collections::{BTreeMap, VecDeque};
use alloc::rc::Rc;
use alloc::vec::Vec;

use iceoryx2::service::service_name::ServiceName;
use iceoryx2_bb_concurrency::cell::{Ref, RefCell};
use iceoryx2_link_backend::WakeHandle;

use crate::adapter::{FakeAdapter, FakeEndpointDescription, FakeEndpoints, header_size};

/// Who created an endpoint. Only remote endpoints make a description one
/// the adapter lists.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Origin {
    Remote,
    Gateway,
}

pub(super) struct Endpoint {
    pub(super) origin: Origin,
    inbox: VecDeque<Vec<u8>>,
}

/// The endpoints under one description.
pub(super) struct Group {
    pub(super) description: FakeEndpointDescription,
    pub(super) endpoints: BTreeMap<u64, Endpoint>,
}

#[derive(Default)]
pub(super) struct State {
    pub(super) groups: BTreeMap<ServiceName, Group>,
    /// Moved whenever endpoints join or leave.
    pub(super) generation: u64,
    next_id: u64,
    /// The wakes of the adapters that attached one.
    pub(super) wakes: Vec<WakeHandle>,
}

impl State {
    fn wake_adapters(&self) {
        for wake in &self.wakes {
            wake.signal();
        }
    }
}

/// A fake substitute for a middleware. Endpoints under one
/// description receive each other's messages.
#[derive(Clone, Default)]
pub struct FakeMiddleware {
    state: Rc<RefCell<State>>,
}

impl FakeMiddleware {
    /// Creates an empty middleware.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates an adapter connected to the middleware.
    pub fn adapter(&self) -> FakeAdapter {
        FakeAdapter::new(self.clone())
    }

    /// Creates remote endpoints under `description`, as an application of
    /// the middleware would.
    pub fn remote_endpoints(&self, description: &FakeEndpointDescription) -> FakeEndpoints {
        let id = self.join(description, Origin::Remote);
        FakeEndpoints::new(
            self.clone(),
            description.settings.name,
            id,
            header_size(description),
        )
    }

    pub(super) fn state(&self) -> Ref<'_, State> {
        self.state.borrow()
    }

    pub(super) fn attach(&self, wake: WakeHandle) {
        self.state.borrow_mut().wakes.push(wake);
    }

    pub(super) fn join(&self, description: &FakeEndpointDescription, origin: Origin) -> u64 {
        let mut state = self.state.borrow_mut();
        let id = state.next_id;
        state.next_id += 1;
        state.generation = state.generation.wrapping_add(1);
        state
            .groups
            .entry(description.settings.name)
            .or_insert_with(|| Group {
                description: description.clone(),
                endpoints: BTreeMap::new(),
            })
            .endpoints
            .insert(
                id,
                Endpoint {
                    origin,
                    inbox: VecDeque::new(),
                },
            );
        state.wake_adapters();
        id
    }

    pub(super) fn leave(&self, name: &ServiceName, id: u64) {
        let mut state = self.state.borrow_mut();
        state.generation = state.generation.wrapping_add(1);
        if let Some(entry) = state.groups.get_mut(name) {
            entry.endpoints.remove(&id);
            if entry.endpoints.is_empty() {
                state.groups.remove(name);
            }
        }
        state.wake_adapters();
    }

    pub(super) fn publish(&self, name: &ServiceName, from: u64, message: &[u8]) {
        let mut state = self.state.borrow_mut();
        let Some(entry) = state.groups.get_mut(name) else {
            return;
        };
        for (_, remote) in entry.endpoints.iter_mut().filter(|(id, _)| **id != from) {
            remote.inbox.push_back(message.to_vec());
        }
        state.wake_adapters();
    }

    pub(super) fn receive(&self, name: &ServiceName, id: u64) -> Option<Vec<u8>> {
        self.state
            .borrow_mut()
            .groups
            .get_mut(name)?
            .endpoints
            .get_mut(&id)?
            .inbox
            .pop_front()
    }
}
