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

use iceoryx2_log::{fail, origin};
use serde::{Deserialize, Serialize};

use crate::rcl;

pub use crate::NameError;

/// A ROS 2 topic name, e.g. `/Camera/FrontRight`, validated against the ROS 2
/// topic naming rules.
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct TopicName(rcl::TopicName);

impl TopicName {
    /// Creates a topic name, validating it against the ROS 2 topic naming rules.
    pub fn new(topic: &str) -> Result<Self, NameError> {
        let origin = origin!("TopicName::new");

        let topic = fail!(
            from origin,
            when rcl::TopicName::new(topic),
            "Invalid ROS 2 topic name '{}'", topic
        );

        Ok(Self(topic))
    }

    /// The topic name as a string slice.
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl core::fmt::Display for TopicName {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl TryFrom<String> for TopicName {
    type Error = NameError;

    fn try_from(topic: String) -> Result<Self, NameError> {
        Self::new(&topic)
    }
}

impl From<TopicName> for String {
    fn from(topic: TopicName) -> Self {
        topic.as_str().to_string()
    }
}

impl From<&TopicName> for rcl::TopicName {
    fn from(topic: &TopicName) -> Self {
        topic.0.clone()
    }
}

impl From<rcl::TopicName> for TopicName {
    fn from(topic: rcl::TopicName) -> Self {
        Self(topic)
    }
}

/// A ROS 2 service name, e.g. `/add_two_ints`, validated against the ROS 2
/// service naming rules.
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct ServiceName(rcl::ServiceName);

impl ServiceName {
    /// Creates a service name, validating it against the ROS 2 service
    /// naming rules.
    pub fn new(service: &str) -> Result<Self, NameError> {
        let origin = origin!("ServiceName::new");

        let service = fail!(
            from origin,
            when rcl::ServiceName::new(service),
            "Invalid ROS 2 service name '{}'", service
        );

        Ok(Self(service))
    }

    /// The service name as a string slice.
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl core::fmt::Display for ServiceName {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl TryFrom<String> for ServiceName {
    type Error = NameError;

    fn try_from(service: String) -> Result<Self, NameError> {
        Self::new(&service)
    }
}

impl From<ServiceName> for String {
    fn from(service: ServiceName) -> Self {
        service.as_str().to_string()
    }
}

impl From<&ServiceName> for rcl::ServiceName {
    fn from(service: &ServiceName) -> Self {
        service.0.clone()
    }
}

impl From<rcl::ServiceName> for ServiceName {
    fn from(service: rcl::ServiceName) -> Self {
        Self(service)
    }
}

/// A ROS 2 type name of the form `package/msg/Message`, `package/srv/Service`
/// or `package/action/Action`, e.g. `geometry_msgs/msg/Twist`.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct TypeName(rcl::TypeName);

impl TypeName {
    pub fn new(type_name: &str) -> Result<Self, NameError> {
        let origin = origin!("TypeName::new");

        let type_name = fail!(
            from origin,
            when rcl::TypeName::new(type_name),
            "Invalid ROS 2 type name '{}'", type_name
        );
        Ok(Self(type_name))
    }

    /// The type name as a string slice.
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl TryFrom<String> for TypeName {
    type Error = NameError;

    fn try_from(type_name: String) -> Result<Self, NameError> {
        Self::new(&type_name)
    }
}

impl From<TypeName> for String {
    fn from(type_name: TypeName) -> Self {
        type_name.as_str().to_string()
    }
}

impl From<&TypeName> for rcl::TypeName {
    fn from(type_name: &TypeName) -> Self {
        type_name.0.clone()
    }
}

impl From<rcl::TypeName> for TypeName {
    /// Adopts an rcl type name, which already upholds the naming rules.
    fn from(type_name: rcl::TypeName) -> Self {
        Self(type_name)
    }
}

/// A ROS 2 node name, e.g. `iceoryx2_gateway`, validated against the ROS 2
/// node naming rules.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct NodeName(rcl::NodeName);

impl NodeName {
    /// Creates a node name, validating it against the ROS 2 node naming
    /// rules.
    pub fn new(node: &str) -> Result<Self, NameError> {
        let origin = origin!("NodeName::new");

        let node = fail!(
            from origin,
            when rcl::NodeName::new(node),
            "Invalid ROS 2 node name '{}'", node
        );

        Ok(Self(node))
    }

    /// The node name as a string slice.
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl core::fmt::Display for NodeName {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl TryFrom<String> for NodeName {
    type Error = NameError;

    fn try_from(node: String) -> Result<Self, NameError> {
        Self::new(&node)
    }
}

impl From<NodeName> for String {
    fn from(node: NodeName) -> Self {
        node.as_str().to_string()
    }
}

impl From<&NodeName> for rcl::NodeName {
    fn from(node: &NodeName) -> Self {
        node.0.clone()
    }
}

/// Configuration for the `Ros2Adapter`.
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    /// The name of the gateway's node, `iceoryx2_gateway` unless set.
    #[serde(default = "node_name_default")]
    pub node_name: NodeName,
    /// Message and service types whose typesupport is resolved during
    /// backend creation. Types left out are resolved on first use instead.
    ///
    /// Types listed here which cannot be resolved cause an error on creation
    /// and thus fails quickly on misconfiguration.
    #[serde(default)]
    pub preload_types: Vec<TypeName>,
    /// Whether the gateway's node publishes its log output on `/rosout`.
    #[serde(default = "rosout_default")]
    pub rosout: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            node_name: node_name_default(),
            preload_types: Vec::new(),
            rosout: rosout_default(),
        }
    }
}

fn node_name_default() -> NodeName {
    NodeName(crate::NODE_NAME)
}

fn rosout_default() -> bool {
    true
}
