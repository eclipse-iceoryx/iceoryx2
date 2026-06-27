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

use alloc::format;

use iceoryx2_bb_container::{semantic_string::SemanticStringError, string::*};
use iceoryx2_bb_derive_macros::{PlacementDefault, ZeroCopySend};
use iceoryx2_bb_elementary_traits::placement_default::PlacementDefault;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use iceoryx2_log::fail;

use crate::constants::MAX_PORT_NAME_LENGTH;

use serde::{Deserialize, Serialize, de::Visitor};

type PortNameString = StaticString<MAX_PORT_NAME_LENGTH>;

/// Represents the name for a port like [`crate::port::publisher::Publisher`], [`crate::port::server::Server`] or [`crate::port::listener::Listener`].
#[derive(
    PlacementDefault, ZeroCopySend, Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord,
)]
#[repr(C)]
pub struct PortName {
    value: PortNameString,
}

impl PortName {
    /// Creates a new [`PortName`].
    /// If the provided name does not contain a valid [`PortName`] it will return a
    /// [`SemanticStringError`] otherwise the [`PortName`].
    pub fn new(name: &str) -> Result<Self, SemanticStringError> {
        Ok(Self {
            value: fail!(from "PortName::new()",
                         when PortNameString::try_from(name),
                         "The string \"{}\" is not a valid port name.",
                         name),
        })
    }

    /// Creates a new empty [`PortName`].
    pub fn new_empty() -> Self {
        Self {
            value: PortNameString::default(),
        }
    }

    /// Returns a str reference to the [`PortName`]
    pub fn as_str(&self) -> &str {
        // SAFETY: `PortName` was created from a `&str` and therefore this conversion is safe
        unsafe { core::str::from_utf8_unchecked(self.value.as_bytes()) }
    }

    /// Returns the maximum length of [`PortName`]
    pub fn max_len() -> usize {
        PortNameString::capacity()
    }
}

impl core::fmt::Display for PortName {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl TryInto<PortName> for &str {
    type Error = SemanticStringError;

    fn try_into(self) -> Result<PortName, Self::Error> {
        PortName::new(self)
    }
}

impl PartialEq<&str> for PortName {
    fn eq(&self, other: &&str) -> bool {
        *self.as_str() == **other
    }
}

impl PartialEq<&str> for &PortName {
    fn eq(&self, other: &&str) -> bool {
        *self.as_str() == **other
    }
}

impl core::ops::Deref for PortName {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

struct PortNameVisitor;

impl Visitor<'_> for PortNameVisitor {
    type Value = PortName;

    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        formatter.write_str("a string containing the service name")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match PortName::new(v) {
            Ok(v) => Ok(v),
            Err(v) => Err(E::custom(format!("invalid port name provided {v:?}."))),
        }
    }
}

impl<'de> Deserialize<'de> for PortName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(PortNameVisitor)
    }
}

impl Serialize for PortName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(core::str::from_utf8(self.as_bytes()).unwrap())
    }
}
