// Copyright (c) 2024 Contributors to the Eclipse Foundation
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

use iceoryx2_bb_container::semantic_string::SemanticStringError;
use serde::{de::Visitor, Deserialize, Serialize};

/// Represent the name for a [`crate::node::Node`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeName {
    value: String,
}

impl NodeName {
    /// Creates a new [`NodeName`].
    /// If the provided name does not contain a valid [`NodeName`] it will return a
    /// [`SemanticStringError`] otherwise the [`NodeName`].
    pub fn new(name: &str) -> Result<Self, SemanticStringError> {
        Ok(Self { value: name.into() })
    }

    /// Returns a str reference to the [`NodeName`]
    pub fn as_str(&self) -> &str {
        // SAFETY: `ServieName` was created from a `&str` and therefore this conversion is safe
        unsafe { core::str::from_utf8_unchecked(self.value.as_bytes()) }
    }
}

impl core::fmt::Display for NodeName {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        std::write!(f, "{}", self.value)
    }
}

impl TryInto<NodeName> for &str {
    type Error = SemanticStringError;

    fn try_into(self) -> Result<NodeName, Self::Error> {
        NodeName::new(self)
    }
}

impl PartialEq<&str> for NodeName {
    fn eq(&self, other: &&str) -> bool {
        *self.as_str() == **other
    }
}

impl PartialEq<&str> for &NodeName {
    fn eq(&self, other: &&str) -> bool {
        *self.as_str() == **other
    }
}

impl core::ops::Deref for NodeName {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

struct NodeNameVisitor;

impl Visitor<'_> for NodeNameVisitor {
    type Value = NodeName;

    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        formatter.write_str("a string containing the service name")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match NodeName::new(v) {
            Ok(v) => Ok(v),
            Err(v) => Err(E::custom(format!("invalid node name provided {:?}.", v))),
        }
    }
}

impl<'de> Deserialize<'de> for NodeName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(NodeNameVisitor)
    }
}

impl Serialize for NodeName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(core::str::from_utf8(self.as_bytes()).unwrap())
    }
}
