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

    /// Sends the next message of the request `id` to the opposing side.
    fn send_request(
        &mut self,
        id: ClientRequestId,
        request: OutboundRequest<'_, S>,
    ) -> Result<(), Self::SendError>;

    /// Receives the next message of the request stream into `loanable`.
    ///
    /// A request is identified by the id the local client gave the loaned
    /// request.
    #[allow(clippy::type_complexity)] // the full outcome stays visible to implementers
    fn receive_request<L: LoanableRequest<S>>(
        &mut self,
        loanable: L,
    ) -> Result<
        ReceiveOutcome<(ClientRequestId, InboundRequest<S, Self::RemoteId>)>,
        Self::ReceiveError,
    >;

    /// Sends the next message of the response stream of the request `id`
    /// to the opposing side.
    fn send_response(
        &mut self,
        id: ClientRequestId,
        response: OutboundResponse<'_, S>,
    ) -> Result<(), Self::SendError>;

    /// Receives the next message of the response stream into the loanable
    /// of its request.
    ///
    /// `loanable` yields `None` when the request is no longer held. The
    /// message is then consumed and reported as skipped.
    #[allow(clippy::type_complexity)] // the full outcome stays visible to implementers
    fn receive_response<L: LoanableResponse<S>>(
        &mut self,
        loanable: L,
    ) -> Result<
        ReceiveOutcome<(ClientRequestId, InboundResponse<S, Self::RemoteId>)>,
        Self::ReceiveError,
    >;
}
