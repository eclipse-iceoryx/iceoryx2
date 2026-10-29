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
use r2r_rcl::{RMW_GID_STORAGE_SIZE, rmw_gid_t};

/// The number of leading bytes of a GUID that name its participant.
const PARTICIPANT_PREFIX_NUMBER_OF_BYTES: usize = 12;

/// Wraps the RMW identifier of an endpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Gid([u8; RMW_GID_STORAGE_SIZE as usize]);

impl Gid {
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// The participant the endpoint belongs to.
    pub fn participant_prefix(&self) -> ParticipantPrefix {
        ParticipantPrefix::of_guid(&self.0)
    }
}

/// The leading bytes of a GUID, which name the participant an entity
/// belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParticipantPrefix([u8; PARTICIPANT_PREFIX_NUMBER_OF_BYTES]);

impl ParticipantPrefix {
    /// The prefix of `guid`, which must be at least as long as the prefix.
    pub(crate) fn of_guid(guid: &[u8]) -> Self {
        let mut prefix = [0; PARTICIPANT_PREFIX_NUMBER_OF_BYTES];
        prefix.copy_from_slice(&guid[..PARTICIPANT_PREFIX_NUMBER_OF_BYTES]);
        Self(prefix)
    }
}

impl From<&rmw_gid_t> for Gid {
    fn from(gid: &rmw_gid_t) -> Self {
        Self(gid.data)
    }
}
