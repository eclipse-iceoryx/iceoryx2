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

/// Length of the DDS writer GUID. rmw's GID buffer (`RMW_GID_STORAGE_SIZE`) can
/// be larger on some distributions (24 on Humble), but only the leading GUID
/// bytes are meaningful.
pub const DDS_GUID_LEN: usize = 16;

/// User header of bridged services, written by the gateway when ingesting a
/// ROS 2 message so subscribers can identify the remote origin.
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, ZeroCopySend)]
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
    /// declare it as their user header so the gateway can recognize them.
    pub fn type_detail() -> TypeDetail {
        TypeDetail::new::<RosHeader>(TypeVariant::FixedSize)
    }
}
