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

//! The bridge contract for the `/chatter` demo, shared by the publisher and
//! subscriber binaries.

use iceoryx2::prelude::*;
use rosidl_runtime_rs::{Message, RmwMessage};

/// The iceoryx2 service mapped to the ROS 2 topic `/chatter`.
pub const SERVICE_NAME: &str = "ros2://topics/chatter";

/// The service payload is a byte slice holding the CDR-serialized message.
/// The type name is taken from the IDL-generated struct to match the
/// ROS side.
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct StdMsgStringByte(pub u8);

unsafe impl ZeroCopySend for StdMsgStringByte {
    unsafe fn type_name() -> &'static str {
        <<std_msgs::msg::String as Message>::RmwMsg as RmwMessage>::TYPE_NAME
    }
}

/// Byte view of a CDR payload slice.
pub fn as_bytes(payload: &[StdMsgStringByte]) -> &[u8] {
    // SAFETY: StdMsgStringByte is #[repr(transparent)] over u8, so length and
    // alignment carry over.
    unsafe { core::slice::from_raw_parts(payload.as_ptr().cast::<u8>(), payload.len()) }
}

// TODO: Move to common library.
/// User header of bridged services, written by the tunnel when ingesting a
/// ROS 2 message so subscribers can identify the remote origin.
#[derive(Debug, Default, Clone, Copy, ZeroCopySend)]
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
