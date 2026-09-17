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

pub use iceoryx2_integrations_ros2_interop::ros_header::{DDS_GUID_LEN, RosHeader};

use crate::rcl::MessageInfo;

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
