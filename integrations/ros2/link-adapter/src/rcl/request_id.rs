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

use r2r_rcl::rmw_request_id_t;

use crate::rcl::ParticipantPrefix;

/// The number an rcl client assigns to each request it sends.
pub type SequenceNumber = i64;

/// Identifies a request at a ROS 2 service by the requesting client and
/// the sequence number the client assigned.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RequestId {
    pub writer_guid: [u8; 16],
    pub sequence_number: SequenceNumber,
}

impl RequestId {
    /// The participant of the client that sent the request.
    pub fn participant_prefix(&self) -> ParticipantPrefix {
        ParticipantPrefix::of_guid(&self.writer_guid)
    }
}

/// The element type rmw declares for a request id's GUID, which is `i8` in
/// Humble and `u8` from Iron on.
trait GuidByte: Copy {
    fn from_u8(byte: u8) -> Self;
    fn to_u8(self) -> u8;
}

impl GuidByte for u8 {
    fn from_u8(byte: u8) -> Self {
        byte
    }

    fn to_u8(self) -> u8 {
        self
    }
}

impl GuidByte for i8 {
    fn from_u8(byte: u8) -> Self {
        byte.cast_signed()
    }

    fn to_u8(self) -> u8 {
        self.cast_unsigned()
    }
}

impl From<&rmw_request_id_t> for RequestId {
    fn from(id: &rmw_request_id_t) -> Self {
        Self {
            writer_guid: id.writer_guid.map(GuidByte::to_u8),
            sequence_number: id.sequence_number,
        }
    }
}

impl From<&RequestId> for rmw_request_id_t {
    fn from(id: &RequestId) -> Self {
        Self {
            writer_guid: id.writer_guid.map(GuidByte::from_u8),
            sequence_number: id.sequence_number,
        }
    }
}
