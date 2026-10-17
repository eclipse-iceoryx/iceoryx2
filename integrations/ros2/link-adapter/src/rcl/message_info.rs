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

use r2r_rcl::rmw_message_info_t;

use crate::rcl::Gid;

/// Per-message metadata accompanying a take.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct MessageInfo {
    /// The originating DDS writer's ID.
    pub gid: Gid,
    pub source_timestamp_ns: i64,
    pub sequence_number: u64,
}

impl From<&rmw_message_info_t> for MessageInfo {
    fn from(info: &rmw_message_info_t) -> Self {
        Self {
            gid: Gid::from(&info.publisher_gid),
            source_timestamp_ns: info.source_timestamp,
            sequence_number: info.publication_sequence_number,
        }
    }
}
