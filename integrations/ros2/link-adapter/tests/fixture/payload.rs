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

use iceoryx2::prelude::ZeroCopySend;
use iceoryx2_link_conformance_tests::parameters::SliceElement;
use ros_env::std_msgs::msg::rmw;

use super::serialization;

/// The C struct of `std_msgs/msg/UInt64`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, ZeroCopySend)]
#[type_name("std_msgs/msg/UInt64")]
#[repr(C)]
pub struct UInt64 {
    pub data: u64,
}

impl From<u64> for UInt64 {
    fn from(data: u64) -> Self {
        Self { data }
    }
}

/// One byte of a serialized `std_msgs/msg/String`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, ZeroCopySend)]
#[type_name("std_msgs/msg/String")]
#[repr(C)]
pub struct StringByte(pub u8);

/// The serialized message reading `message <n>`.
impl SliceElement for StringByte {
    fn slice(n: u64) -> Vec<StringByte> {
        let message = rmw::String {
            data: format!("message {n}").as_str().into(),
        };
        serialization::serialize(&message)
            .into_iter()
            .map(StringByte)
            .collect()
    }
}
