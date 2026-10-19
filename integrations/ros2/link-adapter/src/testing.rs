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

use std::rc::Rc;

use crate::adapter::Ros2Adapter;
use crate::config::{ServiceName, TopicName, TypeName};
use crate::qos::QosProfile;
use crate::rcl::{
    FullyQualifiedNodeName, NodeName, RclClientBuilder, RclNode, RclNodeBuilder,
    RclPublisherBuilder, RclServiceBuilder, RclSubscriptionBuilder,
};
use crate::ros_message::RosMessage;
use crate::typesupport;

pub use crate::rcl::client::RclClient;
pub use crate::rcl::message_info::MessageInfo;
pub use crate::rcl::publisher::RclPublisher;
pub use crate::rcl::request_id::{RequestId, SequenceNumber};
pub use crate::rcl::service::RclService;
pub use crate::rcl::service_info::ServiceInfo;
pub use crate::rcl::subscription::RclSubscription;

/// A handle to query information on the ROS 2 graph.
///
/// Request-response tests wait on it before sending a request, since a
/// request sent before the sender's node has discovered the receiver is
/// lost. rcl counts matched publishers and subscriptions but not matched
/// servers and clients, so publish-subscribe tests need no such handle.
#[derive(Debug, Clone)]
pub struct GraphHandle {
    node: Rc<RclNode>,
}

impl GraphHandle {
    /// Whether the node has discovered a server of `service` on another
    /// node.
    pub fn discovered_server(&self, service: &ServiceName) -> bool {
        let service = service.into();
        self.other_nodes().iter().any(|node| {
            self.node
                .servers_of(node)
                .expect("the node's servers are read")
                .contains(&service)
        })
    }

    /// Whether the node has discovered a client of `service` on another
    /// node.
    pub fn discovered_client(&self, service: &ServiceName) -> bool {
        let service = service.into();
        self.other_nodes().iter().any(|node| {
            self.node
                .clients_of(node)
                .expect("the node's clients are read")
                .contains(&service)
        })
    }

    /// The nodes on the graph other than this one.
    fn other_nodes(&self) -> Vec<FullyQualifiedNodeName> {
        let own = self.node.fully_qualified_name();
        self.node
            .nodes()
            .expect("the nodes are read")
            .into_iter()
            .filter(|node| *node != own)
            .collect()
    }
}

impl Ros2Adapter {
    /// Acquire the handle to query the graph from the adapter's node.
    #[doc(hidden)]
    pub fn graph(&self) -> GraphHandle {
        GraphHandle {
            node: Rc::clone(&self.node),
        }
    }
}

/// A ROS 2 node that the gateway communicates with in tests.
#[derive(Debug, Clone)]
pub struct PeerNode {
    node: Rc<RclNode>,
}

impl Default for PeerNode {
    fn default() -> Self {
        Self::new()
    }
}

impl PeerNode {
    pub fn new() -> Self {
        let name = NodeName::new("iceoryx2_test_peer").expect("a valid node name");
        let node = RclNodeBuilder::new(name)
            .rosout(false)
            .create()
            .expect("the peer node is created");
        Self {
            node: Rc::new(node),
        }
    }

    /// A publisher of `type_name` on `topic`.
    pub fn publisher(
        &self,
        topic: &TopicName,
        type_name: &TypeName,
        qos: QosProfile,
    ) -> RclPublisher {
        let type_support = typesupport::get::<typesupport::MessageKind>(type_name.as_str())
            .expect("the typesupport loads");
        RclPublisherBuilder::new(Rc::clone(&self.node), &topic.into(), type_support)
            .qos(qos)
            .create()
            .expect("the peer publisher is created")
    }

    /// A subscription of `type_name` on `topic`.
    pub fn subscription(
        &self,
        topic: &TopicName,
        type_name: &TypeName,
        qos: QosProfile,
    ) -> RclSubscription {
        let type_support = typesupport::get::<typesupport::MessageKind>(type_name.as_str())
            .expect("the typesupport loads");
        RclSubscriptionBuilder::new(Rc::clone(&self.node), &topic.into(), type_support)
            .qos(qos)
            .create()
            .expect("the peer subscription is created")
    }

    /// A client of the service type `type_name` on `service`.
    pub fn client(
        &self,
        service: &ServiceName,
        type_name: &TypeName,
        qos: QosProfile,
    ) -> RclClient {
        let type_support = typesupport::get::<typesupport::ServiceKind>(type_name.as_str())
            .expect("the typesupport loads");
        RclClientBuilder::new(Rc::clone(&self.node), &service.into(), type_support)
            .qos(qos)
            .create()
            .expect("the peer client is created")
    }

    /// A server of the service type `type_name` on `service`.
    pub fn service(
        &self,
        service: &ServiceName,
        type_name: &TypeName,
        qos: QosProfile,
    ) -> RclService {
        let type_support = typesupport::get::<typesupport::ServiceKind>(type_name.as_str())
            .expect("the typesupport loads");
        RclServiceBuilder::new(Rc::clone(&self.node), &service.into(), type_support)
            .qos(qos)
            .create()
            .expect("the peer server is created")
    }

    /// The graph as the peer's node sees it.
    pub fn graph(&self) -> GraphHandle {
        GraphHandle {
            node: Rc::clone(&self.node),
        }
    }
}

/// The next message on `subscription` in its serialized form with its
/// info, if any.
pub fn take_serialized(subscription: &RclSubscription) -> Option<(Vec<u8>, MessageInfo)> {
    let mut buffer = Vec::new();
    let taken = subscription
        .take_into(|size| {
            buffer.resize(size, 0);
            Some(buffer.as_mut_ptr())
        })
        .expect("taking from the peer subscription succeeds");
    taken.map(|(size, info)| {
        buffer.truncate(size);
        (buffer, info)
    })
}

/// Sends the serialized request `request` of type `request_type` through
/// `client`. Returns the sequence number the response carries.
pub fn send_request(client: &RclClient, request_type: &TypeName, request: &[u8]) -> SequenceNumber {
    let mut message = RosMessage::new(request_type.as_str()).expect("the request type loads");
    message
        .deserialize_from(request)
        .expect("the request deserializes");
    client
        .send_request(&message)
        .expect("the peer client sends the request")
}

/// The next request on `service` in its serialized form with its id, if
/// any.
///
/// rcl delivers a request to every server, including one on the sender's
/// node, so requests from the service's own participant are skipped.
pub fn take_request(service: &RclService, request_type: &TypeName) -> Option<(RequestId, Vec<u8>)> {
    loop {
        let mut message = RosMessage::new(request_type.as_str()).expect("the request type loads");
        let info = service
            .take_request(&mut message)
            .expect("taking from the peer server succeeds")?;
        if info.request_id.participant_prefix() == *service.participant_prefix() {
            continue;
        }
        let bytes = to_cdr(&message);
        return Some((info.request_id, bytes));
    }
}

/// Sends the serialized response `response` of type `response_type` to the
/// request `id` through `service`.
pub fn send_response(
    service: &RclService,
    response_type: &TypeName,
    id: &RequestId,
    response: &[u8],
) {
    let mut message = RosMessage::new(response_type.as_str()).expect("the response type loads");
    message
        .deserialize_from(response)
        .expect("the response deserializes");
    service
        .send_response(id, &message)
        .expect("the peer server sends the response");
}

/// The next response on `client` in its serialized form with its request
/// id, if any.
pub fn take_response(client: &RclClient, response_type: &TypeName) -> Option<(RequestId, Vec<u8>)> {
    let mut message = RosMessage::new(response_type.as_str()).expect("the response type loads");
    let info = client
        .take_response(&mut message)
        .expect("taking from the peer client succeeds")?;
    let bytes = to_cdr(&message);
    Some((info.request_id, bytes))
}

/// The CDR bytes of `message`.
fn to_cdr(message: &RosMessage) -> Vec<u8> {
    let mut buffer = Vec::new();
    let size = message
        .serialize_into(|size| {
            buffer.resize(size, 0);
            Some(buffer.as_mut_ptr())
        })
        .expect("the message serializes");
    buffer.truncate(size);
    buffer
}
