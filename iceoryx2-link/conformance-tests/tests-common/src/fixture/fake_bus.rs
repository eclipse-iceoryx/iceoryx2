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

use iceoryx2_link_carrier::PeerId;
use iceoryx2_link_conformance_tests::fixture::CarrierFixture;
use iceoryx2_link_testing::{FakeBus, FakeCarrier};

/// A fake bus and its peers.
pub(crate) struct FakeBusFixture {
    peers: u8,
    bus: FakeBus,
}

impl CarrierFixture for FakeBusFixture {
    type Carrier = FakeCarrier;

    fn new() -> Self {
        Self {
            peers: 0,
            bus: FakeBus::new(),
        }
    }

    fn carrier(&mut self) -> Self::Carrier {
        self.peers += 1;
        self.bus.join(PeerId::new([self.peers; PeerId::LENGTH]))
    }
}
