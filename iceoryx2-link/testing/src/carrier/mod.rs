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

mod bus;
mod channel;

pub use bus::FakeBus;
use channel::FakeChannel;
pub use channel::{FakeEventChannel, FakeSampleChannel};

use alloc::collections::VecDeque;

use iceoryx2_bb_elementary::generation::Generation;
use iceoryx2_link_backend::service_description::ServiceDescriptor;
use iceoryx2_link_backend::{Reactive, WakeHandle};
use iceoryx2_link_carrier::PeerId;
use iceoryx2_link_carrier::{Announcement, Carrier, Offer};

/// Cannot occur, the bus never fails.
#[derive(Debug)]
pub enum Error {}

impl core::fmt::Display for Error {
    fn fmt(&self, _: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match *self {}
    }
}

impl core::error::Error for Error {}

/// A carrier over a [`FakeBus`].
pub struct FakeCarrier {
    peer: PeerId,
    bus: FakeBus,
}

impl FakeCarrier {
    pub(super) fn new(peer: PeerId, bus: FakeBus) -> Self {
        Self { peer, bus }
    }
}

impl Carrier for FakeCarrier {
    type AnnouncementError = Error;
    type ListError = Error;
    type ChannelError = Error;
    type SampleChannel = FakeSampleChannel;
    type EventChannel = FakeEventChannel;

    fn announce(&mut self, announcement: Announcement) -> Result<(), Self::AnnouncementError> {
        self.bus.state.announce(self.peer, announcement);
        Ok(())
    }

    fn generation(&self) -> Generation {
        Generation::At(self.bus.state.generation.get())
    }

    fn offers(&self, callback: &mut dyn FnMut(Offer)) -> Result<(), Self::ListError> {
        let announced = self.bus.state.announced.borrow();
        for (peer, announced) in announced.iter().filter(|(peer, _)| **peer != self.peer) {
            for descriptor in announced {
                callback(Offer {
                    peer: *peer,
                    descriptor: descriptor.clone(),
                });
            }
        }
        Ok(())
    }

    fn open_sample_channel(
        &mut self,
        descriptor: &ServiceDescriptor,
    ) -> Result<Self::SampleChannel, Self::ChannelError> {
        Ok(FakeSampleChannel(self.open(descriptor)))
    }

    fn open_event_channel(
        &mut self,
        descriptor: &ServiceDescriptor,
    ) -> Result<Self::EventChannel, Self::ChannelError> {
        Ok(FakeEventChannel(self.open(descriptor)))
    }
}

impl FakeCarrier {
    /// Opens this peer's inbox on the descriptor's channel.
    fn open(&mut self, descriptor: &ServiceDescriptor) -> FakeChannel {
        self.bus
            .state
            .inboxes
            .borrow_mut()
            .entry(descriptor.clone())
            .or_default()
            .insert(self.peer, VecDeque::new());
        FakeChannel::new(self.peer, descriptor.clone(), self.bus.clone())
    }
}

impl Reactive for FakeCarrier {
    fn attach(&mut self, wake: WakeHandle) {
        self.bus.state.wakes.borrow_mut().insert(self.peer, wake);
    }
}
