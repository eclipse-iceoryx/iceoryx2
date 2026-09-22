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

use iceoryx2_integrations_ros2_link_adapter::PassthroughTranslator;
use iceoryx2_link_conformance_tests::parameters::SlicePayload;
use ros_env::std_msgs::msg::rmw;

use super::TranslationUnderTest;
use crate::fixture::payload::StringByte;

/// Tests the passthrough translator with `std_msgs/msg/String`. Local
/// services and the topic both carry the serialized message.
pub struct Passthrough;

impl TranslationUnderTest for Passthrough {
    type Translator = PassthroughTranslator;
    type Payload = SlicePayload<StringByte>;
    type Message = rmw::String;

    fn to_wire(value: Vec<StringByte>) -> Vec<u8> {
        value.into_iter().map(|byte| byte.0).collect()
    }

    fn from_wire(wire: &[u8]) -> Vec<StringByte> {
        wire.iter().copied().map(StringByte).collect()
    }
}
