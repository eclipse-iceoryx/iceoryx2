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

use alloc::collections::VecDeque;

use iceoryx2_link_backend::service_description::ServiceDescriptor;
use iceoryx2_link_carrier::Channel;
use iceoryx2_link_carrier::{Frame, PeerId};

use crate::carrier::{Error, FakeBus};

/// A channel over an [`FakeBus`]. Frames reach every other peer that has the
/// channel open.
pub struct FakeChannel {
    peer: PeerId,
    descriptor: ServiceDescriptor,
    bus: FakeBus,
}

impl FakeChannel {
    pub(super) fn new(peer: PeerId, descriptor: ServiceDescriptor, bus: FakeBus) -> Self {
        Self {
            peer,
            descriptor,
            bus,
        }
    }
}

impl Channel for FakeChannel {
    type Error = Error;

    fn send(&mut self, frame: Frame<'_>) -> Result<(), Self::Error> {
        let frame = frame.to_bytes();
        self.bus.state.deliver(&self.descriptor, self.peer, frame);
        Ok(())
    }

    fn receive<R>(&mut self, on_frame: impl FnOnce(&[u8]) -> R) -> Result<Option<R>, Self::Error> {
        let frame = self
            .bus
            .state
            .inboxes
            .borrow_mut()
            .get_mut(&self.descriptor)
            .and_then(|inboxes| inboxes.get_mut(&self.peer))
            .and_then(VecDeque::pop_front);
        Ok(frame.map(|frame| on_frame(&frame)))
    }
}

impl Drop for FakeChannel {
    fn drop(&mut self) {
        let mut inboxes = self.bus.state.inboxes.borrow_mut();
        if let Some(peers) = inboxes.get_mut(&self.descriptor) {
            peers.remove(&self.peer);
            if peers.is_empty() {
                inboxes.remove(&self.descriptor);
            }
        }
    }
}
