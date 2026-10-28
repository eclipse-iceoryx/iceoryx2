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

use alloc::collections::BTreeMap;

use iceoryx2::service::service_name::ServiceName;
use iceoryx2_link_adapter::{
    ClientRequestId, EndpointTypes, RequestDestination, RequestResponseEndpoints,
    ResponseDestination, ResponseKind, SampleBytesRef, SampleDestination, SampleLengths,
    TakeOutcome,
};

use crate::adapter::middleware::{
    EndpointId, FakeMiddleware, FakeRequestId, FakeRequestMessage, FakeResponseMessage, MessageRef,
};
use crate::adapter::{Error, FakeEndpointDescription};

/// A client and server on a [`FakeMiddleware`], the gateway's or an
/// application's.
pub struct FakeRequestResponseEndpoints {
    middleware: FakeMiddleware,
    name: ServiceName,
    id: EndpointId,
    request_header_size: usize,
    response_header_size: usize,
    /// The requests sent through the adapter contract, by the id the
    /// middleware gave them.
    sent: BTreeMap<FakeRequestId, ClientRequestId>,
}

impl FakeRequestResponseEndpoints {
    pub(super) fn new(
        middleware: FakeMiddleware,
        description: &FakeEndpointDescription,
        id: EndpointId,
    ) -> Self {
        let EndpointTypes::RequestResponse { request, response } = &description.types else {
            panic!(
                "the endpoints of {} are not request-response",
                description.settings.name
            );
        };
        Self {
            middleware,
            name: description.settings.name,
            id,
            request_header_size: request.user_header.size,
            response_header_size: response.user_header.size,
            sent: BTreeMap::new(),
        }
    }

    /// Sends a request to every other endpoint with the description.
    pub fn request(&self, message: MessageRef<'_>) -> FakeRequestId {
        self.middleware.send_request(&self.name, self.id, message)
    }

    /// The next pending request, if any.
    pub fn receive_request(&self) -> Option<(FakeRequestId, FakeRequestMessage)> {
        self.middleware.receive_request(&self.name, self.id)
    }

    /// Sends a response to the endpoint that sent the request `id`.
    pub fn respond(&self, id: FakeRequestId, message: MessageRef<'_>) {
        self.middleware.send_response(&self.name, id, message);
    }

    /// Ends the response stream of the request `id`.
    pub fn end_response_stream(&self, id: FakeRequestId) {
        self.middleware.end_response_stream(&self.name, id);
    }

    /// The next pending response or end, if any.
    pub fn receive_response(&self) -> Option<(FakeRequestId, FakeResponseMessage)> {
        self.middleware.receive_response(&self.name, self.id)
    }
}

/// Writes `message`, a header of `header_size` bytes then the payload,
/// into `destination`.
///
/// Returns `false` if the destination declined the lengths.
fn write<'a>(
    destination: impl SampleDestination<'a>,
    header_size: usize,
    message: MessageRef<'_>,
) -> bool {
    let (header, payload) = message.split_at(header_size.min(message.len()));
    let Some(locations) = destination.for_lengths(SampleLengths {
        header: header.len(),
        payload: payload.len(),
    }) else {
        return false;
    };
    locations.header.copy_from_slice(header);
    locations.payload.copy_from_slice(payload);
    true
}

impl RequestResponseEndpoints for FakeRequestResponseEndpoints {
    type Failure = Error;
    type ServerRequestId = FakeRequestId;

    fn send_request(
        &mut self,
        id: ClientRequestId,
        request: SampleBytesRef<'_>,
    ) -> Result<(), Self::Failure> {
        let sent = self.request(&[request.header, request.payload].concat());
        self.sent.insert(sent, id);
        Ok(())
    }

    fn take_response<'a>(
        &mut self,
        destination: impl ResponseDestination<'a>,
    ) -> Result<TakeOutcome<(ClientRequestId, ResponseKind)>, Self::Failure> {
        let Some((sent, response)) = self.receive_response() else {
            return Ok(TakeOutcome::Empty);
        };
        let Some(id) = self.sent.get(&sent).copied() else {
            return Ok(TakeOutcome::Skipped);
        };
        match response {
            FakeResponseMessage::Response(message) => {
                let Some(destination) = destination.of(id) else {
                    return Ok(TakeOutcome::Skipped);
                };
                if !write(destination, self.response_header_size, &message) {
                    return Ok(TakeOutcome::Declined);
                }
                Ok(TakeOutcome::Taken((id, ResponseKind::Response)))
            }
            FakeResponseMessage::End => {
                self.sent.remove(&sent);
                Ok(TakeOutcome::Taken((id, ResponseKind::End)))
            }
        }
    }

    fn take_request<'a>(
        &mut self,
        destination: impl RequestDestination<'a>,
    ) -> Result<TakeOutcome<FakeRequestId>, Self::Failure> {
        let Some((id, message)) = self.receive_request() else {
            return Ok(TakeOutcome::Empty);
        };
        if !write(destination, self.request_header_size, &message) {
            return Ok(TakeOutcome::Declined);
        }
        Ok(TakeOutcome::Taken(id))
    }

    fn send_response(
        &mut self,
        id: &FakeRequestId,
        response: SampleBytesRef<'_>,
    ) -> Result<(), Self::Failure> {
        self.respond(*id, &[response.header, response.payload].concat());
        Ok(())
    }

    fn end_response_stream(&mut self, id: FakeRequestId) -> Result<(), Self::Failure> {
        Self::end_response_stream(self, id);
        Ok(())
    }
}

impl Drop for FakeRequestResponseEndpoints {
    fn drop(&mut self) {
        self.middleware.leave(&self.name, self.id);
    }
}
