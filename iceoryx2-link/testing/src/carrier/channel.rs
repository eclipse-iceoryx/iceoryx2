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
use alloc::vec::Vec;

use iceoryx2_link_backend::service_description::ServiceDescriptor;
use iceoryx2_link_backend::wire::sample::LoanableSample;
use iceoryx2_link_carrier::PeerId;
use iceoryx2_link_carrier::{Channel, ReceiveError, header_size, populate};
use iceoryx2_log::{fail, origin};

use crate::carrier::{Error, FakeBus};

/// A channel over a [`FakeBus`]. Frames reach every other peer that has the
/// channel open.
pub struct FakeChannel {
    peer: PeerId,
    descriptor: ServiceDescriptor,
    header_size: usize,
    bus: FakeBus,
}

impl FakeChannel {
    pub(super) fn new(peer: PeerId, descriptor: ServiceDescriptor, bus: FakeBus) -> Self {
        Self {
            peer,
            header_size: header_size(&descriptor),
            descriptor,
            bus,
        }
    }
}

impl Channel for FakeChannel {
    type Error = Error;

    fn send(&mut self, header: &[u8], payload: &[u8]) -> Result<(), Self::Error> {
        let mut bytes = Vec::with_capacity(header.len() + payload.len());
        bytes.extend_from_slice(header);
        bytes.extend_from_slice(payload);
        self.bus.state.deliver(&self.descriptor, self.peer, bytes);
        Ok(())
    }

    fn receive<L: LoanableSample>(
        &mut self,
        loanable: L,
    ) -> Result<Option<L::Sample>, ReceiveError<Self::Error>> {
        let origin = origin!("FakeChannel::receive");

        let bytes = self
            .bus
            .state
            .inboxes
            .borrow_mut()
            .get_mut(&self.descriptor)
            .and_then(|inboxes| inboxes.get_mut(&self.peer))
            .and_then(VecDeque::pop_front);
        let Some(bytes) = bytes else {
            return Ok(None);
        };
        let sample = fail!(
            from origin,
            when populate(self.header_size, &bytes, loanable),
            to ReceiveError<Error>,
            "Dropped a frame of {} bytes", bytes.len()
        );
        Ok(Some(sample))
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
