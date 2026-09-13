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
use core::fmt::Display;

use iceoryx2::service::Service;
use iceoryx2::service::service_hash::ServiceHash;

use crate::description::ServiceDescription;
use crate::generation::Generation;
use crate::relay::{EventRelay, PublishSubscribeRelay, RelayFactory};
use crate::resolver::Resolver;

/// What identifies a description in a backend's listing.
pub type RemoteId<S, B> = <B as Backend<S>>::RemoteId;
/// What a backend lists, and what a bridged service is opened as on the
/// opposing side.
pub type RemoteDescription<S, B> = <B as Backend<S>>::RemoteDescription;
/// Why a backend's resolver refuses a service.
pub type Refusal<S, B> = <B as Backend<S>>::Refusal;
/// What a listing hands each remote to.
pub type OnRemote<'a, S, B> = dyn FnMut(&RemoteId<S, B>, &RemoteDescription<S, B>) + 'a;

/// A change in what this side offers the opposing side.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Announcement<'a> {
    /// The service is bridged and offered under this description.
    Offered(&'a ServiceDescription),
    /// The service is no longer offered.
    Withdrawn(ServiceHash),
}

/// A link's view of the opposing side of the boundary.
pub trait Backend<S: Service> {
    type ListError: Error;
    type AnnouncementError: Error;
    /// What identifies a description in the opposing side's listing.
    type RemoteId: Ord + Clone + Display;
    /// What the opposing side lists, and what a bridged service is
    /// opened as there.
    type RemoteDescription: Clone + PartialEq + 'static;
    /// Why a service is refused.
    type Refusal: Display + PartialEq;
    type Resolver<'a>: Resolver<
            S,
            RemoteId = Self::RemoteId,
            RemoteDescription = Self::RemoteDescription,
            Refusal = Self::Refusal,
        >
    where
        Self: 'a;
    type PublishSubscribeRelay: PublishSubscribeRelay<S>;
    type EventRelay: EventRelay<S>;
    type RelayFactory<'a>: RelayFactory<
            S,
            RemoteDescription = RemoteDescription<S, Self>,
            PublishSubscribeRelay = Self::PublishSubscribeRelay,
            EventRelay = Self::EventRelay,
        >
    where
        Self: 'a;

    /// The generation of the opposing side's listing.
    ///
    /// Moves whenever the listing changed, and never otherwise. Untracked
    /// when the backend cannot tell, the listing then counts as changed
    /// every time.
    fn generation(&self) -> Generation {
        Generation::Untracked
    }

    /// Lists what the opposing side offers.
    ///
    /// Hands every remote to `on_remote` once, with what identifies it
    /// there. An error means the listing is incomplete.
    fn list(&self, on_remote: &mut OnRemote<'_, S, Self>) -> Result<(), Self::ListError>;

    /// The resolver that decides, service by service, whether and how
    /// it is bridged.
    fn resolver(&self) -> Self::Resolver<'_>;

    /// Tells the opposing side of a change in what this side offers.
    ///
    /// Ok means the opposing side has been told, an error that it has
    /// not and the same announcement may be made again. A backend whose
    /// opposing side learns of services another way returns Ok.
    fn announce(&mut self, announcement: Announcement<'_>) -> Result<(), Self::AnnouncementError>;

    /// The factory the relays of bridged services are created with.
    fn relay_factory(&self) -> Self::RelayFactory<'_>;
}
