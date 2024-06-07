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

const NODE_NAME_LENGTH: usize = 255;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeName {
    value: iceoryx2_bb_container::byte_string::FixedSizeByteString<NODE_NAME_LENGTH>,
}

impl NodeName {
    /// Creates a new [`NodeName`]. Is not allowed to be longer than [`NodeName::max_len()`].
    pub fn new(name: &str) -> Result<Self, SemanticStringError> {
        Ok(Self {
            value: iceoryx2_bb_container::byte_string::FixedSizeByteString::from_bytes(
                name.as_bytes(),
            )?,
        })
    }

    /// Returns a str reference to the [`NodeName`]
    pub fn as_str(&self) -> &str {
        // SAFETY: `ServieName` was created from a `&str` and therefore this conversion is safe
        unsafe { std::str::from_utf8_unchecked(self.value.as_bytes()) }
    }

    /// Returns the maximum length of a [`NodeName`]
    pub const fn max_len() -> usize {
        NODE_NAME_LENGTH
    }
}

impl std::fmt::Display for NodeName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::write!(f, "{}", self.value)
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

impl std::ops::Deref for NodeName {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

struct NodeNameVisitor;

impl<'de> Visitor<'de> for NodeNameVisitor {
    type Value = NodeName;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
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
        serializer.serialize_str(std::str::from_utf8(self.as_bytes()).unwrap())
    }
}
