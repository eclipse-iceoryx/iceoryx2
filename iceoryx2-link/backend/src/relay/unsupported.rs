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
use core::marker::PhantomData;

use iceoryx2::port::event_id::EventId;
use iceoryx2::service::Service;

use crate::Never;
use crate::relay::{EventRelay, PublishSubscribeRelay, RelayBuilder};
use crate::wire::publish_subscribe::Sample;
use crate::wire::sample::LoanableSample;

/// The backend does not bridge the pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Unsupported;

impl core::fmt::Display for Unsupported {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Unsupported")
    }
}

impl Error for Unsupported {}

/// The relay of a pattern the backend does not bridge, never built.
pub struct UnsupportedRelay<S>(Never, PhantomData<S>);

impl<S: Service> PublishSubscribeRelay<S> for UnsupportedRelay<S> {
    type SendError = Unsupported;
    type ReceiveError = Unsupported;

    fn send(&mut self, _: &Sample<S>) -> Result<(), Self::SendError> {
        match self.0 {}
    }

    fn receive<L: LoanableSample>(
        &mut self,
        _: L,
    ) -> Result<Option<L::Sample>, Self::ReceiveError> {
        match self.0 {}
    }
}

impl<S: Service> EventRelay<S> for UnsupportedRelay<S> {
    type SendError = Unsupported;
    type ReceiveError = Unsupported;

    fn send(&mut self, _: EventId) -> Result<(), Self::SendError> {
        match self.0 {}
    }

    fn receive(&mut self) -> Result<Option<EventId>, Self::ReceiveError> {
        match self.0 {}
    }
}

/// The builder of an [`UnsupportedRelay`], which fails. A backend's
/// resolver leaves the pattern out, so the link never asks.
pub struct UnsupportedRelayBuilder<S>(PhantomData<S>);

impl<S> UnsupportedRelayBuilder<S> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<S> Default for UnsupportedRelayBuilder<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S: Service> RelayBuilder for UnsupportedRelayBuilder<S> {
    type CreationError = Unsupported;
    type Relay = UnsupportedRelay<S>;

    fn create(self) -> Result<Self::Relay, Self::CreationError> {
        Err(Unsupported)
    }
}
