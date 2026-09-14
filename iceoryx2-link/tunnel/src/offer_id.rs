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

use iceoryx2::service::service_hash::ServiceHash;
use iceoryx2_link_carrier::{Offer, PeerId};

/// What identifies one peer's offer of a service in the peers' listing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct OfferId {
    pub peer: PeerId,
    pub hash: ServiceHash,
}

impl From<&Offer> for OfferId {
    fn from(offer: &Offer) -> Self {
        Self {
            peer: offer.peer,
            hash: offer.descriptor.hash,
        }
    }
}

impl core::fmt::Display for OfferId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{} on {}", self.hash, self.peer)
    }
}
