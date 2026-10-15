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

use alloc::vec::Vec;
use core::marker::PhantomData;

use iceoryx2_link_adapter::{EndpointDescription, EndpointTypes};
use iceoryx2_link_backend::service_description::{ServiceDescription, ServiceTypes};
use iceoryx2_link_conformance_tests::fixture::{
    Discoverable, PropagatesPayloads, PropagatesPayloadsWithHeaders,
    PublishSubscribeMessageEndpoints, RequestResponseMessageEndpoints, ResponseMessage,
};
use iceoryx2_link_testing::{
    FakeEndpointDescription, FakeEndpointSettings, FakeMiddleware, FakePublishSubscribeEndpoints,
    FakeRequestId, FakeRequestResponseEndpoints, FakeResponseMessage, FakeSampleTypes, endpoint_of,
};

use super::wire_form::{Header, Payload, WireForm};

/// The endpoint `service` maps to.
pub(crate) fn endpoint_for<T: WireForm>(service: &ServiceDescription) -> FakeEndpointDescription {
    let types = match service.types() {
        ServiceTypes::PublishSubscribe(types) => EndpointTypes::PublishSubscribe(
            T::default()
                .remote(types)
                .expect("the fake translators never fail"),
        ),
        ServiceTypes::RequestResponse(types) => {
            let translator = T::default();
            EndpointTypes::RequestResponse {
                request: translator
                    .remote(&types.request)
                    .expect("the fake translators never fail"),
                response: translator
                    .remote(&types.response)
                    .expect("the fake translators never fail"),
            }
        }
        ServiceTypes::Event => EndpointTypes::Event,
    };
    EndpointDescription {
        types,
        ..endpoint_of(service)
    }
}

/// Remote endpoints on the fake middleware's endpoint of a service. Their
/// messages are in the wire form the translator `T` expects.
pub(crate) struct FakeRemoteEndpoints<T> {
    service: ServiceDescription,
    description: FakeEndpointDescription,
    remote: Remote,
    _translator: PhantomData<T>,
}

/// The fake endpoints of one pattern and the size of the user header
/// leading each message they carry. The header is sent as zero bytes.
enum Remote {
    PublishSubscribe {
        endpoints: FakePublishSubscribeEndpoints,
        header: usize,
    },
    RequestResponse {
        endpoints: FakeRequestResponseEndpoints,
    },
}

impl Remote {
    fn publish_subscribe(&self) -> (&FakePublishSubscribeEndpoints, usize) {
        match self {
            Self::PublishSubscribe { endpoints, header } => (endpoints, *header),
            Self::RequestResponse { .. } => {
                panic!("the remote endpoints are not publish-subscribe")
            }
        }
    }

    fn request_response(&self) -> &FakeRequestResponseEndpoints {
        match self {
            Self::RequestResponse { endpoints } => endpoints,
            Self::PublishSubscribe { .. } => {
                panic!("the remote endpoints are not request-response")
            }
        }
    }
}

impl<T: WireForm> FakeRemoteEndpoints<T> {
    /// Creates remote endpoints on `middleware`'s endpoint of `service`.
    pub(crate) fn on(middleware: &FakeMiddleware, service: ServiceDescription) -> Self {
        let description = endpoint_for::<T>(&service);
        let remote = match service.types() {
            ServiceTypes::PublishSubscribe(types) => Remote::PublishSubscribe {
                endpoints: middleware.remote_publish_subscribe_endpoints(&description),
                header: types.user_header.size,
            },
            ServiceTypes::Event => Remote::PublishSubscribe {
                endpoints: middleware.remote_publish_subscribe_endpoints(&description),
                header: 0,
            },
            ServiceTypes::RequestResponse(_) => Remote::RequestResponse {
                endpoints: middleware.remote_request_response_endpoints(&description),
            },
        };
        Self {
            service,
            description,
            remote,
            _translator: PhantomData,
        }
    }
}

/// The value of a message is its bytes. The fake middleware imposes no
/// form of its own on them.
impl<T: WireForm>
    PublishSubscribeMessageEndpoints<FakeEndpointSettings, EndpointTypes<FakeSampleTypes>>
    for FakeRemoteEndpoints<T>
{
    type Value = Vec<u8>;

    fn description(&self) -> &FakeEndpointDescription {
        &self.description
    }

    fn value(&self, n: u64) -> Vec<u8> {
        n.to_le_bytes().to_vec()
    }

    fn encode(&self, value: &Vec<u8>) -> Vec<u8> {
        value.clone()
    }

    fn decode(&self, bytes: &[u8]) -> Vec<u8> {
        bytes.to_vec()
    }

    fn send(&self, value: Vec<u8>) {
        let (endpoints, _) = self.remote.publish_subscribe();
        endpoints.send(&value);
    }

    fn receive(&self) -> Option<Vec<u8>> {
        let (endpoints, _) = self.remote.publish_subscribe();
        endpoints.receive()
    }
}

/// The value of a message is its bytes. The fake middleware imposes no
/// form of its own on them.
impl<T: WireForm>
    RequestResponseMessageEndpoints<FakeEndpointSettings, EndpointTypes<FakeSampleTypes>>
    for FakeRemoteEndpoints<T>
{
    type Request = Vec<u8>;
    type Response = Vec<u8>;
    type RequestId = FakeRequestId;

    fn description(&self) -> &FakeEndpointDescription {
        &self.description
    }

    fn request(&self, n: u64) -> Vec<u8> {
        n.to_le_bytes().to_vec()
    }

    fn response(&self, n: u64) -> Vec<u8> {
        n.to_le_bytes().to_vec()
    }

    fn encode_request(&self, request: &Vec<u8>) -> Vec<u8> {
        request.clone()
    }

    fn decode_request(&self, bytes: &[u8]) -> Vec<u8> {
        bytes.to_vec()
    }

    fn encode_response(&self, response: &Vec<u8>) -> Vec<u8> {
        response.clone()
    }

    fn decode_response(&self, bytes: &[u8]) -> Vec<u8> {
        bytes.to_vec()
    }

    fn send_request(&self, request: Vec<u8>) -> FakeRequestId {
        self.remote.request_response().request(&request)
    }

    fn receive_request(&self) -> Option<(FakeRequestId, Vec<u8>)> {
        self.remote.request_response().received_request()
    }

    fn send_response(&self, id: FakeRequestId, response: Vec<u8>) {
        self.remote.request_response().respond(id, &response);
    }

    fn end_response_stream(&self, id: FakeRequestId) {
        self.remote.request_response().end_response_stream(id);
    }

    fn receive_response(&self) -> Option<(FakeRequestId, ResponseMessage<Vec<u8>>)> {
        let (id, message) = self.remote.request_response().received_response()?;
        let message = match message {
            FakeResponseMessage::Response(bytes) => ResponseMessage::Response(bytes),
            FakeResponseMessage::End => ResponseMessage::End,
        };
        Some((id, message))
    }
}

impl<T: WireForm> Discoverable for FakeRemoteEndpoints<T> {
    fn service(&self) -> &ServiceDescription {
        &self.service
    }
}

impl<T: WireForm> PropagatesPayloads<Payload> for FakeRemoteEndpoints<T> {
    fn send_payload(&self, payload: Payload) {
        let (endpoints, header) = self.remote.publish_subscribe();
        let mut message = alloc::vec![0; header];
        message.extend_from_slice(&T::encode_payload(payload));
        endpoints.send(&message);
    }

    fn receive_payload(&self) -> Option<Payload> {
        let (endpoints, header) = self.remote.publish_subscribe();
        let message = endpoints.receive()?;
        let (_, payload) = message.split_at(header);
        Some(T::decode_payload(payload.try_into().expect("a payload")))
    }
}

impl<T: WireForm> PropagatesPayloadsWithHeaders<Header, Payload> for FakeRemoteEndpoints<T> {
    fn send_with_header(&self, header: Header, payload: Payload) {
        let (endpoints, _) = self.remote.publish_subscribe();
        endpoints.send(&[T::encode_header(header), T::encode_payload(payload)].concat());
    }

    fn receive_with_header(&self) -> Option<(Header, Payload)> {
        let (endpoints, header_size) = self.remote.publish_subscribe();
        let message = endpoints.receive()?;
        let (header, payload) = message.split_at(header_size);
        Some((
            T::decode_header(header.try_into().expect("a header")),
            T::decode_payload(payload.try_into().expect("a payload")),
        ))
    }
}
