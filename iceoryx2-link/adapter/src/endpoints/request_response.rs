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

use iceoryx2_link_backend::wire::request_response::ClientRequestId;

use super::{SampleDestination, TakeOutcome};
use crate::SampleBytesRef;

/// Where a taken request's bytes are written.
pub trait RequestDestination<'a>: SampleDestination<'a> {}

impl<'a, D: SampleDestination<'a>> RequestDestination<'a> for D {}

/// Mechanism to retrieve the response destination for a specific request.
pub trait ResponseDestination<'a> {
    // Where a taken response's bytes are written.
    type Destination: SampleDestination<'a>;

    /// The destination for a response to `id`.
    ///
    /// Returns `None` if the request is no longer held.
    fn of(self, id: ClientRequestId) -> Option<Self::Destination>;
}

impl<'a, D, F> ResponseDestination<'a> for F
where
    D: SampleDestination<'a>,
    F: FnOnce(ClientRequestId) -> Option<D>,
{
    type Destination = D;

    fn of(self, id: ClientRequestId) -> Option<D> {
        self(id)
    }
}

/// The possible data that can be provided as a response.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResponseKind {
    /// A response. It was written into the destination.
    Response,
    /// The end of the stream. Nothing was written.
    End,
}

/// The gateway's client and server on the middleware.
pub trait RequestResponseEndpoints {
    type Failure: Error;
    /// The middleware's identity of a request its server took.
    type ServerRequestId;

    /// Sends the bytes of the request `id` to every other server with the
    /// same description.
    fn send_request(
        &mut self,
        id: ClientRequestId,
        request: SampleBytesRef<'_>,
    ) -> Result<(), Self::Failure>;

    /// Takes the pending response or end of a stream, if any, in the
    /// middleware's wire form into the locations `destination` provides
    /// for its request.
    ///
    /// `destination` yields `None` when the request is no longer held.
    /// The response is then dropped and reported as skipped.
    fn take_response<'a>(
        &mut self,
        destination: impl ResponseDestination<'a>,
    ) -> Result<TakeOutcome<(ClientRequestId, ResponseKind)>, Self::Failure>;

    /// Takes the pending request, if any, in the middleware's wire form into
    /// the locations `destination` provides for its lengths.
    fn take_request<'a>(
        &mut self,
        destination: impl RequestDestination<'a>,
    ) -> Result<TakeOutcome<Self::ServerRequestId>, Self::Failure>;

    /// Sends the bytes of a response to the request `id` the server took.
    fn send_response(
        &mut self,
        id: &Self::ServerRequestId,
        response: SampleBytesRef<'_>,
    ) -> Result<(), Self::Failure>;

    /// Ends the response stream of the request `id` the server took.
    fn end_response_stream(&mut self, id: Self::ServerRequestId) -> Result<(), Self::Failure>;
}
