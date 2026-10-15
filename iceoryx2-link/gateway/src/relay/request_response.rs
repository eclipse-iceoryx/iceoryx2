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
use core::marker::PhantomData;

use iceoryx2::service::Service;
use iceoryx2_link_adapter::{
    Adapter, ClientRequestId, EndpointDescription, EndpointTypes, Mapping,
    RequestResponseEndpoints, ResponseDestination, ResponseKind, SampleBytesRef, SampleShape,
    TakeOutcome, TranscodesSamples, Translator,
};
use iceoryx2_link_backend::relay::{ReceiveOutcome, RelayBuilder, RequestResponseRelay};
use iceoryx2_link_backend::service_description::{Identified, RequestResponseDescription};
use iceoryx2_link_backend::wire::request_response::{
    InboundRequest, InboundResponse, LoanableRequest, LoanableResponse, OutboundRequest,
    OutboundResponse, request_payload_bytes, response_payload_bytes,
};
use iceoryx2_link_backend::wire::sample::user_header_bytes;
use iceoryx2_log::{fail, fatal_panic, origin};

use crate::relay::transcoding::{Destination, PendingDecode, Transcoding};
use crate::relay::{CreationError, ReceiveError, SendError};

/// The id of the remote an endpoint description names.
type RemoteId<M> = <<M as Mapping>::EndpointSettings as Identified>::Id;

/// Creates relays over an adapter's request-response endpoints.
pub struct Builder<'a, S, A, M: Mapping, T: Translator<SampleShape>> {
    pub(super) adapter: &'a mut A,
    pub(super) translator: &'a T,
    pub(super) request_response_description: RequestResponseDescription<'a>,
    /// The endpoint description of the service opened on the middleware.
    pub(super) endpoint_description:
        &'a EndpointDescription<M::EndpointSettings, EndpointTypes<T::RemoteTypes>>,
    pub(super) _service: PhantomData<S>,
}

impl<S, A, M, T> RelayBuilder for Builder<'_, S, A, M, T>
where
    S: Service,
    M: Mapping,
    T: Translator<SampleShape, Transcoders: TranscodesSamples>,
    A: Adapter<
            EndpointSettings = M::EndpointSettings,
            EndpointTypes = EndpointTypes<T::RemoteTypes>,
        >,
{
    type CreationError = CreationError;
    type Relay = Relay<S, A::RequestResponseEndpoints, T, RemoteId<M>>;

    fn create(self) -> Result<Self::Relay, Self::CreationError> {
        let origin = origin!("Builder::create");
        let description = self.request_response_description;
        let types = description.types();

        let EndpointTypes::RequestResponse { request, response } = &self.endpoint_description.types
        else {
            fatal_panic!(
                from origin,
                "The endpoint of service {} is not a request-response endpoint", description.name()
            );
        };
        let request_transcoders = fail!(
            from origin,
            when self.translator.transcoders(&types.request, request),
            with CreationError::Translation,
            "No transcoders for the requests of service {}", description.name()
        );
        let response_transcoders = fail!(
            from origin,
            when self.translator.transcoders(&types.response, response),
            with CreationError::Translation,
            "No transcoders for the responses of service {}", description.name()
        );
        let endpoints = fail!(
            from origin,
            when self.adapter.request_response(self.endpoint_description),
            with CreationError::Endpoints,
            "Failed to open the endpoints of service {}", description.name()
        );

        Ok(Relay {
            endpoints,
            request_transcoding: Transcoding::new(request_transcoders, &types.request),
            response_transcoding: Transcoding::new(response_transcoders, &types.response),
            request_header_size: types.request.user_header.size,
            response_header_size: types.response.user_header.size,
            endpoint_id: self.endpoint_description.settings.id(),
            remote_requests: BTreeMap::new(),
            _service: PhantomData,
        })
    }
}

/// Relays requests and responses between the link and the gateway's
/// endpoints, transcoding them between the two forms.
pub struct Relay<S, E, X, Y>
where
    E: RequestResponseEndpoints,
    X: Translator<SampleShape, Transcoders: TranscodesSamples>,
{
    endpoints: E,
    request_transcoding: Transcoding<X::Transcoders>,
    response_transcoding: Transcoding<X::Transcoders>,
    request_header_size: usize,
    response_header_size: usize,
    /// The id of the endpoint description the relay was built for.
    endpoint_id: Y,
    /// Routing between a remote request received and the local request
    /// made by the link to local servers.
    remote_requests: BTreeMap<ClientRequestId, E::ServerRequestId>,
    _service: PhantomData<S>,
}

impl<S, E, X, Y> RequestResponseRelay<S> for Relay<S, E, X, Y>
where
    S: Service,
    E: RequestResponseEndpoints,
    X: Translator<SampleShape, Transcoders: TranscodesSamples>,
    Y: Clone,
{
    type RemoteId = Y;
    type SendError = SendError;
    type ReceiveError = ReceiveError;

    fn send_request(
        &mut self,
        id: ClientRequestId,
        request: OutboundRequest<'_, S>,
    ) -> Result<(), Self::SendError> {
        let origin = origin!("Relay::send_request");

        let request = match request {
            OutboundRequest::Request(request) => request,
            OutboundRequest::Abandoned => return Ok(()),
        };
        let local = SampleBytesRef {
            // SAFETY: the request belongs to the service this relay was built
            // for, whose description states the user header size.
            header: unsafe { user_header_bytes(request.user_header(), self.request_header_size) },
            payload: request_payload_bytes(request),
        };
        let wire = fail!(
            from origin,
            when self.request_transcoding.encode(local),
            "Failed to encode a request"
        );

        fail!(
            from origin,
            when self.endpoints.send_request(id, wire),
            with SendError::Endpoints,
            "Failed to send a request"
        );

        Ok(())
    }

    fn receive_request<L: LoanableRequest<S>>(
        &mut self,
        loanable: L,
    ) -> Result<ReceiveOutcome<(ClientRequestId, InboundRequest<S, Y>)>, Self::ReceiveError> {
        let origin = origin!("Relay::receive_request");

        let mut decode = self.request_transcoding.decode(loanable);
        let outcome = fail!(
            from origin,
            when self.endpoints.take_request(decode.input()),
            with ReceiveError::Endpoints,
            "Failed to take a request"
        );
        match outcome {
            TakeOutcome::Taken(server_request_id) => {
                let request = fail!(
                    from origin,
                    when decode.output(),
                    "Failed to decode a request"
                );
                let client_request_id = ClientRequestId::of(request.header());
                self.remote_requests
                    .insert(client_request_id, server_request_id);
                let request = InboundRequest::Request {
                    request,
                    origin: self.endpoint_id.clone(),
                };
                Ok(ReceiveOutcome::Sample((client_request_id, request)))
            }
            TakeOutcome::Declined => Err(decode.refusal()),
            TakeOutcome::Skipped => Ok(ReceiveOutcome::Skipped),
            TakeOutcome::Empty => Ok(ReceiveOutcome::Empty),
        }
    }

    fn send_response(
        &mut self,
        id: ClientRequestId,
        response: OutboundResponse<'_, S>,
    ) -> Result<(), Self::SendError> {
        let origin = origin!("Relay::send_response");

        let response = match response {
            OutboundResponse::Response(response) => response,
            OutboundResponse::End => {
                let Some(server_request_id) = self.remote_requests.remove(&id) else {
                    fail!(
                        from origin,
                        with SendError::UnknownRequest,
                        "No request {} was taken from the middleware", id
                    );
                };
                fail!(
                    from origin,
                    when self.endpoints.end_response_stream(server_request_id),
                    with SendError::Endpoints,
                    "Failed to end a response stream"
                );
                return Ok(());
            }
        };
        let Some(server_request_id) = self.remote_requests.get(&id) else {
            fail!(
                from origin,
                with SendError::UnknownRequest,
                "No request {} was taken from the middleware", id
            );
        };
        let local = SampleBytesRef {
            // SAFETY: the response belongs to the service this relay was
            // built for, whose description states the user header size.
            header: unsafe { user_header_bytes(response.user_header(), self.response_header_size) },
            payload: response_payload_bytes(response),
        };
        let wire = fail!(
            from origin,
            when self.response_transcoding.encode(local),
            "Failed to encode a response"
        );

        fail!(
            from origin,
            when self.endpoints.send_response(server_request_id, wire),
            with SendError::Endpoints,
            "Failed to send a response"
        );

        Ok(())
    }

    fn receive_response<L: LoanableResponse<S>>(
        &mut self,
        loanable: L,
    ) -> Result<ReceiveOutcome<(ClientRequestId, InboundResponse<S, Y>)>, Self::ReceiveError> {
        let origin = origin!("Relay::receive_response");

        // The take consumes the destination, so the decode it begins lives
        // here, where the relay can finish it once the take returns.
        let mut pending = None;
        let destination = ResponseDestinations {
            pending: &mut pending,
            transcoding: &mut self.response_transcoding,
            loanable,
            _service: PhantomData,
        };
        let outcome = fail!(
            from origin,
            when self.endpoints.take_response(destination),
            with ReceiveError::Endpoints,
            "Failed to take a response"
        );
        match (outcome, pending) {
            (TakeOutcome::Taken((id, ResponseKind::Response)), Some(decode)) => {
                let response = fail!(
                    from origin,
                    when decode.output(),
                    "Failed to decode a response"
                );
                Ok(ReceiveOutcome::Sample((
                    id,
                    InboundResponse::Response(response),
                )))
            }
            (TakeOutcome::Taken((id, ResponseKind::End)), _) => {
                let end = InboundResponse::End {
                    responder: self.endpoint_id.clone(),
                };
                Ok(ReceiveOutcome::Sample((id, end)))
            }
            (TakeOutcome::Declined, Some(decode)) => Err(decode.refusal()),
            (TakeOutcome::Skipped, _) => Ok(ReceiveOutcome::Skipped),
            (TakeOutcome::Empty, _) => Ok(ReceiveOutcome::Empty),
            (TakeOutcome::Taken((_, ResponseKind::Response)) | TakeOutcome::Declined, None) => {
                fatal_panic!(
                    from origin,
                    "The endpoints reported a response taken without asking for its destination. This cannot occur with correctly implemented endpoints."
                );
            }
        }
    }
}

/// The destinations of a take of a response. There is one per request the
/// ports hold, and it decodes into the loan of that request.
struct ResponseDestinations<'a, 't, S, L, X>
where
    S: Service,
    L: LoanableResponse<S>,
    X: TranscodesSamples,
{
    pending: &'a mut Option<PendingDecode<'t, L::Loanable, X>>,
    transcoding: &'t mut Transcoding<X>,
    loanable: L,
    _service: PhantomData<S>,
}

impl<'a, 't: 'a, S, L, X> ResponseDestination<'a> for ResponseDestinations<'a, 't, S, L, X>
where
    S: Service,
    L: LoanableResponse<S>,
    X: TranscodesSamples,
{
    type Destination = Destination<'a, 'a, L::Loanable>;

    fn of(self, id: ClientRequestId) -> Option<Destination<'a, 'a, L::Loanable>> {
        let loanable = self.loanable.of(id)?;
        let pending = self.pending.insert(self.transcoding.decode(loanable));
        Some(pending.input())
    }
}
