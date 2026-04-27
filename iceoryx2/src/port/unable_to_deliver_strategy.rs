// Copyright (c) 2025 Contributors to the Eclipse Foundation
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

use serde::{Deserialize, Serialize, de::Visitor};

/// Defines the strategy a sender shall pursue when the buffer of the receiver is full
/// and the service does not overflow.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum UnableToDeliverStrategy {
    /// Retries until the receiver has consumed some
    /// data from the full buffer and there is space again
    RetryUntilDelivered,
    /// Do not deliver the data to receiver with a full buffer
    DiscardData,
}

impl Serialize for UnableToDeliverStrategy {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&alloc::format!("{self:?}"))
    }
}

struct UnableToDeliverStrategyVisitor;

impl Visitor<'_> for UnableToDeliverStrategyVisitor {
    type Value = UnableToDeliverStrategy;

    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        formatter.write_str("a string containing either 'RetryUntilDelivered' or 'DiscardData'")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match v {
            "DiscardData" => Ok(UnableToDeliverStrategy::DiscardData),
            "RetryUntilDelivered" => Ok(UnableToDeliverStrategy::RetryUntilDelivered),
            v => Err(E::custom(alloc::format!(
                "Invalid UnableToDeliverStrategy provided: \"{v:?}\"."
            ))),
        }
    }
}

impl<'de> Deserialize<'de> for UnableToDeliverStrategy {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(UnableToDeliverStrategyVisitor)
    }
}
