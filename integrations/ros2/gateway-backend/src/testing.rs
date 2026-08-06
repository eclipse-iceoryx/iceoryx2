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

//! Test support: a minimal ROS 2 peer observing the gateway from the
//! network side.

use std::rc::Rc;

use crate::qos::QosProfile;
use crate::rcl::{
    NodeName, RclNode, RclNodeBuilder, RclPublisherBuilder, RclSubscriptionBuilder, TopicName,
};
use crate::typesupport;

pub use crate::rcl::publisher::RclPublisher;
pub use crate::rcl::subscription::RclSubscription;

/// A separate ROS 2 peer for observing and validating changes to
/// ROS 2 graph.
#[derive(Debug)]
pub struct TestPeer {
    node: Rc<RclNode>,
}

impl TestPeer {
    pub fn create() -> Self {
        let name = NodeName::new("iceoryx2_test_peer").expect("valid test peer node name");
        let node = RclNodeBuilder::new(name)
            .create()
            .expect("failed to create the test peer node");
        Self {
            node: Rc::new(node),
        }
    }

    /// Creates a publisher on `topic` with default QoS.
    pub fn create_publisher(&self, topic: &str, type_name: &str) -> RclPublisher {
        let topic = TopicName::new(topic).expect("valid topic name");
        let type_support = typesupport::load(type_name).expect("failed to load typesupport");
        RclPublisherBuilder::new(Rc::clone(&self.node), &topic, type_support)
            .create()
            .expect("failed to create the test peer publisher")
    }

    /// Creates a subscription on `topic` with default QoS.
    pub fn create_subscription(&self, topic: &str, type_name: &str) -> RclSubscription {
        let topic = TopicName::new(topic).expect("valid topic name");
        let type_support = typesupport::load(type_name).expect("failed to load typesupport");
        RclSubscriptionBuilder::new(Rc::clone(&self.node), &topic, type_support)
            .create()
            .expect("failed to create the test peer subscription")
    }

    /// The QoS profiles of the publishers on `topic`.
    pub fn publisher_qos(&self, topic: &str) -> Vec<QosProfile> {
        let topic = TopicName::new(topic).expect("valid topic name");
        self.node
            .publisher_qos_profiles(&topic)
            .expect("failed to query publisher QoS")
    }

    /// The QoS profiles of the subscriptions on `topic`.
    pub fn subscription_qos(&self, topic: &str) -> Vec<QosProfile> {
        let topic = TopicName::new(topic).expect("valid topic name");
        self.node
            .subscription_qos_profiles(&topic)
            .expect("failed to query subscription QoS")
    }

    /// The type names the graph currently reports for `topic`.
    pub fn topic_types(&self, topic: &str) -> Vec<String> {
        self.node
            .topic_names_and_types()
            .expect("failed to query the ROS 2 graph")
            .into_iter()
            .filter(|(name, _)| name.as_str() == topic)
            .flat_map(|(_, types)| types.into_iter().map(|name| name.as_str().to_string()))
            .collect()
    }
}

/// Backend hook exposing the shared gateway test helpers (`retry`, `sync`) for
/// use in conformance tests which have no information about the gateway implementation.
pub struct Testing;

impl iceoryx2_gateway_backend::traits::testing::Testing for Testing {}

/// Takes the next message on `subscription` as its serialized bytes, or
/// [`None`] when the queue is empty.
pub fn take_serialized(subscription: &RclSubscription) -> Option<Vec<u8>> {
    let mut buffer = Vec::new();
    let taken = subscription
        .take_into(|size| {
            buffer.resize(size, 0);
            Some(buffer.as_mut_ptr())
        })
        .expect("failed to take from the test peer subscription");

    taken.map(|(size, _message_info)| {
        buffer.truncate(size);
        buffer
    })
}
