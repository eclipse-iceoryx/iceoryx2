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

use alloc::vec::Vec;
use core::fmt;
use core::str::FromStr;

use iceoryx2::identifiers::UniqueNodeId;
use serde::{Deserialize, Serialize};

/// Number of bytes a [`BackendId`] holds. Sized for 128-bit native
/// identities.
pub const BACKEND_ID_LENGTH: usize = 16;

/// Identifies one backend instance.
///
/// To be created by the backend from its native peer identity. The gateway
/// treats the content as opaque bytes.
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct BackendId([u8; BACKEND_ID_LENGTH]);

impl BackendId {
    pub fn new(bytes: [u8; BACKEND_ID_LENGTH]) -> Self {
        Self(bytes)
    }
}

impl fmt::Display for BackendId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}

impl fmt::Debug for BackendId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BackendId({self})")
    }
}

/// Identifies one gateway instance.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct GatewayId {
    node: UniqueNodeId,
    backend: BackendId,
}

impl GatewayId {
    pub fn new(node: UniqueNodeId, backend: BackendId) -> Self {
        Self { node, backend }
    }

    pub fn backend(&self) -> &BackendId {
        &self.backend
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum GatewayIdParseError {
    InvalidHex,
    InvalidEncoding,
}

impl fmt::Display for GatewayIdParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "GatewayIdParseError::{self:?}")
    }
}

impl core::error::Error for GatewayIdParseError {}

impl fmt::Display for GatewayId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let bytes = postcard::to_allocvec(self).map_err(|_| fmt::Error)?;
        for byte in bytes {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}

impl FromStr for GatewayId {
    type Err = GatewayIdParseError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let bytes = decode_hex(text)?;
        let (id, remainder) = postcard::take_from_bytes(&bytes)
            .map_err(|_| GatewayIdParseError::InvalidEncoding)?;
        if !remainder.is_empty() {
            return Err(GatewayIdParseError::InvalidEncoding);
        }
        Ok(id)
    }
}

fn decode_hex(text: &str) -> Result<Vec<u8>, GatewayIdParseError> {
    if !text.is_ascii() || !text.len().is_multiple_of(2) {
        return Err(GatewayIdParseError::InvalidHex);
    }
    (0..text.len())
        .step_by(2)
        .map(|cursor| {
            u8::from_str_radix(&text[cursor..cursor + 2], 16)
                .map_err(|_| GatewayIdParseError::InvalidHex)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;
    use iceoryx2::node::NodeBuilder;
    use iceoryx2::service::local;
    use iceoryx2_bb_testing::assert_that;

    fn gateway_id() -> GatewayId {
        let node = NodeBuilder::new()
            .create::<local::Service>()
            .expect("node creation succeeds");
        let backend = BackendId::new([0xab; BACKEND_ID_LENGTH]);
        GatewayId::new(*node.id(), backend)
    }

    #[test]
    fn gateway_id_round_trips_through_text() {
        let id = gateway_id();

        let parsed: GatewayId = id.to_string().parse().expect("valid text");

        assert_that!(parsed, eq id);
    }

    #[test]
    fn gateway_id_text_contains_only_hex_digits() {
        let text = gateway_id().to_string();

        assert_that!(text.chars().all(|c| c.is_ascii_hexdigit()), eq true);
    }

    #[test]
    fn gateway_id_rejects_malformed_text() {
        assert_that!("abc".parse::<GatewayId>(), eq Err(GatewayIdParseError::InvalidHex));
        assert_that!("zz".parse::<GatewayId>(), eq Err(GatewayIdParseError::InvalidHex));
        assert_that!("00".parse::<GatewayId>(), eq Err(GatewayIdParseError::InvalidEncoding));
    }

    #[test]
    fn gateway_id_rejects_text_with_trailing_bytes() {
        let mut text = gateway_id().to_string();
        text.push_str("00");

        assert_that!(text.parse::<GatewayId>(), eq Err(GatewayIdParseError::InvalidEncoding));
    }
}
