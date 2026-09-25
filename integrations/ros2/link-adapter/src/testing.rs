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

use crate::config::{TopicName, TypeName};
use crate::qos::QosProfile;
use crate::rcl::{NodeName, RclNode, RclNodeBuilder, RclPublisherBuilder, RclSubscriptionBuilder};
use crate::typesupport;

pub use crate::rcl::publisher::RclPublisher;
pub use crate::rcl::subscription::{MessageInfo, RclSubscription};

/// A ROS 2 node beside the gateway's, holding the endpoints a test
/// speaks through.
#[derive(Debug)]
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
        let type_support = typesupport::load(type_name.as_str()).expect("the typesupport loads");
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
        let type_support = typesupport::load(type_name.as_str()).expect("the typesupport loads");
        RclSubscriptionBuilder::new(Rc::clone(&self.node), &topic.into(), type_support)
            .qos(qos)
            .create()
            .expect("the peer subscription is created")
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
