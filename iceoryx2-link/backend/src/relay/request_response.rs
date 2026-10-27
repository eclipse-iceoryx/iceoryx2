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

use iceoryx2::service::Service;

use crate::relay::ReceiveOutcome;
use crate::wire::request_response::{
    ClientRequestId, InboundRequest, InboundResponse, LoanableRequest, LoanableResponse,
    OutboundRequest, OutboundResponse,
};

/// Relays requests and responses over the backend.
pub trait RequestResponseRelay<S: Service> {
    /// What identifies a remote on the opposing side.
    type RemoteId;
    type SendError: Error;
    type ReceiveError: Error;

    /// Sends the request `id` of a local client, or its abandonment, to
    /// the opposing side.
    fn send_request(
        &mut self,
        id: ClientRequestId,
        request: OutboundRequest<'_, S>,
    ) -> Result<(), Self::SendError>;

    /// Receives one request from the opposing side, or the abandonment of
    /// one, into the request `loanable` loans.
    ///
    /// The request is returned with the id the link's own client gave the
    /// loaned request, so the responses can be routed back to it.
    #[allow(clippy::type_complexity)] // the full outcome stays visible to implementers
    fn receive_request<L: LoanableRequest<S>>(
        &mut self,
        loanable: L,
    ) -> Result<
        ReceiveOutcome<(ClientRequestId, InboundRequest<S, Self::RemoteId>)>,
        Self::ReceiveError,
    >;

    /// Sends a response of a local server to the request `id`, or the end
    /// of its response stream, back to the remote client that sent the
    /// request.
    fn send_response(
        &mut self,
        id: ClientRequestId,
        response: OutboundResponse<'_, S>,
    ) -> Result<(), Self::SendError>;

    /// Receives one response from the opposing side, or the end of a
    /// response stream, into the loan `loanable` provides for its request.
    ///
    /// `loanable` yields `None` when the request is no longer held. The
    /// response is then consumed and reported as skipped.
    #[allow(clippy::type_complexity)] // the full outcome stays visible to implementers
    fn receive_response<L: LoanableResponse<S>>(
        &mut self,
        loanable: L,
    ) -> Result<
        ReceiveOutcome<(ClientRequestId, InboundResponse<S, Self::RemoteId>)>,
        Self::ReceiveError,
    >;
}
