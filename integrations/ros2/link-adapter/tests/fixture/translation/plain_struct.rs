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

use iceoryx2_integrations_ros2_link_adapter::PlainStructTranslator;
use iceoryx2_link_conformance_tests::parameters::FixedSizePayload;
use ros_env::std_msgs::msg::rmw;

use super::TranslationUnderTest;
use crate::fixture::payload::UInt64;
use crate::fixture::serialization;

/// Tests the plain struct translator with `std_msgs/msg/UInt64`. Local
/// services carry the message's C struct and the topic its serialization.
pub struct PlainStruct;

impl TranslationUnderTest for PlainStruct {
    type Translator = PlainStructTranslator;
    type Payload = FixedSizePayload<UInt64>;
    type Message = rmw::UInt64;

    fn to_wire(value: UInt64) -> Vec<u8> {
        serialization::serialize(&rmw::UInt64 { data: value.data })
    }

    fn from_wire(wire: &[u8]) -> UInt64 {
        UInt64::from(serialization::deserialize::<rmw::UInt64>(wire).data)
    }
}
