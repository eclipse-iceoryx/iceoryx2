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
    Discoverable, PropagatesPayloads, PropagatesPayloadsWithHeaders, PropagatesRequestResponse,
    PropagatesRequestResponseWithHeaders, PublishSubscribeMessageEndpoints,
    RequestResponseMessageEndpoints, ResponseMessage,
};
use iceoryx2_link_testing::{
    FakeEndpointDescription, FakeEndpointSettings, FakeMiddleware, FakePublishSubscribeEndpoints,
    FakeRequestId, FakeRequestResponseEndpoints, FakeResponseMessage, FakeSampleTypes, endpoint_of,
};

use super::wire_form::{Header, Payload, WireForm};

/// The description of the endpoint `service` maps to.
pub(crate) fn endpoint_description_for<T: WireForm>(
    service: &ServiceDescription,
) -> FakeEndpointDescription {
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

/// Endpoints on the fake middleware at the endpoint a service maps to.
/// Their messages are in the wire form the translator `T` expects.
pub(crate) struct FakeRemoteEndpoints<T> {
    service_description: ServiceDescription,
    endpoint_description: FakeEndpointDescription,
    endpoints: PatternEndpoints,
    _translator: PhantomData<T>,
}

/// The fake endpoints of one pattern and the size of the user header
/// leading each message they carry. The header is sent as zero bytes.
enum PatternEndpoints {
    PublishSubscribe {
        endpoints: FakePublishSubscribeEndpoints,
        header_size: usize,
    },
    RequestResponse {
        endpoints: FakeRequestResponseEndpoints,
        request_header_size: usize,
        response_header_size: usize,
    },
}

impl PatternEndpoints {
    fn publish_subscribe(&self) -> (&FakePublishSubscribeEndpoints, usize) {
        match self {
            Self::PublishSubscribe {
                endpoints,
                header_size,
            } => (endpoints, *header_size),
            Self::RequestResponse { .. } => {
                panic!("the remote endpoints are not publish-subscribe")
            }
        }
    }

    fn request_response(&self) -> (&FakeRequestResponseEndpoints, usize, usize) {
        match self {
            Self::RequestResponse {
                endpoints,
                request_header_size,
                response_header_size,
            } => (endpoints, *request_header_size, *response_header_size),
            Self::PublishSubscribe { .. } => {
                panic!("the remote endpoints are not request-response")
            }
        }
    }
}

impl<T: WireForm> FakeRemoteEndpoints<T> {
    /// Creates remote endpoints at the endpoint `service_description` maps to.
    pub(crate) fn on(middleware: &FakeMiddleware, service_description: ServiceDescription) -> Self {
        let endpoint_description = endpoint_description_for::<T>(&service_description);
        let endpoints = match &endpoint_description.types {
            EndpointTypes::PublishSubscribe(types) => PatternEndpoints::PublishSubscribe {
                endpoints: middleware.remote_publish_subscribe_endpoints(&endpoint_description),
                header_size: types.user_header.size,
            },
            EndpointTypes::Event => PatternEndpoints::PublishSubscribe {
                endpoints: middleware.remote_publish_subscribe_endpoints(&endpoint_description),
                header_size: 0,
            },
            EndpointTypes::RequestResponse { request, response } => {
                PatternEndpoints::RequestResponse {
                    endpoints: middleware.remote_request_response_endpoints(&endpoint_description),
                    request_header_size: request.user_header.size,
                    response_header_size: response.user_header.size,
                }
            }
        };
        Self {
            service_description,
            endpoint_description,
            endpoints,
            _translator: PhantomData,
        }
    }
}

/// A message of `header_size` zero bytes and the encoded `payload`.
fn message_with_empty_header(header_size: usize, payload: &[u8]) -> Vec<u8> {
    let mut message = alloc::vec![0; header_size];
    message.extend_from_slice(payload);
    message
}

/// The payload of a message led by `header_size` bytes.
fn decoded_payload<W: WireForm>(message: &[u8], header_size: usize) -> Payload {
    let (_, payload) = message.split_at(header_size);
    W::decode_payload(payload.try_into().expect("a payload"))
}

/// The header and the payload of a message led by `header_size` bytes.
fn decoded_sample<W: WireForm>(message: &[u8], header_size: usize) -> (Header, Payload) {
    let (header, payload) = message.split_at(header_size);
    (
        W::decode_header(header.try_into().expect("a header")),
        W::decode_payload(payload.try_into().expect("a payload")),
    )
}

/// The value of a message is its bytes. The fake middleware imposes no
/// form of its own on them.
impl<T: WireForm>
    PublishSubscribeMessageEndpoints<FakeEndpointSettings, EndpointTypes<FakeSampleTypes>>
    for FakeRemoteEndpoints<T>
{
    type Value = Vec<u8>;

    fn description(&self) -> &FakeEndpointDescription {
        &self.endpoint_description
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
        let (endpoints, _) = self.endpoints.publish_subscribe();
        endpoints.send(&value);
    }

    fn receive(&self) -> Option<Vec<u8>> {
        let (endpoints, _) = self.endpoints.publish_subscribe();
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
        &self.endpoint_description
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
        let (endpoints, _, _) = self.endpoints.request_response();
        endpoints.request(&request)
    }

    fn receive_request(&self) -> Option<(FakeRequestId, Vec<u8>)> {
        let (endpoints, _, _) = self.endpoints.request_response();
        endpoints.receive_request()
    }

    fn send_response(&self, id: FakeRequestId, response: Vec<u8>) {
        let (endpoints, _, _) = self.endpoints.request_response();
        endpoints.respond(id, &response);
    }

    fn end_response_stream(&self, id: FakeRequestId) {
        let (endpoints, _, _) = self.endpoints.request_response();
        endpoints.end_response_stream(id);
    }

    fn receive_response(&self) -> Option<(FakeRequestId, ResponseMessage<Vec<u8>>)> {
        let (endpoints, _, _) = self.endpoints.request_response();
        let (id, message) = endpoints.receive_response()?;
        let message = match message {
            FakeResponseMessage::Response(bytes) => ResponseMessage::Response(bytes),
            FakeResponseMessage::End => ResponseMessage::End,
        };
        Some((id, message))
    }
}

impl<T: WireForm> Discoverable for FakeRemoteEndpoints<T> {
    fn service(&self) -> &ServiceDescription {
        &self.service_description
    }
}

impl<T: WireForm> PropagatesPayloads<Payload> for FakeRemoteEndpoints<T> {
    fn send_payload(&self, payload: Payload) {
        let (endpoints, header_size) = self.endpoints.publish_subscribe();
        endpoints.send(&message_with_empty_header(
            header_size,
            &T::encode_payload(payload),
        ));
    }

    fn receive_payload(&self) -> Option<Payload> {
        let (endpoints, header_size) = self.endpoints.publish_subscribe();
        let payload = decoded_payload::<T>(&endpoints.receive()?, header_size);
        Some(payload)
    }
}

impl<T: WireForm> PropagatesPayloadsWithHeaders<Header, Payload> for FakeRemoteEndpoints<T> {
    fn send_with_header(&self, header: Header, payload: Payload) {
        let (endpoints, _) = self.endpoints.publish_subscribe();
        endpoints.send(&[T::encode_header(header), T::encode_payload(payload)].concat());
    }

    fn receive_with_header(&self) -> Option<(Header, Payload)> {
        let (endpoints, header_size) = self.endpoints.publish_subscribe();
        let (header, payload) = decoded_sample::<T>(&endpoints.receive()?, header_size);
        Some((header, payload))
    }
}

impl<T: WireForm> PropagatesRequestResponse<Payload, Payload> for FakeRemoteEndpoints<T> {
    type RequestId = FakeRequestId;

    fn send_request(&self, request: Payload) -> FakeRequestId {
        let (endpoints, request_header_size, _) = self.endpoints.request_response();
        endpoints.request(&message_with_empty_header(
            request_header_size,
            &T::encode_payload(request),
        ))
    }

    fn receive_request(&self) -> Option<(FakeRequestId, Payload)> {
        let (endpoints, request_header_size, _) = self.endpoints.request_response();
        let (id, message) = endpoints.receive_request()?;
        let request = decoded_payload::<T>(&message, request_header_size);
        Some((id, request))
    }

    fn send_response(&self, id: FakeRequestId, response: Payload) {
        let (endpoints, _, response_header_size) = self.endpoints.request_response();
        endpoints.respond(
            id,
            &message_with_empty_header(response_header_size, &T::encode_payload(response)),
        );
    }

    fn end_response_stream(&self, id: FakeRequestId) {
        let (endpoints, _, _) = self.endpoints.request_response();
        endpoints.end_response_stream(id);
    }

    fn receive_response(&self) -> Option<(FakeRequestId, ResponseMessage<Payload>)> {
        let (endpoints, _, response_header_size) = self.endpoints.request_response();
        let (id, message) = endpoints.receive_response()?;
        let response = match message {
            FakeResponseMessage::Response(bytes) => {
                let response = decoded_payload::<T>(&bytes, response_header_size);
                ResponseMessage::Response(response)
            }
            FakeResponseMessage::End => ResponseMessage::End,
        };
        Some((id, response))
    }
}

impl<T: WireForm> PropagatesRequestResponseWithHeaders<Header, Payload, Header, Payload>
    for FakeRemoteEndpoints<T>
{
    fn send_request_with_header(&self, header: Header, request: Payload) -> FakeRequestId {
        let (endpoints, _, _) = self.endpoints.request_response();
        endpoints.request(&[T::encode_header(header), T::encode_payload(request)].concat())
    }

    fn receive_request_with_header(&self) -> Option<(FakeRequestId, Header, Payload)> {
        let (endpoints, request_header_size, _) = self.endpoints.request_response();
        let (id, message) = endpoints.receive_request()?;
        let (header, request) = decoded_sample::<T>(&message, request_header_size);
        Some((id, header, request))
    }

    fn send_response_with_header(&self, id: FakeRequestId, header: Header, response: Payload) {
        let (endpoints, _, _) = self.endpoints.request_response();
        endpoints.respond(
            id,
            &[T::encode_header(header), T::encode_payload(response)].concat(),
        );
    }

    fn receive_response_with_header(
        &self,
    ) -> Option<(FakeRequestId, ResponseMessage<(Header, Payload)>)> {
        let (endpoints, _, response_header_size) = self.endpoints.request_response();
        let (id, message) = endpoints.receive_response()?;
        let response = match message {
            FakeResponseMessage::Response(bytes) => {
                let (header, response) = decoded_sample::<T>(&bytes, response_header_size);
                ResponseMessage::Response((header, response))
            }
            FakeResponseMessage::End => ResponseMessage::End,
        };
        Some((id, response))
    }
}
