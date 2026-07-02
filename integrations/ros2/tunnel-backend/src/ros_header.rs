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
use iceoryx2::service::static_config::message_type_details::{TypeDetail, TypeVariant};

use crate::rcl::MessageInfo;

/// Length of the DDS writer GUID. rmw's GID buffer (`RMW_GID_STORAGE_SIZE`) can
/// be larger on some distributions (24 on Humble), but only the leading GUID
/// bytes are meaningful.
const DDS_GUID_LEN: usize = 16;

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
    pub gid: [u8; DDS_GUID_LEN],
    /// Source timestamp in nanoseconds since the epoch.
    pub source_timestamp_ns: i64,
    /// Per-writer publication sequence number.
    pub sequence_number: u64,
}

const _: () = assert!(core::mem::size_of::<RosHeader>() == 32);
const _: () = assert!(core::mem::align_of::<RosHeader>() == 8);

impl RosHeader {
    /// The iceoryx2 [`TypeDetail`] describing this header. Bridged services
    /// declare it as their user header so the tunnel can recognize them.
    pub(crate) fn type_detail() -> TypeDetail {
        TypeDetail::new::<RosHeader>(TypeVariant::FixedSize)
    }
}

impl From<MessageInfo> for RosHeader {
    fn from(info: MessageInfo) -> Self {
        // The publisher GID in `rmw_message_info_t` is an `RMW_GID_STORAGE_SIZE`
        // buffer (24 bytes on Humble); the DDS GUID occupies only the leading 16
        // bytes, the rest is unused.
        let mut gid = [0u8; DDS_GUID_LEN];
        gid.copy_from_slice(&info.gid[..DDS_GUID_LEN]);
        Self {
            gid,
            source_timestamp_ns: info.source_timestamp_ns,
            sequence_number: info.sequence_number,
        }
    }
}
