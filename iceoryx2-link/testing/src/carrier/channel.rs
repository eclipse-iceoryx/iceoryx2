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

use iceoryx2::port::event_id::EventId;
use iceoryx2_link_backend::service_description::ServiceDescriptor;
use iceoryx2_link_backend::wire::event::{decode, encode};
use iceoryx2_link_backend::wire::sample::SampleBytesRef;
use iceoryx2_link_carrier::PeerId;
use iceoryx2_link_carrier::{EventChannel, EventReceiveError, SampleChannel};
use iceoryx2_log::{fail, origin};

use crate::carrier::{Error, FakeBus};

/// One peer's inbox on a [`FakeBus`] for one service. Bytes reach every
/// other peer that has the channel open.
pub(super) struct FakeChannel {
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

    /// Delivers the bytes to every other peer with the channel open.
    fn deliver(&self, bytes: &[&[u8]]) {
        let mut joined = Vec::with_capacity(bytes.iter().map(|slice| slice.len()).sum());
        for slice in bytes {
            joined.extend_from_slice(slice);
        }
        self.bus.state.deliver(&self.descriptor, self.peer, joined);
    }

    /// The next pending bytes, if any.
    fn pop(&self) -> Option<Vec<u8>> {
        self.bus
            .state
            .inboxes
            .borrow_mut()
            .get_mut(&self.descriptor)
            .and_then(|inboxes| inboxes.get_mut(&self.peer))
            .and_then(VecDeque::pop_front)
    }
}

/// The samples of one service over a [`FakeBus`].
pub struct FakeSampleChannel {
    channel: FakeChannel,
    /// The bytes last received, held until the next receive.
    received: Vec<u8>,
}

impl FakeSampleChannel {
    pub(super) fn new(channel: FakeChannel) -> Self {
        Self {
            channel,
            received: Vec::new(),
        }
    }
}

impl SampleChannel for FakeSampleChannel {
    type Error = Error;

    fn send(&mut self, sample: SampleBytesRef<'_>) -> Result<(), Self::Error> {
        self.channel.deliver(&[sample.header, sample.payload]);
        Ok(())
    }

    fn receive(&mut self) -> Result<Option<&[u8]>, Self::Error> {
        match self.channel.pop() {
            Some(bytes) => {
                self.received = bytes;
                Ok(Some(&self.received))
            }
            None => Ok(None),
        }
    }
}

/// The ids of one event service over a [`FakeBus`].
pub struct FakeEventChannel(pub(super) FakeChannel);

impl EventChannel for FakeEventChannel {
    type Error = Error;

    fn send(&mut self, id: EventId) -> Result<(), Self::Error> {
        self.0.deliver(&[&encode(id)]);
        Ok(())
    }

    fn receive(&mut self) -> Result<Option<EventId>, EventReceiveError<Self::Error>> {
        let origin = origin!("FakeEventChannel::receive");

        let Some(bytes) = self.0.pop() else {
            return Ok(None);
        };
        let id = fail!(
            from origin,
            when decode(&bytes),
            to EventReceiveError<Error>,
            "Dropped {} bytes that are not an event id", bytes.len()
        );
        Ok(Some(id))
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
