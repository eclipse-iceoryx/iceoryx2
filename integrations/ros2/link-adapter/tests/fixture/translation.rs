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

use iceoryx2_integrations_ros2_link_adapter::{
    PassthroughTranslator, PlainStructTranslator, TopicTypes,
};
use iceoryx2_link_adapter::Translator;
use iceoryx2_link_conformance_tests::parameters::{
    FixedSizePayload, PayloadShape, PublishSubscribePayload,
};
use ros_env::std_msgs::msg::rmw;
use rosidl_runtime_rs::RmwMessage;

use super::payload::{SerializedString, UInt64};

/// A translator under test with the payload and the ROS 2 message type it
/// is tested with. A value of the payload converts to the message and back.
pub trait TranslatorUnderTest {
    type Translator: Translator<EndpointTypes = TopicTypes> + Default;
    type Payload: PublishSubscribePayload;
    type Message: RmwMessage
        + PartialEq
        + From<<Self::Payload as PayloadShape>::Value>
        + Into<<Self::Payload as PayloadShape>::Value>;
}

/// Tests the plain struct translator with `std_msgs/msg/UInt64`. Local
/// services carry the C struct.
pub struct PlainStruct;

impl TranslatorUnderTest for PlainStruct {
    type Translator = PlainStructTranslator;
    type Payload = FixedSizePayload<UInt64>;
    type Message = rmw::UInt64;
}

/// Tests the passthrough translator with `std_msgs/msg/String`. Local
/// services carry the serialized message.
pub struct Passthrough;

impl TranslatorUnderTest for Passthrough {
    type Translator = PassthroughTranslator;
    type Payload = SerializedString;
    type Message = rmw::String;
}
