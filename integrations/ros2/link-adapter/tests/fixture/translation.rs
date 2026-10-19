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
    MessageType, MirroredHeader, PassthroughTranslator, PlainStructTranslator,
};
use iceoryx2_link_adapter::{SampleShape, TranscodesSamples, Translator};
use iceoryx2_link_conformance_tests::parameters::{self, FixedSizePayload, PayloadShape};
use ros_env::example_interfaces::srv::rmw as srv;
use ros_env::std_msgs::msg::rmw;
use rosidl_runtime_rs::RmwMessage;

use super::payload::{
    AddTwoIntsRequest, AddTwoIntsResponse, SerializedAddTwoIntsRequest,
    SerializedAddTwoIntsResponse, SerializedString, UInt64,
};

/// A translator under test with the payloads and the ROS 2 message types
/// it is tested with, on a topic and on a service. A value of a payload
/// converts to its message and back.
pub trait TranslatorUnderTest {
    type Translator: Translator<SampleShape, RemoteTypes = MessageType, Transcoders: TranscodesSamples>;
    type SamplePayload: parameters::PublishSubscribePayload;
    type SampleMessage: RmwMessage
        + PartialEq
        + From<<Self::SamplePayload as PayloadShape>::Value>
        + Into<<Self::SamplePayload as PayloadShape>::Value>;
    type RequestPayload: parameters::RequestPayload;
    type RequestMessage: RmwMessage
        + PartialEq
        + From<<Self::RequestPayload as PayloadShape>::Value>
        + Into<<Self::RequestPayload as PayloadShape>::Value>;
    type ResponsePayload: parameters::ResponsePayload;
    type ResponseMessage: RmwMessage
        + PartialEq
        + From<<Self::ResponsePayload as PayloadShape>::Value>
        + Into<<Self::ResponsePayload as PayloadShape>::Value>;

    /// The translator as configured for the test.
    fn translator() -> Self::Translator;
}

/// Tests the plain struct translator with `std_msgs/msg/UInt64` and
/// `example_interfaces/srv/AddTwoInts`. Local services carry the C structs.
pub struct PlainStruct;

impl TranslatorUnderTest for PlainStruct {
    type Translator = PlainStructTranslator;
    type SamplePayload = FixedSizePayload<UInt64>;
    type SampleMessage = rmw::UInt64;
    type RequestPayload = FixedSizePayload<AddTwoIntsRequest>;
    type RequestMessage = srv::AddTwoInts_Request;
    type ResponsePayload = FixedSizePayload<AddTwoIntsResponse>;
    type ResponseMessage = srv::AddTwoInts_Response;

    fn translator() -> PlainStructTranslator {
        PlainStructTranslator::default()
    }
}

/// Tests the plain struct translator with `std_msgs/msg/UInt64` and
/// `example_interfaces/srv/AddTwoInts`. Mirrored services carry the
/// `RosHeader`.
pub struct PlainStructWithHeader;

impl TranslatorUnderTest for PlainStructWithHeader {
    type Translator = PlainStructTranslator;
    type SamplePayload = FixedSizePayload<UInt64>;
    type SampleMessage = rmw::UInt64;
    type RequestPayload = FixedSizePayload<AddTwoIntsRequest>;
    type RequestMessage = srv::AddTwoInts_Request;
    type ResponsePayload = FixedSizePayload<AddTwoIntsResponse>;
    type ResponseMessage = srv::AddTwoInts_Response;

    fn translator() -> PlainStructTranslator {
        PlainStructTranslator {
            header: MirroredHeader::RosHeader,
        }
    }
}

/// Tests the passthrough translator with `std_msgs/msg/String` and
/// `example_interfaces/srv/AddTwoInts`. Local services carry the
/// serialized messages.
pub struct Passthrough;

impl TranslatorUnderTest for Passthrough {
    type Translator = PassthroughTranslator;
    type SamplePayload = SerializedString;
    type SampleMessage = rmw::String;
    type RequestPayload = SerializedAddTwoIntsRequest;
    type RequestMessage = srv::AddTwoInts_Request;
    type ResponsePayload = SerializedAddTwoIntsResponse;
    type ResponseMessage = srv::AddTwoInts_Response;

    fn translator() -> PassthroughTranslator {
        PassthroughTranslator::default()
    }
}

/// Tests the passthrough translator with `std_msgs/msg/String` and
/// `example_interfaces/srv/AddTwoInts`. Mirrored services carry the
/// `RosHeader`.
pub struct PassthroughWithHeader;

impl TranslatorUnderTest for PassthroughWithHeader {
    type Translator = PassthroughTranslator;
    type SamplePayload = SerializedString;
    type SampleMessage = rmw::String;
    type RequestPayload = SerializedAddTwoIntsRequest;
    type RequestMessage = srv::AddTwoInts_Request;
    type ResponsePayload = SerializedAddTwoIntsResponse;
    type ResponseMessage = srv::AddTwoInts_Response;

    fn translator() -> PassthroughTranslator {
        PassthroughTranslator {
            header: MirroredHeader::RosHeader,
        }
    }
}
