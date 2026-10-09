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
use crate::relay::{
    EventRelay, PublishSubscribeRelay, ReceiveOutcome, RelayBuilder, RequestResponseRelay,
};
use crate::wire::publish_subscribe::Sample;
use crate::wire::request_response::{
    ClientRequestId, InboundRequest, InboundResponse, LoanableRequest, LoanableResponse,
    OutboundRequest, OutboundResponse,
};
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
pub struct UnsupportedRelay<S, RemoteId = Never>(Never, PhantomData<(S, RemoteId)>);

impl<S: Service, RemoteId> PublishSubscribeRelay<S> for UnsupportedRelay<S, RemoteId> {
    type SendError = Unsupported;
    type ReceiveError = Unsupported;

    fn send(&mut self, _: &Sample<S>) -> Result<(), Self::SendError> {
        match self.0 {}
    }

    fn receive<L: LoanableSample>(
        &mut self,
        _: L,
    ) -> Result<ReceiveOutcome<L::InitializedSample>, Self::ReceiveError> {
        match self.0 {}
    }
}

impl<S: Service, RemoteId> EventRelay<S> for UnsupportedRelay<S, RemoteId> {
    type SendError = Unsupported;
    type ReceiveError = Unsupported;

    fn send(&mut self, _: EventId) -> Result<(), Self::SendError> {
        match self.0 {}
    }

    fn receive(&mut self) -> Result<Option<EventId>, Self::ReceiveError> {
        match self.0 {}
    }
}

impl<S: Service, RemoteId> RequestResponseRelay<S> for UnsupportedRelay<S, RemoteId> {
    type RemoteId = RemoteId;
    type SendError = Unsupported;
    type ReceiveError = Unsupported;

    fn send_request(
        &mut self,
        _: ClientRequestId,
        _: OutboundRequest<'_, S>,
    ) -> Result<(), Unsupported> {
        match self.0 {}
    }

    fn receive_request<L: LoanableRequest<S>>(
        &mut self,
        _: L,
    ) -> Result<ReceiveOutcome<(ClientRequestId, InboundRequest<S, RemoteId>)>, Unsupported> {
        match self.0 {}
    }

    fn send_response(
        &mut self,
        _: ClientRequestId,
        _: OutboundResponse<'_, S>,
    ) -> Result<(), Unsupported> {
        match self.0 {}
    }

    fn receive_response<L: LoanableResponse<S>>(
        &mut self,
        _: L,
    ) -> Result<ReceiveOutcome<(ClientRequestId, InboundResponse<S, RemoteId>)>, Unsupported> {
        match self.0 {}
    }
}

/// The builder of an [`UnsupportedRelay`], which fails. A backend's
/// resolver leaves the pattern out, so the link never asks.
pub struct UnsupportedRelayBuilder<S, RemoteId = Never>(PhantomData<(S, RemoteId)>);

impl<S, RemoteId> UnsupportedRelayBuilder<S, RemoteId> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<S, RemoteId> Default for UnsupportedRelayBuilder<S, RemoteId> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S: Service, RemoteId> RelayBuilder for UnsupportedRelayBuilder<S, RemoteId> {
    type CreationError = Unsupported;
    type Relay = UnsupportedRelay<S, RemoteId>;

    fn create(self) -> Result<Self::Relay, Self::CreationError> {
        Err(Unsupported)
    }
}
