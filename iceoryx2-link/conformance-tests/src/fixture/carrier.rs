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

use core::time::Duration;

use iceoryx2::service::service_hash::ServiceHash;
use iceoryx2_link_carrier::Carrier;

/// One communication mechanism and the carriers on it, each a peer.
pub trait CarrierFixture {
    type Carrier: Carrier;

    /// A fresh, isolated mechanism.
    fn new() -> Self;

    /// A carrier on the mechanism, a new peer.
    fn carrier(&mut self) -> Self::Carrier;

    /// Waits until the peers' channels of the service `hash` have
    /// propagated through the mechanism, or `timeout` passes.
    fn sync(&self, _hash: &ServiceHash, _timeout: Duration) -> bool {
        true
    }
}
