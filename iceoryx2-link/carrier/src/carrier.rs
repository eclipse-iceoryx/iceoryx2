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

use core::error::Error;

use iceoryx2_bb_elementary::generation::Generation;
use iceoryx2_link_backend::service_description::ServiceDescriptor;

use crate::{Announcement, Channel, Offer};

/// The abstraction over the communication mechanism connecting `iceoryx2`
/// systems.
pub trait Carrier {
    type AnnouncementError: Error;
    type ListError: Error;
    type ChannelError: Error;
    type Channel: Channel;

    /// Makes a change to this tunnel's offers visible to peers.
    fn announce(&mut self, announcement: Announcement) -> Result<(), Self::AnnouncementError>;

    /// The generation of the peers' offers. A tracked one moves
    /// whenever they may have changed since the last call and holds still
    /// while they have not. Untracked, every listing is taken as changed.
    fn generation(&self) -> Generation {
        Generation::Untracked
    }

    /// Calls `callback` once for each offer of each peer.
    fn offers(&self, callback: &mut dyn FnMut(Offer)) -> Result<(), Self::ListError>;

    /// The byte channel for the described service.
    fn open_channel(
        &mut self,
        descriptor: &ServiceDescriptor,
    ) -> Result<Self::Channel, Self::ChannelError>;
}
