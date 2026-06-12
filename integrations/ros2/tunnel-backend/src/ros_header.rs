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

use iceoryx2::prelude::*;

use crate::rcl::MessageInfo;

/// User header of bridged services, written by the tunnel when ingesting a
/// ROS 2 message into iceoryx2 so that local subscribers can identify the
/// remote origin. Applications in other languages can rely on the layout:
/// 32 bytes, alignment 8, field order as declared, little-endian on the
/// usual platforms.
#[derive(Debug, Clone, Copy, Eq, PartialEq, ZeroCopySend)]
#[type_name("RosHeader")]
#[repr(C)]
pub struct RosHeader {
    /// The originating DDS writer's GUID.
    pub gid: [u8; 16],
    /// Source timestamp in nanoseconds since the epoch.
    pub source_timestamp_ns: i64,
    /// Per-writer publication sequence number.
    pub sequence_number: u64,
}

const _: () = assert!(core::mem::size_of::<RosHeader>() == 32);
const _: () = assert!(core::mem::align_of::<RosHeader>() == 8);

impl RosHeader {
    pub(crate) fn from_message_info(info: &MessageInfo) -> Self {
        Self {
            gid: info.gid,
            source_timestamp_ns: info.source_timestamp_ns,
            sequence_number: info.sequence_number,
        }
    }
}
