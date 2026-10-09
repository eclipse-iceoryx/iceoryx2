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

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::vec::Vec;

use iceoryx2::node::Node;
use iceoryx2::port::LoanError;
use iceoryx2::prelude::AllocationStrategy;
use iceoryx2::service::Service;
use iceoryx2::service::builder::request_response;
use iceoryx2::service::service_name::ServiceName;
use iceoryx2_link_backend::relay::ReceiveOutcome;
use iceoryx2_link_backend::service_description::{
    RequestResponseSettings, RequestResponseTypes, TypeIdentifier,
};
use iceoryx2_link_backend::wire::request_response::{
    ActiveRequest, Client, ClientRequestId, InboundRequest, InboundResponse, OutboundRequest,
    OutboundResponse, PendingResponse, RequestMutUninit, ResponseMut, ResponseMutUninit, Server,
};
use iceoryx2_link_backend::wire::sample::{Header, LoanableSample, Payload, UnloanedSample};
use iceoryx2_log::{fail, origin, trace, warn};

use super::number_of_elements;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum CreationError {
    TypeDetails,
    Service,
    Server,
    Client,
}

impl core::fmt::Display for CreationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "CreationError::{self:?}")
    }
}

impl core::error::Error for CreationError {}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum SendError {
    RequestIngestion,
    RequestDelivery,
    ResponseIngestion,
    ResponseDelivery,
}

impl core::fmt::Display for SendError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "SendError::{self:?}")
    }
}

impl core::error::Error for SendError {}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum ReceiveError {
    RequestReceive,
    RequestPropagation,
    AbandonmentPropagation,
    ResponseReceive,
    ResponsePropagation,
    EndPropagation,
}

impl core::fmt::Display for ReceiveError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ReceiveError::{self:?}")
    }
}

impl core::error::Error for ReceiveError {}

/// A local client's request that went out. Dropping it ends the client's
/// stream.
struct LocalRequest<S: Service, RemoteId> {
    request: ActiveRequest<S>,
    /// The remotes whose end is still awaited.
    responders: BTreeSet<RemoteId>,
}

/// A remote client's request that was injected. Its pending response
/// receives what the local servers answer.
struct RemoteRequest<S: Service, RemoteId> {
    response: PendingResponse<S>,
    /// The remote who made the request.
    origin: RemoteId,
}

/// The link's own ports on a request-response service, opened from its
/// description with the exact types and settings.
///
/// `RemoteId` identifies a remote on the opposing side.
pub(crate) struct RequestResponsePorts<S: Service, RemoteId> {
    name: ServiceName,
    types: RequestResponseTypes,
    server: Server<S>,
    client: Client<S>,
    /// The requests of local clients that went out and still await
    /// responses.
    local_requests: BTreeMap<ClientRequestId, LocalRequest<S, RemoteId>>,
    /// The requests of remote clients that were injected and still await
    /// responses.
    remote_requests: BTreeMap<ClientRequestId, RemoteRequest<S, RemoteId>>,
    /// The remotes offering the service at the last reconcile.
    remotes: BTreeSet<RemoteId>,
}

impl<S: Service, RemoteId: Ord + Clone + core::fmt::Display> RequestResponsePorts<S, RemoteId> {
    pub(crate) fn open(
        node: &Node<S>,
        name: &ServiceName,
        settings: &RequestResponseSettings,
        types: &RequestResponseTypes,
    ) -> Result<Self, CreationError> {
        let origin = origin!("RequestResponsePorts::open");

        let (request_payload, request_header) = fail!(
            from origin,
            when types.request.type_details(),
            with CreationError::TypeDetails,
            "Request types of {} cannot be laid out as samples", name
        );
        let (response_payload, response_header) = fail!(
            from origin,
            when types.response.type_details(),
            with CreationError::TypeDetails,
            "Response types of {} cannot be laid out as samples", name
        );

        // SAFETY: the type details come from a description of this exact
        // service and compose into valid sample layouts, so the untyped
        // markers stand in for the real types.
        let mut builder = unsafe {
            node.service_builder(name)
                .request_response::<Payload, Payload>()
                .request_user_header::<Header>()
                .response_user_header::<Header>()
                .__internal_set_request_payload_type_details(&request_payload)
                .__internal_set_request_header_type_details(&request_header)
                .__internal_set_response_payload_type_details(&response_payload)
                .__internal_set_response_header_type_details(&response_header)
        };
        // A flatbuffer payload's schema is stored on create and verified on open.
        if let TypeIdentifier::Flatbuffer(schema) = &types.request.payload.identifier {
            // SAFETY: the schema describes the payload the type details name.
            builder = unsafe { builder.__internal_request_flatbuffer_schema(schema.bytes()) };
        }
        if let TypeIdentifier::Flatbuffer(schema) = &types.response.payload.identifier {
            // SAFETY: the schema describes the payload the type details name.
            builder = unsafe { builder.__internal_response_flatbuffer_schema(schema.bytes()) };
        }
        let service = fail!(
            from origin,
            when apply_settings(builder, settings).open_or_create(),
            with CreationError::Service,
            "Failed to open or create {}", name
        );
        let server = fail!(
            from origin,
            when service
                .server_builder()
                .allocation_strategy(AllocationStrategy::PowerOfTwo)
                .create(),
            with CreationError::Server,
            "Failed to create the server of {}", name
        );
        let client = fail!(
            from origin,
            when service
                .client_builder()
                .allocation_strategy(AllocationStrategy::PowerOfTwo)
                .create(),
            with CreationError::Client,
            "Failed to create the client of {}", name
        );

        Ok(Self {
            name: *name,
            types: types.clone(),
            server,
            client,
            local_requests: BTreeMap::new(),
            remote_requests: BTreeMap::new(),
            remotes: BTreeSet::new(),
        })
    }

    /// Receives requests from local clients and propagates them to remote
    /// servers.
    ///
    /// The request is held until either:
    /// * the local client drops it, which is propagated as an abandonment
    /// * every remote offering the service ended its response stream or
    ///   departed
    ///
    /// With no remote offering the service the request is dropped without
    /// being propagated.
    ///
    /// Requests the link's own client sent are skipped, so an injected
    /// request is not propagated back.
    ///
    /// Returns the number of requests propagated.
    pub(crate) fn receive_requests<E>(
        &mut self,
        mut propagate: impl FnMut(ClientRequestId, OutboundRequest<'_, S>) -> Result<(), E>,
    ) -> Result<u64, ReceiveError> {
        let origin = origin!("RequestResponsePorts::receive_requests");

        let mut received = 0;
        loop {
            let request = fail!(
                from origin,
                when self.server.receive(),
                with ReceiveError::RequestReceive,
                "Failed to receive a request of {}", self.name
            );
            let Some(request) = request else {
                break;
            };

            // The link's own requests are what it injected from the
            // opposing side. Propagating them back would loop.
            if request.header().client_id() == self.client.id() {
                continue;
            }
            if self.remotes.is_empty() {
                trace!(from origin, "Dropped a request of {}, nobody on the opposing side serves it", self.name);
                continue;
            }

            let id = ClientRequestId::of(request.header());
            fail!(
                from origin,
                when propagate(id, OutboundRequest::Request(&request)),
                with ReceiveError::RequestPropagation,
                "Failed to propagate a request of {}", self.name
            );
            self.local_requests.insert(
                id,
                LocalRequest {
                    request,
                    responders: self.remotes.clone(),
                },
            );
            received += 1;
        }

        let abandoned: Vec<ClientRequestId> = self
            .local_requests
            .iter()
            .filter(|(_, local)| !local.request.is_connected())
            .map(|(id, _)| *id)
            .collect();
        for id in abandoned {
            fail!(
                from origin,
                when propagate(id, OutboundRequest::Abandoned),
                with ReceiveError::AbandonmentPropagation,
                "Failed to propagate the abandonment of a request of {}", self.name
            );
            self.local_requests.remove(&id);
        }

        Ok(received)
    }

    /// Receives requests from remote clients and sends them to local
    /// servers.
    ///
    /// Each request `ingest` writes is sent and held with the id it came
    /// with, until either:
    /// * the remote client abandons it, which ends it for the local servers
    /// * its origin departs
    ///
    /// Returns the number of requests sent.
    pub(crate) fn send_requests<E>(
        &mut self,
        mut ingest: impl FnMut(
            UnloanedSample<'_, RequestMutUninit<S>, LoanError>,
        ) -> Result<
            ReceiveOutcome<(ClientRequestId, InboundRequest<S, RemoteId>)>,
            E,
        >,
    ) -> Result<u64, SendError> {
        let origin = origin!("RequestResponsePorts::send_requests");
        let Self {
            name,
            types,
            client,
            remote_requests,
            ..
        } = self;

        let mut loan = |number_of_bytes| {
            let Some(number_of_elements) =
                number_of_elements(&types.request.payload, number_of_bytes)
            else {
                fail!(
                    from origin,
                    with LoanError::InternalFailure,
                    "A loan of {} bytes does not hold whole request elements of {}", number_of_bytes, name
                );
            };
            // SAFETY: the client was created for the untyped payload marker
            // with this service's real type details.
            unsafe { client.loan_custom_payload(number_of_elements) }
        };

        let mut sent = 0;
        loop {
            let outcome = fail!(
                from origin,
                when ingest(UnloanedSample::new(&types.request, &mut loan)),
                with SendError::RequestIngestion,
                "Failed to ingest a request for {}", name
            );
            match outcome {
                ReceiveOutcome::Sample((
                    id,
                    InboundRequest::Request {
                        request,
                        origin: from,
                    },
                )) => {
                    let response = fail!(
                        from origin,
                        when request.send(),
                        with SendError::RequestDelivery,
                        "Failed to send a request of {}", name
                    );
                    remote_requests.insert(
                        id,
                        RemoteRequest {
                            response,
                            origin: from,
                        },
                    );
                    sent += 1;
                }
                ReceiveOutcome::Sample((id, InboundRequest::Abandoned)) => {
                    remote_requests.remove(&id);
                }
                ReceiveOutcome::Skipped => continue,
                ReceiveOutcome::Empty => break,
            }
        }

        Ok(sent)
    }

    /// Receives responses from local servers and propagates them to the
    /// remote client that asked.
    ///
    /// Once every local server finished with a held request, its end is
    /// propagated and the request is released.
    ///
    /// Returns the number of responses propagated.
    pub(crate) fn receive_responses<E>(
        &mut self,
        mut propagate: impl FnMut(ClientRequestId, OutboundResponse<'_, S>) -> Result<(), E>,
    ) -> Result<u64, ReceiveError> {
        let origin = origin!("RequestResponsePorts::receive_responses");

        let mut received = 0;
        let mut ended = Vec::new();

        for (id, remote) in &self.remote_requests {
            // Checked before draining, so a disconnect seen here comes after
            // the last response was queued.
            let finished = !remote.response.is_connected();
            loop {
                // SAFETY: the client was created for the untyped payload
                // marker with this service's real type details.
                let response = fail!(
                    from origin,
                    when unsafe { remote.response.receive_custom_payload() },
                    with ReceiveError::ResponseReceive,
                    "Failed to receive a response of {}", self.name
                );
                let Some(response) = response else {
                    break;
                };
                fail!(
                    from origin,
                    when propagate(*id, OutboundResponse::Response(&response)),
                    with ReceiveError::ResponsePropagation,
                    "Failed to propagate a response of {}", self.name
                );
                received += 1;
            }

            if finished {
                ended.push(*id);
            }
        }

        for id in ended {
            fail!(
                from origin,
                when propagate(id, OutboundResponse::End),
                with ReceiveError::EndPropagation,
                "Failed to propagate the end of a response stream of {}", self.name
            );
            self.remote_requests.remove(&id);
        }

        Ok(received)
    }

    /// Receives responses from remote servers and sends them to the local
    /// client that asked.
    ///
    /// A remote server that ends its response stream is removed from the
    /// servers still expected to respond to the request. Once none is left,
    /// the request is released, which ends the stream for the local client.
    ///
    /// Returns the number of responses sent.
    pub(crate) fn send_responses<E>(
        &mut self,
        mut ingest: impl FnMut(
            &Self,
        ) -> Result<
            ReceiveOutcome<(ClientRequestId, InboundResponse<S, RemoteId>)>,
            E,
        >,
    ) -> Result<u64, SendError> {
        let origin = origin!("RequestResponsePorts::send_responses");

        let mut sent = 0;
        loop {
            let outcome = fail!(
                from origin,
                when ingest(self),
                with SendError::ResponseIngestion,
                "Failed to ingest a response for {}", self.name
            );
            match outcome {
                ReceiveOutcome::Sample((_, InboundResponse::Response(response))) => {
                    fail!(
                        from origin,
                        when response.send(),
                        with SendError::ResponseDelivery,
                        "Failed to send a response of {}", self.name
                    );
                    sent += 1;
                }
                ReceiveOutcome::Sample((id, InboundResponse::End { responder })) => {
                    self.end_response_stream(id, &responder);
                }
                ReceiveOutcome::Skipped => continue,
                ReceiveOutcome::Empty => break,
            }
        }

        Ok(sent)
    }

    /// Removes `responder` from the servers still expected to respond to the
    /// held request `id`, and releases the request once none is left.
    fn end_response_stream(&mut self, id: ClientRequestId, responder: &RemoteId) {
        let origin = origin!("RequestResponsePorts::end_response_stream");

        let Some(local) = self.local_requests.get_mut(&id) else {
            return;
        };
        if !local.responders.remove(responder) {
            return;
        }
        trace!(from origin, "Ended the response stream of {} from {}", self.name, responder);
        if local.responders.is_empty() {
            self.local_requests.remove(&id);
        }
    }

    /// The loanable response to the held request `id`, or `None` if no
    /// such request is held anymore.
    pub(crate) fn response_loan(
        &self,
        id: ClientRequestId,
    ) -> Option<impl LoanableSample<InitializedSample = ResponseMut<S>> + '_> {
        let origin = origin!("RequestResponsePorts::response_loan");

        let Some(local) = self.local_requests.get(&id) else {
            warn!(from origin, "Discarding a response of {} to a request no longer held", self.name);
            return None;
        };
        let request = &local.request;
        let name = &self.name;
        let payload = &self.types.response.payload;

        let loan = move |number_of_bytes| -> Result<ResponseMutUninit<S>, LoanError> {
            let Some(number_of_elements) = number_of_elements(payload, number_of_bytes) else {
                fail!(
                    from origin,
                    with LoanError::InternalFailure,
                    "A loan of {} bytes does not hold whole response elements of {}", number_of_bytes, name
                );
            };
            // SAFETY: the server was created for the untyped payload marker
            // with this service's real type details.
            unsafe { request.loan_custom_payload(number_of_elements) }
        };

        Some(UnloanedSample::new(&self.types.response, loan))
    }

    /// Updates the held requests to the remotes currently offering the
    /// service.
    ///
    /// A departed remote is removed from the servers still expected to
    /// respond to each local request, and the remote requests it originated
    /// are dropped.
    pub(crate) fn update_remotes(&mut self, remotes: BTreeSet<RemoteId>) {
        self.drop_departed_servers(&remotes);
        self.drop_departed_clients(&remotes);
        self.remotes = remotes;
    }

    /// Stops expecting responses of the servers gone from `remotes`, and
    /// releases the local requests no server is left for.
    fn drop_departed_servers(&mut self, remotes: &BTreeSet<RemoteId>) {
        let origin = origin!("RequestResponsePorts::drop_departed_servers");

        self.local_requests.retain(|_, local| {
            local
                .responders
                .retain(|responder| remotes.contains(responder));
            let awaited = !local.responders.is_empty();
            if !awaited {
                trace!(from origin, "Ended the response stream of {}, its responders departed", self.name);
            }
            awaited
        });
    }

    /// Releases the remote requests whose client is gone from `remotes`.
    fn drop_departed_clients(&mut self, remotes: &BTreeSet<RemoteId>) {
        let origin = origin!("RequestResponsePorts::drop_departed_clients");

        let departed: BTreeSet<RemoteId> = self.remotes.difference(remotes).cloned().collect();
        self.remote_requests.retain(|_, remote| {
            let held = !departed.contains(&remote.origin);
            if !held {
                trace!(from origin, "Dropped a request of {} from the departed {}", self.name, remote.origin);
            }
            held
        });
    }

    /// Releases every held local request, handing its id to `release`.
    pub(crate) fn drain_local_requests<E: core::fmt::Display>(
        &mut self,
        mut release: impl FnMut(ClientRequestId) -> Result<(), E>,
    ) {
        let origin = origin!("RequestResponsePorts::drain_local_requests");

        for (id, _) in core::mem::take(&mut self.local_requests) {
            if let Err(error) = release(id) {
                warn!(from origin, "Failed to release a local request of {}: {}", self.name, error);
            }
        }
    }

    /// Releases every held remote request, handing its id to `release`.
    pub(crate) fn drain_remote_requests<E: core::fmt::Display>(
        &mut self,
        mut release: impl FnMut(ClientRequestId) -> Result<(), E>,
    ) {
        let origin = origin!("RequestResponsePorts::drain_remote_requests");

        for (id, _) in core::mem::take(&mut self.remote_requests) {
            if let Err(error) = release(id) {
                warn!(from origin, "Failed to release a remote request of {}: {}", self.name, error);
            }
        }
    }
}

fn apply_settings<S: Service>(
    builder: request_response::Builder<Payload, Header, Payload, Header, S>,
    settings: &RequestResponseSettings,
) -> request_response::Builder<Payload, Header, Payload, Header, S> {
    builder
        .max_servers(settings.max_servers)
        .max_clients(settings.max_clients)
        .max_nodes(settings.max_nodes)
        .max_active_requests_per_client(settings.max_active_requests_per_client)
        .max_response_buffer_size(settings.max_response_buffer_size)
        .max_borrowed_responses_per_pending_response(
            settings.max_borrowed_responses_per_pending_response,
        )
        .max_loaned_requests(settings.max_loaned_requests)
        .enable_safe_overflow_for_requests(settings.safe_overflow_for_requests)
        .enable_safe_overflow_for_responses(settings.safe_overflow_for_responses)
        .enable_fire_and_forget_requests(settings.fire_and_forget_requests)
}

#[cfg(test)]
mod tests {
    use super::*;

    use iceoryx2::config::Config;
    use iceoryx2::node::NodeBuilder;
    use iceoryx2::port::client::Client as TypedClient;
    use iceoryx2::port::server::Server as TypedServer;
    use iceoryx2::service::local;
    use iceoryx2::service::messaging_pattern::MessagingPattern;
    use iceoryx2::service::port_factory::request_response::PortFactory;
    use iceoryx2::testing::{generate_isolated_config, generate_service_name};
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_link_backend::service_description::{
        PatternSettings, ServiceDescription, ServiceTypes,
    };
    use iceoryx2_link_backend::wire::request_response::{
        request_payload_bytes, response_payload_bytes,
    };
    use iceoryx2_link_backend::wire::sample::WritableSample;

    type RemoteId = u8;
    type RequestPayload = u64;
    type ResponsePayload = u32;

    /// A local application connected to the link's ports.
    struct Application {
        service: PortFactory<local::Service, RequestPayload, (), ResponsePayload, ()>,
        _node: Node<local::Service>,
    }

    impl Application {
        fn new(config: &Config, service_name: &ServiceName, fire_and_forget: bool) -> Self {
            let node = NodeBuilder::new()
                .config(config)
                .create::<local::Service>()
                .expect("node is created");
            let service = node
                .service_builder(service_name)
                .request_response::<RequestPayload, ResponsePayload>()
                .enable_fire_and_forget_requests(fire_and_forget)
                .create()
                .expect("service is created");
            Self {
                service,
                _node: node,
            }
        }

        fn client(&self) -> TypedClient<local::Service, RequestPayload, (), ResponsePayload, ()> {
            self.service
                .client_builder()
                .create()
                .expect("client is created")
        }

        /// A server receiving the requests sent from now on.
        fn server(&self) -> TypedServer<local::Service, RequestPayload, (), ResponsePayload, ()> {
            self.service
                .server_builder()
                .create()
                .expect("server is created")
        }
    }

    /// The ports under test.
    struct Sut {
        ports: RequestResponsePorts<local::Service, RemoteId>,
        _node: Node<local::Service>,
    }

    impl Sut {
        fn open(config: &Config, service_name: &ServiceName) -> Self {
            let node = NodeBuilder::new()
                .config(config)
                .create::<local::Service>()
                .expect("node is created");
            let static_config =
                local::Service::details(service_name, config, MessagingPattern::RequestResponse)
                    .expect("details are readable")
                    .expect("service exists")
                    .static_details;
            let description = ServiceDescription::load::<local::Service>(config, &static_config)
                .expect("carried pattern");
            let PatternSettings::RequestResponse(settings) = &description.settings().pattern else {
                panic!("a request-response service");
            };
            let ServiceTypes::RequestResponse(types) = description.types() else {
                panic!("a request-response service");
            };
            let ports = RequestResponsePorts::open(&node, service_name, settings, types)
                .expect("ports open on the existing service");
            Self { ports, _node: node }
        }
    }

    fn setup(fire_and_forget: bool) -> (Application, Sut) {
        let config = generate_isolated_config();
        let service_name = generate_service_name();
        let app = Application::new(&config, &service_name, fire_and_forget);
        let sut = Sut::open(&config, &service_name);
        (app, sut)
    }

    fn responders(remotes: &[RemoteId]) -> BTreeSet<RemoteId> {
        remotes.iter().copied().collect()
    }

    /// An id as a request of the opposing side would carry it. It is taken
    /// from a request of a client on another service, which the ports never
    /// see.
    fn remote_id() -> ClientRequestId {
        let app = Application::new(&generate_isolated_config(), &generate_service_name(), false);
        let pending = app.client().send_copy(0).expect("request is sent");
        ClientRequestId::of(pending.header())
    }

    /// Lets the link's server let go of its copies of the injected
    /// requests. The bridge does this each cycle before reading responses.
    fn release_injected_requests(ports: &mut RequestResponsePorts<local::Service, RemoteId>) {
        let (propagated, abandoned) = propagate_requests(ports);
        assert_that!(propagated, len 0);
        assert_that!(abandoned, len 0);
    }

    /// Propagates requests from the local application's client over the ports.
    ///
    /// Returns the payload that was propagated and the requests that were
    /// abandoned.
    fn propagate_requests(
        ports: &mut RequestResponsePorts<local::Service, RemoteId>,
    ) -> (Vec<(ClientRequestId, RequestPayload)>, Vec<ClientRequestId>) {
        let mut propagated = Vec::new();
        let mut abandoned = Vec::new();

        ports
            .receive_requests(|id, outbound| {
                match outbound {
                    OutboundRequest::Request(request) => {
                        let payload = RequestPayload::from_ne_bytes(
                            request_payload_bytes(request)
                                .try_into()
                                .expect("payload is a request"),
                        );
                        propagated.push((id, payload));
                    }
                    OutboundRequest::Abandoned => abandoned.push(id),
                }
                Ok::<(), ()>(())
            })
            .expect("receive succeeds");

        (propagated, abandoned)
    }

    /// Injects one request from `origin` with the provided `id` as if
    /// coming across the boundary.
    fn inject_request(
        ports: &mut RequestResponsePorts<local::Service, RemoteId>,
        id: ClientRequestId,
        origin: RemoteId,
        payload: RequestPayload,
    ) {
        let mut injected = false;
        let sent = ports
            .send_requests(|unloaned| {
                if injected {
                    return Ok::<_, ()>(ReceiveOutcome::Empty);
                }
                injected = true;
                let mut loaned = unloaned
                    .loan_forward(core::mem::size_of::<RequestPayload>())
                    .expect("one request fits");
                loaned
                    .as_mut()
                    .payload
                    .copy_from_slice(&payload.to_ne_bytes());
                // SAFETY: the payload was written above and the header
                // is zero sized.
                let request = unsafe { loaned.assume_init() };
                Ok(ReceiveOutcome::Sample((
                    id,
                    InboundRequest::Request { request, origin },
                )))
            })
            .expect("send succeeds");
        assert_that!(sent, eq 1);
    }

    /// Injects the abandonment of `id`.
    fn inject_abandonment(
        ports: &mut RequestResponsePorts<local::Service, RemoteId>,
        id: ClientRequestId,
    ) {
        let mut injected = false;
        ports
            .send_requests(|_| {
                if injected {
                    return Ok::<_, ()>(ReceiveOutcome::Empty);
                }
                injected = true;
                Ok(ReceiveOutcome::Sample((id, InboundRequest::Abandoned)))
            })
            .expect("send succeeds");
    }

    /// Propagates responses from the local application's server over the
    /// ports.
    ///
    /// Returns the payload that was propagated and the response streams that
    /// were ended.
    fn propagate_responses(
        ports: &mut RequestResponsePorts<local::Service, RemoteId>,
    ) -> (
        Vec<(ClientRequestId, ResponsePayload)>,
        Vec<ClientRequestId>,
    ) {
        let mut propagated = Vec::new();
        let mut ended = Vec::new();
        ports
            .receive_responses(|id, outbound| {
                match outbound {
                    OutboundResponse::Response(response) => {
                        let payload = ResponsePayload::from_ne_bytes(
                            response_payload_bytes(response)
                                .try_into()
                                .expect("payload is a response"),
                        );
                        propagated.push((id, payload));
                    }
                    OutboundResponse::End => ended.push(id),
                }
                Ok::<(), ()>(())
            })
            .expect("receive succeeds");
        (propagated, ended)
    }

    /// Injects one response to `id` as the opposing side would send it.
    /// Returns whether the ports took it.
    fn inject_response(
        ports: &mut RequestResponsePorts<local::Service, RemoteId>,
        id: ClientRequestId,
        payload: ResponsePayload,
    ) -> bool {
        let mut injected = false;
        let sent = ports
            .send_responses(|ports| {
                if injected {
                    return Ok::<_, ()>(ReceiveOutcome::Empty);
                }
                injected = true;
                let Some(unloaned) = ports.response_loan(id) else {
                    return Ok(ReceiveOutcome::Skipped);
                };
                let mut loaned = unloaned
                    .loan_forward(core::mem::size_of::<ResponsePayload>())
                    .expect("one response fits");
                loaned
                    .as_mut()
                    .payload
                    .copy_from_slice(&payload.to_ne_bytes());

                // SAFETY: the payload was written above and the header
                // is zero sized.
                let response = unsafe { loaned.assume_init() };
                Ok(ReceiveOutcome::Sample((
                    id,
                    InboundResponse::Response(response),
                )))
            })
            .expect("send succeeds");

        sent == 1
    }

    /// Injects the end of `responder`'s stream for `id`.
    fn inject_end(
        ports: &mut RequestResponsePorts<local::Service, RemoteId>,
        id: ClientRequestId,
        responder: RemoteId,
    ) {
        let mut injected = false;
        ports
            .send_responses(|_| {
                if injected {
                    return Ok::<_, ()>(ReceiveOutcome::Empty);
                }
                injected = true;
                Ok(ReceiveOutcome::Sample((
                    id,
                    InboundResponse::End { responder },
                )))
            })
            .expect("send succeeds");
    }

    #[test]
    fn a_local_request_is_propagated_and_held() {
        const FIRE_AND_FORGET: bool = false;
        const RESPONDER: RemoteId = 1;
        const REQUEST: RequestPayload = 42;

        let (app, mut sut) = setup(FIRE_AND_FORGET);
        let client = app.client();
        let pending = client.send_copy(REQUEST).expect("request is sent");

        sut.ports.update_remotes(responders(&[RESPONDER]));
        let (propagated, abandoned) = propagate_requests(&mut sut.ports);

        assert_that!(propagated, eq alloc::vec![(ClientRequestId::of(pending.header()), REQUEST)]);
        assert_that!(abandoned, len 0);
        assert_that!(pending.is_connected(), eq true);
    }

    #[test]
    fn a_local_request_nobody_serves_is_dropped() {
        const FIRE_AND_FORGET: bool = false;
        const REQUEST: RequestPayload = 42;

        let (app, mut sut) = setup(FIRE_AND_FORGET);
        let client = app.client();
        let pending = client.send_copy(REQUEST).expect("request is sent");

        let (propagated, _) = propagate_requests(&mut sut.ports);

        assert_that!(propagated, len 0);
        assert_that!(pending.is_connected(), eq false);
    }

    #[test]
    fn a_request_let_go_of_is_abandoned() {
        const FIRE_AND_FORGET: bool = false;
        const RESPONDER: RemoteId = 1;
        const REQUEST: RequestPayload = 42;

        let (app, mut sut) = setup(FIRE_AND_FORGET);
        let client = app.client();
        let pending = client.send_copy(REQUEST).expect("request is sent");
        let id = ClientRequestId::of(pending.header());
        sut.ports.update_remotes(responders(&[RESPONDER]));
        propagate_requests(&mut sut.ports);

        drop(pending);
        let (propagated, abandoned) = propagate_requests(&mut sut.ports);

        assert_that!(propagated, len 0);
        assert_that!(abandoned, eq alloc::vec![id]);
    }

    #[test]
    fn a_fired_and_forgotten_request_is_propagated_and_abandoned_at_once() {
        const FIRE_AND_FORGET: bool = true;
        const RESPONDER: RemoteId = 1;
        const REQUEST: RequestPayload = 42;

        let (app, mut sut) = setup(FIRE_AND_FORGET);
        let client = app.client();
        let id = {
            let pending = client.send_copy(REQUEST).expect("request is sent");
            ClientRequestId::of(pending.header())
        };

        sut.ports.update_remotes(responders(&[RESPONDER]));
        let (propagated, abandoned) = propagate_requests(&mut sut.ports);

        assert_that!(propagated, eq alloc::vec![(id, REQUEST)]);
        assert_that!(abandoned, eq alloc::vec![id]);
    }

    #[test]
    fn an_injected_request_reaches_the_local_server_and_is_not_propagated_back() {
        const FIRE_AND_FORGET: bool = false;
        const RESPONDER: RemoteId = 1;
        const REQUEST: RequestPayload = 7;

        let (app, mut sut) = setup(FIRE_AND_FORGET);
        let id = remote_id();

        let server = app.server();

        inject_request(&mut sut.ports, id, RESPONDER, REQUEST);

        let active = server
            .receive()
            .expect("receive succeeds")
            .expect("the injected request");
        assert_that!(*active.payload(), eq REQUEST);
        sut.ports.update_remotes(responders(&[RESPONDER]));
        let (propagated, _) = propagate_requests(&mut sut.ports);
        assert_that!(propagated, len 0);
    }

    #[test]
    fn responses_of_the_local_server_are_propagated_and_the_stream_ends_when_it_finishes() {
        const FIRE_AND_FORGET: bool = false;
        const RESPONDER: RemoteId = 1;
        const REQUEST: RequestPayload = 7;
        const RESPONSES: [ResponsePayload; 2] = [1, 2];

        let (app, mut sut) = setup(FIRE_AND_FORGET);
        let id = remote_id();
        let server = app.server();
        inject_request(&mut sut.ports, id, RESPONDER, REQUEST);
        release_injected_requests(&mut sut.ports);
        let active = server
            .receive()
            .expect("receive succeeds")
            .expect("the injected request");
        for response in RESPONSES {
            active.send_copy(response).expect("response is sent");
        }

        let (propagated, ended) = propagate_responses(&mut sut.ports);
        assert_that!(propagated, eq alloc::vec![(id, RESPONSES[0]), (id, RESPONSES[1])]);
        assert_that!(ended, len 0);

        drop(active);
        let (propagated, ended) = propagate_responses(&mut sut.ports);
        assert_that!(propagated, len 0);
        assert_that!(ended, eq alloc::vec![id]);
    }

    #[test]
    fn an_injected_abandonment_ends_the_request_for_the_local_server() {
        const FIRE_AND_FORGET: bool = false;
        const RESPONDER: RemoteId = 1;
        const REQUEST: RequestPayload = 7;

        let (app, mut sut) = setup(FIRE_AND_FORGET);
        let id = remote_id();
        let server = app.server();
        inject_request(&mut sut.ports, id, RESPONDER, REQUEST);
        let active = server
            .receive()
            .expect("receive succeeds")
            .expect("the injected request");
        assert_that!(active.is_connected(), eq true);

        inject_abandonment(&mut sut.ports, id);

        assert_that!(active.is_connected(), eq false);
    }

    #[test]
    fn an_injected_response_reaches_the_local_client() {
        const FIRE_AND_FORGET: bool = false;
        const RESPONDER: RemoteId = 1;
        const REQUEST: RequestPayload = 42;
        const RESPONSE: ResponsePayload = 9;

        let (app, mut sut) = setup(FIRE_AND_FORGET);
        let client = app.client();
        let pending = client.send_copy(REQUEST).expect("request is sent");
        let id = ClientRequestId::of(pending.header());
        sut.ports.update_remotes(responders(&[RESPONDER]));
        propagate_requests(&mut sut.ports);

        assert_that!(inject_response(&mut sut.ports, id, RESPONSE), eq true);

        let response = pending
            .receive()
            .expect("receive succeeds")
            .expect("the injected response");
        assert_that!(*response.payload(), eq RESPONSE);
    }

    #[test]
    fn a_response_to_a_request_no_longer_held_is_skipped() {
        const FIRE_AND_FORGET: bool = false;
        const RESPONDER: RemoteId = 1;
        const REQUEST: RequestPayload = 42;
        const RESPONSE: ResponsePayload = 9;

        let (app, mut sut) = setup(FIRE_AND_FORGET);
        let client = app.client();
        let pending = client.send_copy(REQUEST).expect("request is sent");
        let id = ClientRequestId::of(pending.header());
        sut.ports.update_remotes(responders(&[RESPONDER]));
        propagate_requests(&mut sut.ports);
        drop(pending);
        propagate_requests(&mut sut.ports);

        assert_that!(inject_response(&mut sut.ports, id, RESPONSE), eq false);
    }

    #[test]
    fn the_stream_ends_once_every_responder_ended() {
        const FIRE_AND_FORGET: bool = false;
        const RESPONDER: RemoteId = 1;
        const OTHER_RESPONDER: RemoteId = 2;
        const REQUEST: RequestPayload = 42;

        let (app, mut sut) = setup(FIRE_AND_FORGET);
        let client = app.client();
        let pending = client.send_copy(REQUEST).expect("request is sent");
        let id = ClientRequestId::of(pending.header());
        sut.ports
            .update_remotes(responders(&[RESPONDER, OTHER_RESPONDER]));
        propagate_requests(&mut sut.ports);

        inject_end(&mut sut.ports, id, RESPONDER);
        assert_that!(pending.is_connected(), eq true);

        inject_end(&mut sut.ports, id, OTHER_RESPONDER);
        assert_that!(pending.is_connected(), eq false);
    }

    #[test]
    fn an_end_from_a_responder_not_awaited_changes_nothing() {
        const FIRE_AND_FORGET: bool = false;
        const RESPONDER: RemoteId = 1;
        const OTHER_RESPONDER: RemoteId = 2;
        const REQUEST: RequestPayload = 42;

        let (app, mut sut) = setup(FIRE_AND_FORGET);
        let client = app.client();
        let pending = client.send_copy(REQUEST).expect("request is sent");
        let id = ClientRequestId::of(pending.header());
        sut.ports.update_remotes(responders(&[RESPONDER]));
        propagate_requests(&mut sut.ports);

        inject_end(&mut sut.ports, id, OTHER_RESPONDER);

        assert_that!(pending.is_connected(), eq true);
    }

    #[test]
    fn a_departed_responder_counts_as_finished() {
        const FIRE_AND_FORGET: bool = false;
        const RESPONDER: RemoteId = 1;
        const OTHER_RESPONDER: RemoteId = 2;
        const REQUEST: RequestPayload = 42;

        let (app, mut sut) = setup(FIRE_AND_FORGET);
        let client = app.client();
        let pending = client.send_copy(REQUEST).expect("request is sent");
        sut.ports
            .update_remotes(responders(&[RESPONDER, OTHER_RESPONDER]));
        propagate_requests(&mut sut.ports);

        sut.ports.update_remotes(responders(&[OTHER_RESPONDER]));
        assert_that!(pending.is_connected(), eq true);

        sut.ports.update_remotes(responders(&[]));
        assert_that!(pending.is_connected(), eq false);
    }

    #[test]
    fn a_request_of_a_departed_origin_is_dropped() {
        const FIRE_AND_FORGET: bool = false;
        const RESPONDER: RemoteId = 1;
        const REQUEST: RequestPayload = 7;

        let (app, mut sut) = setup(FIRE_AND_FORGET);
        let id = remote_id();
        sut.ports.update_remotes(responders(&[RESPONDER]));
        let server = app.server();
        inject_request(&mut sut.ports, id, RESPONDER, REQUEST);
        let active = server
            .receive()
            .expect("receive succeeds")
            .expect("the injected request");

        sut.ports.update_remotes(responders(&[]));

        assert_that!(active.is_connected(), eq false);
    }

    #[test]
    fn a_request_of_an_origin_not_yet_seen_is_kept() {
        const FIRE_AND_FORGET: bool = false;
        const RESPONDER: RemoteId = 1;
        const REQUEST: RequestPayload = 7;

        let (app, mut sut) = setup(FIRE_AND_FORGET);
        let id = remote_id();
        let server = app.server();
        inject_request(&mut sut.ports, id, RESPONDER, REQUEST);
        let active = server
            .receive()
            .expect("receive succeeds")
            .expect("the injected request");

        sut.ports.update_remotes(responders(&[]));

        assert_that!(active.is_connected(), eq true);
    }

    #[test]
    fn close_abandons_held_requests_and_ends_injected_ones() {
        const FIRE_AND_FORGET: bool = false;
        const RESPONDER: RemoteId = 1;
        const REQUEST: RequestPayload = 42;

        let (app, mut sut) = setup(FIRE_AND_FORGET);
        let client = app.client();
        let pending = client.send_copy(REQUEST).expect("request is sent");
        let local = ClientRequestId::of(pending.header());
        sut.ports.update_remotes(responders(&[RESPONDER]));
        propagate_requests(&mut sut.ports);
        let remote = remote_id();
        let server = app.server();
        inject_request(&mut sut.ports, remote, RESPONDER, REQUEST);
        let active = server
            .receive()
            .expect("receive succeeds")
            .expect("the injected request");

        let mut abandoned = Vec::new();
        let mut ended = Vec::new();
        sut.ports.drain_local_requests(|id| {
            abandoned.push(id);
            Ok::<(), CreationError>(())
        });
        sut.ports.drain_remote_requests(|id| {
            ended.push(id);
            Ok::<(), CreationError>(())
        });

        assert_that!(abandoned, eq alloc::vec![local]);
        assert_that!(ended, eq alloc::vec![remote]);
        assert_that!(pending.is_connected(), eq false);
        assert_that!(active.is_connected(), eq false);
    }
}
