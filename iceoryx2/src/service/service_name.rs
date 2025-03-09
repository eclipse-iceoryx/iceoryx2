// Copyright (c) 2023 Contributors to the Eclipse Foundation
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

//! # Example
//!
//! ```
//! use iceoryx2::prelude::*;
//!
//! # fn main() -> Result<(), Box<dyn core::error::Error>> {
//! let service_name = ServiceName::new("My/Funk/ServiceName")?;
//!
//! # Ok(())
//! # }
//! ```

use iceoryx2_bb_container::semantic_string::SemanticStringError;
use serde::{de::Visitor, Deserialize, Serialize};

/// The name of a [`Service`](crate::service::Service).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ServiceName {
    value: String,
}

impl ServiceName {
    /// Creates a new [`ServiceName`]. The name is not allowed to be empty.
    pub fn new(name: &str) -> Result<Self, SemanticStringError> {
        if name.is_empty() {
            return Err(SemanticStringError::InvalidContent);
        }

        Ok(Self { value: name.into() })
    }

    /// Returns a str reference to the [`ServiceName`]
    pub fn as_str(&self) -> &str {
        &self.value
    }
}

impl core::fmt::Display for ServiceName {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        std::write!(f, "{}", self.value)
    }
}

impl TryInto<ServiceName> for &str {
    type Error = SemanticStringError;

    fn try_into(self) -> Result<ServiceName, Self::Error> {
        ServiceName::new(self)
    }
}

impl PartialEq<&str> for ServiceName {
    fn eq(&self, other: &&str) -> bool {
        *self.as_str() == **other
    }
}

impl PartialEq<&str> for &ServiceName {
    fn eq(&self, other: &&str) -> bool {
        *self.as_str() == **other
    }
}

impl core::ops::Deref for ServiceName {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

struct ServiceNameVisitor;

impl Visitor<'_> for ServiceNameVisitor {
    type Value = ServiceName;

    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        formatter.write_str("a string containing the service name")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match ServiceName::new(v) {
            Ok(v) => Ok(v),
            Err(v) => Err(E::custom(format!("invalid service name provided {:?}.", v))),
        }
    }
}

impl<'de> Deserialize<'de> for ServiceName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(ServiceNameVisitor)
    }
}

impl Serialize for ServiceName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(core::str::from_utf8(self.as_bytes()).unwrap())
    }
}
