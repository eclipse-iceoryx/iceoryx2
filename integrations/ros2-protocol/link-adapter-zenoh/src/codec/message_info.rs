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

use iceoryx2_integrations_ros2_interop::RosHeader;

use super::gid::{GID_SIZE, Gid};

const SEQUENCE_NUMBER_SIZE: usize = size_of::<i64>();
const SOURCE_TIMESTAMP_SIZE: usize = size_of::<i64>();
const GID_LENGTH_PREFIX: u8 = 0x10;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum MessageInfoError {
    InvalidLength,
    InvalidGidLengthPrefix,
}

impl core::fmt::Display for MessageInfoError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "MessageInfoError::{self:?}")
    }
}

impl core::error::Error for MessageInfoError {}

/// Per-message metadata that `rmw_zenoh_cpp` sends as the zenoh attachment of
/// every sample, query and reply.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct MessageInfo {
    pub sequence_number: i64,
    /// Nanoseconds since the epoch.
    pub source_timestamp: i64,
    pub gid: Gid,
}

impl MessageInfo {
    pub fn encode(&self) -> Vec<u8> {
        [
            &self.sequence_number.to_le_bytes()[..],
            &self.source_timestamp.to_le_bytes(),
            &[GID_LENGTH_PREFIX],
            &self.gid.0,
        ]
        .concat()
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, MessageInfoError> {
        let Some((sequence_number, rest)) = bytes.split_first_chunk::<SEQUENCE_NUMBER_SIZE>()
        else {
            return Err(MessageInfoError::InvalidLength);
        };
        let Some((source_timestamp, rest)) = rest.split_first_chunk::<SOURCE_TIMESTAMP_SIZE>()
        else {
            return Err(MessageInfoError::InvalidLength);
        };
        let Some((gid_length_prefix, gid)) = rest.split_first() else {
            return Err(MessageInfoError::InvalidLength);
        };
        let Ok(gid) = <[u8; GID_SIZE]>::try_from(gid) else {
            return Err(MessageInfoError::InvalidLength);
        };
        if *gid_length_prefix != GID_LENGTH_PREFIX {
            return Err(MessageInfoError::InvalidGidLengthPrefix);
        }
        Ok(Self {
            sequence_number: i64::from_le_bytes(*sequence_number),
            source_timestamp: i64::from_le_bytes(*source_timestamp),
            gid: Gid(gid),
        })
    }
}

impl From<&RosHeader> for MessageInfo {
    fn from(header: &RosHeader) -> Self {
        Self {
            sequence_number: header.sequence_number as i64,
            source_timestamp: header.source_timestamp_ns,
            gid: Gid(header.gid),
        }
    }
}

impl From<&MessageInfo> for RosHeader {
    fn from(message_info: &MessageInfo) -> Self {
        Self {
            gid: message_info.gid.0,
            source_timestamp_ns: message_info.source_timestamp,
            sequence_number: message_info.sequence_number as u64,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ATTACHMENT_SIZE: usize = 33;
    const CAPTURED: [u8; ATTACHMENT_SIZE] = [
        0x12, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x1f, 0xcf, 0x44, 0x19, 0x28, 0xb6, 0xea,
        0x18, 0x10, 0x2a, 0xaf, 0xd1, 0xf8, 0xdf, 0xec, 0xe2, 0x28, 0x8c, 0xb9, 0x8d, 0x1f, 0x04,
        0xed, 0x0d, 0x66,
    ];
    const CAPTURED_SEQUENCE_NUMBER: i64 = 18;
    const CAPTURED_SOURCE_TIMESTAMP: i64 = 0x18ea_b628_1944_cf1f;
    const CAPTURED_GID: [u8; GID_SIZE] = [
        0x2a, 0xaf, 0xd1, 0xf8, 0xdf, 0xec, 0xe2, 0x28, 0x8c, 0xb9, 0x8d, 0x1f, 0x04, 0xed, 0x0d,
        0x66,
    ];
    const GID_LENGTH_PREFIX_OFFSET: usize = SEQUENCE_NUMBER_SIZE + SOURCE_TIMESTAMP_SIZE;
    const TRUNCATED_SIZE: usize = 32;
    const WRONG_GID_LENGTH_PREFIX: u8 = 0x11;
    const NEGATIVE_SEQUENCE_NUMBER: i64 = -1;

    fn captured() -> MessageInfo {
        MessageInfo {
            sequence_number: CAPTURED_SEQUENCE_NUMBER,
            source_timestamp: CAPTURED_SOURCE_TIMESTAMP,
            gid: Gid(CAPTURED_GID),
        }
    }

    #[test]
    fn decodes_the_captured_attachment() {
        assert_eq!(MessageInfo::decode(&CAPTURED), Ok(captured()));
    }

    #[test]
    fn encodes_the_captured_attachment() {
        assert_eq!(captured().encode(), CAPTURED);
    }

    #[test]
    fn attachment_of_wrong_length_is_rejected() {
        let longer = [&CAPTURED[..], &[GID_LENGTH_PREFIX]].concat();
        for input in [
            &[][..],
            &CAPTURED[..SEQUENCE_NUMBER_SIZE],
            &CAPTURED[..GID_LENGTH_PREFIX_OFFSET],
            &CAPTURED[..TRUNCATED_SIZE],
            &longer,
        ] {
            assert_eq!(
                MessageInfo::decode(input),
                Err(MessageInfoError::InvalidLength),
                "{input:?}"
            );
        }
    }

    #[test]
    fn attachment_with_wrong_gid_length_prefix_is_rejected() {
        let mut bytes = CAPTURED;
        bytes[GID_LENGTH_PREFIX_OFFSET] = WRONG_GID_LENGTH_PREFIX;

        assert_eq!(
            MessageInfo::decode(&bytes),
            Err(MessageInfoError::InvalidGidLengthPrefix)
        );
    }

    #[test]
    fn converts_from_and_to_the_ros_header() {
        let message_info = MessageInfo {
            sequence_number: NEGATIVE_SEQUENCE_NUMBER,
            ..captured()
        };
        let header = RosHeader::from(&message_info);

        assert_eq!(header.gid, CAPTURED_GID);
        assert_eq!(header.source_timestamp_ns, CAPTURED_SOURCE_TIMESTAMP);
        assert_eq!(header.sequence_number, u64::MAX);
        assert_eq!(MessageInfo::from(&header), message_info);
    }
}
