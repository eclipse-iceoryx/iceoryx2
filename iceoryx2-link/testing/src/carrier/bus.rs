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

use alloc::collections::{BTreeMap, BTreeSet, VecDeque};
use alloc::rc::Rc;
use alloc::vec::Vec;

use iceoryx2_bb_concurrency::cell::{Cell, RefCell};
use iceoryx2_link_backend::WakeHandle;
use iceoryx2_link_backend::service_description::ServiceDescriptor;
use iceoryx2_link_carrier::Announcement;
use iceoryx2_link_carrier::PeerId;

use crate::carrier::FakeCarrier;

pub(super) type Announced = BTreeMap<PeerId, BTreeSet<ServiceDescriptor>>;
pub(super) type Inboxes = BTreeMap<ServiceDescriptor, BTreeMap<PeerId, VecDeque<Vec<u8>>>>;

#[derive(Default)]
pub(super) struct State {
    pub(super) announced: RefCell<Announced>,
    pub(super) inboxes: RefCell<Inboxes>,
    /// Moved on every announcement.
    pub(super) generation: Cell<u64>,
    /// The wake of each peer that attached one.
    pub(super) wakes: RefCell<BTreeMap<PeerId, WakeHandle>>,
}

impl State {
    /// Records `from`'s announcement in the view every other peer lists,
    /// and wakes them.
    pub(super) fn announce(&self, from: PeerId, announcement: Announcement) {
        {
            let mut announced = self.announced.borrow_mut();
            let announced = announced.entry(from).or_default();
            match announcement {
                Announcement::Offered { descriptor } => {
                    announced.insert(descriptor);
                }
                Announcement::Withdrawn { hash } => {
                    announced.retain(|descriptor| descriptor.hash != hash);
                }
            }
        }
        self.generation.set(self.generation.get().wrapping_add(1));
        self.wake_others(from);
    }

    /// Delivers bytes from `from` to every other peer with the channel
    /// open, and wakes each of them.
    pub(super) fn deliver(&self, descriptor: &ServiceDescriptor, from: PeerId, bytes: Vec<u8>) {
        let mut receivers = Vec::new();
        {
            let mut inboxes = self.inboxes.borrow_mut();
            let Some(inboxes) = inboxes.get_mut(descriptor) else {
                return;
            };
            for (peer, inbox) in inboxes.iter_mut().filter(|(peer, _)| **peer != from) {
                inbox.push_back(bytes.clone());
                receivers.push(*peer);
            }
        }
        let wakes = self.wakes.borrow();
        for wake in receivers.iter().filter_map(|peer| wakes.get(peer)) {
            wake.signal();
        }
    }

    /// Signals the wakes of every peer but `except`.
    fn wake_others(&self, except: PeerId) {
        for (_, wake) in self
            .wakes
            .borrow()
            .iter()
            .filter(|(peer, _)| **peer != except)
        {
            wake.signal();
        }
    }
}

/// A fake substitute for a communication mechanism. Every carrier
/// joined to the same bus sees the others' announcements and bytes.
#[derive(Clone, Default)]
pub struct FakeBus {
    pub(super) state: Rc<State>,
}

impl FakeBus {
    /// Creates an empty bus.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a carrier participating on the bus as `peer`.
    pub fn join(&self, peer: PeerId) -> FakeCarrier {
        FakeCarrier::new(peer, self.clone())
    }
}
