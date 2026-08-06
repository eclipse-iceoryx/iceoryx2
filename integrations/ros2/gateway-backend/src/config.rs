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

//! Public, serializable configuration for the ROS 2 gateway backend.

use serde::{Deserialize, Serialize};

use crate::rcl;

pub use crate::NameError;

/// A ROS 2 topic name, e.g. `/Camera/FrontRight`, validated against the ROS 2
/// topic naming rules.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct TopicName(rcl::TopicName);

impl TopicName {
    /// Creates a topic name, validating it against the ROS 2 topic naming rules.
    pub fn new(topic: &str) -> Result<Self, NameError> {
        Ok(Self(rcl::TopicName::new(topic)?))
    }

    /// The topic name as a string slice.
    pub fn as_str(&self) -> &str {
        self.0.as_str()
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

/// A ROS 2 message type name of the form `package/msg/Message`, e.g.
/// `geometry_msgs/msg/Twist`.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct TypeName(rcl::TypeName);

impl TypeName {
    pub fn new(type_name: &str) -> Result<Self, NameError> {
        Ok(Self(rcl::TypeName::new(type_name)?))
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

/// Configuration for the `Ros2Backend`.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    /// Message types whose typesupport is resolved during backend creation.
    /// Types left out are resolved on first use instead.
    ///
    /// Types listed here which cannot be resolved cause an error on creation
    /// and thus fails quickly on misconfiguration.
    #[serde(default)]
    pub preload_types: Vec<TypeName>,
}
