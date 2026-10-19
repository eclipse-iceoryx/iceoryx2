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

use core::marker::PhantomData;
use core::time::Duration;
use std::collections::{BTreeMap, VecDeque};
use std::rc::Rc;
use std::time::Instant;

use iceoryx2_bb_concurrency::cell::RefCell;
use iceoryx2_integrations_ros2_link_adapter::testing::{
    GraphHandle, PeerNode, RclClient, RclPublisher, RclService, RclSubscription, RequestId,
    SequenceNumber, send_request, send_response, take_request, take_response, take_serialized,
};
use iceoryx2_integrations_ros2_link_adapter::{
    EndpointDescription, EndpointName, EndpointSettings, MessageType, ServiceName, TypeName,
};
use iceoryx2_link_adapter::EndpointTypes;
use iceoryx2_link_backend::service_description::ServiceDescription;
use iceoryx2_link_conformance_tests::fixture::{
    Describable, Discoverable, PropagatesMessages, PropagatesPayloads, PropagatesRequestResponse,
    PropagatesRequestResponseMessages, ResponseMessage,
};
use iceoryx2_link_conformance_tests::parameters::PayloadShape;

use super::TranslatorUnderTest;
use super::serialization;

/// The peer's own endpoints match each other.
const OWN: usize = 1;

/// The pause between two looks at the matching counts.
pub const POLL_PERIOD: Duration = Duration::from_millis(10);

/// A remote peer's RCL endpoints on a topic that communicates the wire form
/// of the types defined by the translator under test `T`. Required to
/// support tests against remote endpoints implementing [`PropagatesMessages`].
pub struct RclPublishSubscribeEndpoints<T> {
    service_description: ServiceDescription,
    endpoint_description: EndpointDescription,
    publisher: RclPublisher,
    subscription: RclSubscription,
    _translator: PhantomData<T>,
}

impl<T> RclPublishSubscribeEndpoints<T> {
    pub(super) fn new(
        peer: &PeerNode,
        service_description: ServiceDescription,
        endpoint_description: EndpointDescription,
    ) -> Self {
        let EndpointDescription { settings, types } = &endpoint_description;
        let (EndpointName::Topic(topic), EndpointTypes::PublishSubscribe(types)) =
            (&settings.name, types)
        else {
            panic!("the fixture opens publish-subscribe endpoints only");
        };

        let publisher = peer.publisher(topic, &types.type_name, settings.qos.clone());
        let subscription = peer.subscription(topic, &types.type_name, settings.qos.clone());

        Self {
            service_description,
            endpoint_description,
            publisher,
            subscription,
            _translator: PhantomData,
        }
    }
}

impl<T: TranslatorUnderTest> Describable for RclPublishSubscribeEndpoints<T> {
    type Settings = EndpointSettings;
    type Types = EndpointTypes<MessageType>;

    fn description(&self) -> &EndpointDescription {
        &self.endpoint_description
    }

    /// Matched once the peer's publisher sees a subscription and its
    /// subscription sees a publisher beyond itself.
    fn sync(&self, timeout: Duration) -> bool {
        let started = Instant::now();
        loop {
            let publisher_matched = self
                .publisher
                .subscription_count()
                .expect("the subscription count is read")
                > OWN;
            let subscription_matched = self
                .subscription
                .publisher_count()
                .expect("the publisher count is read")
                > OWN;
            if publisher_matched && subscription_matched {
                return true;
            }
            if started.elapsed() >= timeout {
                return false;
            }
            std::thread::sleep(POLL_PERIOD);
        }
    }
}

impl<T: TranslatorUnderTest> PropagatesMessages for RclPublishSubscribeEndpoints<T> {
    type Value = <T::SamplePayload as PayloadShape>::Value;

    fn value(&self, n: u64) -> Self::Value {
        <T::SamplePayload as PayloadShape>::value(n)
    }

    fn encode(&self, value: &Self::Value) -> Vec<u8> {
        serialization::serialize(&T::SampleMessage::from(value.clone()))
    }

    fn decode(&self, bytes: &[u8]) -> Self::Value {
        serialization::deserialize::<T::SampleMessage>(bytes).into()
    }

    fn send(&self, value: Self::Value) {
        self.publisher
            .publish(&self.encode(&value))
            .expect("the peer publishes the message");
    }

    fn receive(&self) -> Option<Self::Value> {
        loop {
            let (message, info) = take_serialized(&self.subscription)?;
            if info.gid != *self.publisher.gid() {
                return Some(self.decode(&message));
            }
        }
    }
}

/// A remote peer's rcl client and server on a ROS 2 service that
/// communicates the wire form of the request and response types defined by
/// the translator under test `T`. Required to support tests against remote
/// endpoints implementing [`PropagatesRequestResponseMessages`].
///
/// A request is identified by its sequence number. rcl reports a request
/// by an entity of the client the client itself cannot name, so the id
/// rcl reports is kept beside the number for the response.
///
/// A ROS 2 server answers a request once, so the end of a response stream
/// is reported right after its response.
pub struct RclRequestResponseEndpoints<T> {
    service_description: ServiceDescription,
    endpoint_description: EndpointDescription,
    service: ServiceName,
    request_type: TypeName,
    response_type: TypeName,
    client: RclClient,
    server: RclService,
    /// The graphs of the peer's node and the gateways' nodes, for matching.
    graphs: Rc<RefCell<Vec<GraphHandle>>>,
    /// The rcl ids of the requests taken, by sequence number.
    taken: RefCell<BTreeMap<SequenceNumber, RequestId>>,
    /// The ends to report for responses received.
    pending_ends: RefCell<VecDeque<SequenceNumber>>,
    _translator: PhantomData<T>,
}

impl<T> RclRequestResponseEndpoints<T> {
    pub(super) fn new(
        peer: &PeerNode,
        graphs: Rc<RefCell<Vec<GraphHandle>>>,
        service_description: ServiceDescription,
        endpoint_description: EndpointDescription,
    ) -> Self {
        let EndpointDescription { settings, types } = &endpoint_description;
        let (EndpointName::Service(service), EndpointTypes::RequestResponse { request, response }) =
            (&settings.name, types)
        else {
            panic!("the fixture opens request-response endpoints only");
        };
        let service_type = request
            .type_name
            .as_str()
            .strip_suffix("_Request")
            .expect("the request type names a service");
        let service_type = TypeName::new(service_type).expect("a valid type name");

        let client = peer.client(service, &service_type, settings.qos.clone());
        let server = peer.service(service, &service_type, settings.qos.clone());

        Self {
            service: service.clone(),
            request_type: request.type_name.clone(),
            response_type: response.type_name.clone(),
            service_description,
            endpoint_description,
            client,
            server,
            graphs,
            taken: RefCell::new(BTreeMap::new()),
            pending_ends: RefCell::new(VecDeque::new()),
            _translator: PhantomData,
        }
    }
}

impl<T: TranslatorUnderTest> Describable for RclRequestResponseEndpoints<T> {
    type Settings = EndpointSettings;
    type Types = EndpointTypes<MessageType>;

    fn description(&self) -> &EndpointDescription {
        &self.endpoint_description
    }

    /// Matched once every node, the peer's and the gateways', counts a
    /// server and a client of the service beyond its own. A request sent
    /// before the sender's node has discovered the receiver is lost.
    fn sync(&self, timeout: Duration) -> bool {
        let started = Instant::now();
        loop {
            let matched = self.graphs.borrow().iter().all(|graph| {
                graph.discovered_server(&self.service) && graph.discovered_client(&self.service)
            });
            if matched {
                return true;
            }
            if started.elapsed() >= timeout {
                return false;
            }
            std::thread::sleep(POLL_PERIOD);
        }
    }
}

impl<T: TranslatorUnderTest> PropagatesRequestResponseMessages for RclRequestResponseEndpoints<T> {
    type Request = T::RequestMessage;
    type Response = T::ResponseMessage;
    type RequestId = SequenceNumber;

    fn request(&self, n: u64) -> T::RequestMessage {
        <T::RequestPayload as PayloadShape>::value(n).into()
    }

    fn response(&self, n: u64) -> T::ResponseMessage {
        <T::ResponsePayload as PayloadShape>::value(n).into()
    }

    fn encode_request(&self, request: &T::RequestMessage) -> Vec<u8> {
        serialization::serialize(request)
    }

    fn decode_request(&self, bytes: &[u8]) -> T::RequestMessage {
        serialization::deserialize(bytes)
    }

    fn encode_response(&self, response: &T::ResponseMessage) -> Vec<u8> {
        serialization::serialize(response)
    }

    fn decode_response(&self, bytes: &[u8]) -> T::ResponseMessage {
        serialization::deserialize(bytes)
    }

    fn send_request(&self, request: T::RequestMessage) -> SequenceNumber {
        send_request(
            &self.client,
            &self.request_type,
            &self.encode_request(&request),
        )
    }

    fn receive_request(&self) -> Option<(SequenceNumber, T::RequestMessage)> {
        let (id, bytes) = take_request(&self.server, &self.request_type)?;
        self.taken.borrow_mut().insert(id.sequence_number, id);
        let request = self.decode_request(&bytes);
        Some((id.sequence_number, request))
    }

    fn send_response(&self, id: SequenceNumber, response: T::ResponseMessage) {
        let taken = self.taken.borrow();
        let id = taken.get(&id).expect("the request was taken");
        send_response(
            &self.server,
            &self.response_type,
            id,
            &self.encode_response(&response),
        );
    }

    /// ROS 2 has no end of stream, the one response ends it.
    fn end_response_stream(&self, id: SequenceNumber) {
        self.taken.borrow_mut().remove(&id);
    }

    fn receive_response(&self) -> Option<(SequenceNumber, ResponseMessage<T::ResponseMessage>)> {
        if let Some(id) = self.pending_ends.borrow_mut().pop_front() {
            return Some((id, ResponseMessage::End));
        }
        let (id, bytes) = take_response(&self.client, &self.response_type)?;
        self.pending_ends.borrow_mut().push_back(id.sequence_number);
        let response = self.decode_response(&bytes);
        Some((id.sequence_number, ResponseMessage::Response(response)))
    }
}

/// The peer's rcl endpoints on the topic or ROS 2 service a service maps to.
pub enum RclEndpoints<T> {
    PublishSubscribe(RclPublishSubscribeEndpoints<T>),
    RequestResponse(RclRequestResponseEndpoints<T>),
}

impl<T> RclEndpoints<T> {
    /// Creates the endpoints `endpoint_description` describes, on the topic
    /// or ROS 2 service `service_description` maps to.
    pub(super) fn new(
        peer: &PeerNode,
        graphs: Rc<RefCell<Vec<GraphHandle>>>,
        service_description: ServiceDescription,
        endpoint_description: EndpointDescription,
    ) -> Self {
        match &endpoint_description.types {
            EndpointTypes::PublishSubscribe(_) => Self::PublishSubscribe(
                RclPublishSubscribeEndpoints::new(peer, service_description, endpoint_description),
            ),
            EndpointTypes::RequestResponse { .. } => {
                Self::RequestResponse(RclRequestResponseEndpoints::new(
                    peer,
                    graphs,
                    service_description,
                    endpoint_description,
                ))
            }
            EndpointTypes::Event => panic!("ROS 2 has no events"),
        }
    }

    fn publish_subscribe(&self) -> &RclPublishSubscribeEndpoints<T> {
        match self {
            Self::PublishSubscribe(endpoints) => endpoints,
            Self::RequestResponse(_) => panic!("the endpoints are on a ROS 2 service, not a topic"),
        }
    }

    fn request_response(&self) -> &RclRequestResponseEndpoints<T> {
        match self {
            Self::RequestResponse(endpoints) => endpoints,
            Self::PublishSubscribe(_) => {
                panic!("the endpoints are on a topic, not a ROS 2 service")
            }
        }
    }
}

impl<T: TranslatorUnderTest> Describable for RclEndpoints<T> {
    type Settings = EndpointSettings;
    type Types = EndpointTypes<MessageType>;

    fn description(&self) -> &EndpointDescription {
        match self {
            Self::PublishSubscribe(endpoints) => endpoints.description(),
            Self::RequestResponse(endpoints) => endpoints.description(),
        }
    }

    fn sync(&self, timeout: Duration) -> bool {
        match self {
            Self::PublishSubscribe(endpoints) => endpoints.sync(timeout),
            Self::RequestResponse(endpoints) => endpoints.sync(timeout),
        }
    }
}

impl<T: TranslatorUnderTest> PropagatesMessages for RclEndpoints<T> {
    type Value = <T::SamplePayload as PayloadShape>::Value;

    fn value(&self, n: u64) -> Self::Value {
        self.publish_subscribe().value(n)
    }

    fn encode(&self, value: &Self::Value) -> Vec<u8> {
        self.publish_subscribe().encode(value)
    }

    fn decode(&self, bytes: &[u8]) -> Self::Value {
        self.publish_subscribe().decode(bytes)
    }

    fn send(&self, value: Self::Value) {
        self.publish_subscribe().send(value);
    }

    fn receive(&self) -> Option<Self::Value> {
        self.publish_subscribe().receive()
    }
}

impl<T: TranslatorUnderTest> PropagatesRequestResponseMessages for RclEndpoints<T> {
    type Request = T::RequestMessage;
    type Response = T::ResponseMessage;
    type RequestId = SequenceNumber;

    fn request(&self, n: u64) -> T::RequestMessage {
        self.request_response().request(n)
    }

    fn response(&self, n: u64) -> T::ResponseMessage {
        self.request_response().response(n)
    }

    fn encode_request(&self, request: &T::RequestMessage) -> Vec<u8> {
        self.request_response().encode_request(request)
    }

    fn decode_request(&self, bytes: &[u8]) -> T::RequestMessage {
        self.request_response().decode_request(bytes)
    }

    fn encode_response(&self, response: &T::ResponseMessage) -> Vec<u8> {
        self.request_response().encode_response(response)
    }

    fn decode_response(&self, bytes: &[u8]) -> T::ResponseMessage {
        self.request_response().decode_response(bytes)
    }

    fn send_request(&self, request: T::RequestMessage) -> SequenceNumber {
        self.request_response().send_request(request)
    }

    fn receive_request(&self) -> Option<(SequenceNumber, T::RequestMessage)> {
        self.request_response().receive_request()
    }

    fn send_response(&self, id: SequenceNumber, response: T::ResponseMessage) {
        self.request_response().send_response(id, response);
    }

    fn end_response_stream(&self, id: SequenceNumber) {
        self.request_response().end_response_stream(id);
    }

    fn receive_response(&self) -> Option<(SequenceNumber, ResponseMessage<T::ResponseMessage>)> {
        self.request_response().receive_response()
    }
}

impl<T: TranslatorUnderTest> Discoverable for RclEndpoints<T> {
    fn service(&self) -> &ServiceDescription {
        match self {
            Self::PublishSubscribe(endpoints) => &endpoints.service_description,
            Self::RequestResponse(endpoints) => &endpoints.service_description,
        }
    }

    fn sync(&self, timeout: Duration) -> bool {
        Describable::sync(self, timeout)
    }
}

impl<T: TranslatorUnderTest> PropagatesPayloads<<T::SamplePayload as PayloadShape>::Value>
    for RclEndpoints<T>
{
    fn send_payload(&self, payload: <T::SamplePayload as PayloadShape>::Value) {
        self.publish_subscribe().send(payload);
    }

    fn receive_payload(&self) -> Option<<T::SamplePayload as PayloadShape>::Value> {
        self.publish_subscribe().receive()
    }
}

impl<T: TranslatorUnderTest>
    PropagatesRequestResponse<
        <T::RequestPayload as PayloadShape>::Value,
        <T::ResponsePayload as PayloadShape>::Value,
    > for RclEndpoints<T>
{
    type RequestId = SequenceNumber;

    fn send_request(&self, request: <T::RequestPayload as PayloadShape>::Value) -> SequenceNumber {
        self.request_response().send_request(request.into())
    }

    fn receive_request(
        &self,
    ) -> Option<(SequenceNumber, <T::RequestPayload as PayloadShape>::Value)> {
        let (id, request) = self.request_response().receive_request()?;
        Some((id, request.into()))
    }

    fn send_response(
        &self,
        id: SequenceNumber,
        response: <T::ResponsePayload as PayloadShape>::Value,
    ) {
        self.request_response().send_response(id, response.into());
    }

    fn end_response_stream(&self, id: SequenceNumber) {
        self.request_response().end_response_stream(id);
    }

    fn receive_response(
        &self,
    ) -> Option<(
        SequenceNumber,
        ResponseMessage<<T::ResponsePayload as PayloadShape>::Value>,
    )> {
        let (id, message) = self.request_response().receive_response()?;
        let message = match message {
            ResponseMessage::Response(response) => ResponseMessage::Response(response.into()),
            ResponseMessage::End => ResponseMessage::End,
        };
        Some((id, message))
    }
}
