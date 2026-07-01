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

//! Public, serializable configuration for the ROS 2 tunnel backend.
//!
//! The name types here validate against the ROS 2 naming rules on construction
//! and on deserialization, and store their value as a plain string. This keeps
//! the FFI-facing `rcl` name types out of the public API while sharing the same
//! validation grammar; converting into an `rcl` name is therefore infallible.

use serde::{Deserialize, Serialize};

use iceoryx2_log::fail;

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

/// A ROS 2 message type name of the form `package/msg/Message`, e.g.
/// `geometry_msgs/msg/Twist`.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct TypeName(rcl::TypeName);

impl TypeName {
    /// Creates a type name, validating it against the ROS 2 type naming rules.
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

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum TopicConfigError {
    /// The topic is not a valid ROS 2 topic name.
    InvalidTopic,
    /// The type name is not of the form `package/msg/Message`.
    InvalidTypeName,
}

impl core::fmt::Display for TopicConfigError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "TopicConfigError::{self:?}")
    }
}

impl core::error::Error for TopicConfigError {}

/// A ROS 2 topic to bridge.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct TopicConfig {
    /// Fully-qualified ROS 2 topic name, e.g. `/Camera/FrontRight`.
    pub topic: TopicName,
    /// ROS 2 type name, e.g. `geometry_msgs/msg/Twist`.
    pub type_name: TypeName,
}

impl TopicConfig {
    /// Creates a config entry, validating the ROS 2 topic name and message
    /// type name.
    ///
    /// - The topic must be a valid ROS 2 topic name
    /// - The type name must have the form `package/msg/Message`
    pub fn new(topic: &str, type_name: &str) -> Result<Self, TopicConfigError> {
        let origin = "TopicConfig::new";

        let topic = match TopicName::new(topic) {
            Ok(topic) => topic,
            Err(error) => {
                fail!(from origin,
                    with TopicConfigError::InvalidTopic,
                    "Failed to create topic config from invalid topic name '{}': {}",
                    topic,
                    error
                );
            }
        };
        let type_name = match TypeName::new(type_name) {
            Ok(type_name) => type_name,
            Err(error) => {
                fail!(from origin,
                    with TopicConfigError::InvalidTypeName,
                    "Failed to create topic config from invalid type name '{}': {}",
                    type_name,
                    error
                );
            }
        };

        Ok(Self { topic, type_name })
    }
}

/// Configuration for the `Ros2Backend`.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    /// The topics to bridge. Typesupport for every entry is resolved during
    /// backend creation, which fails if any cannot be resolved.
    pub topics: Vec<TopicConfig>,
}
