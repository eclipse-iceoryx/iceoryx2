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
use iceoryx2_integrations_ros2_link_adapter::{
    PassthroughTranslator, PlainStructTranslator, TopicTypes,
};
use iceoryx2_link_adapter::Translator;
use iceoryx2_link_conformance_tests::parameters::{
    FixedSizePayload, PayloadShape, PublishSubscribePayload, SlicePayload,
};

/// The CDR encapsulation header preceding every wire message.
pub const CDR_HEADER: [u8; 4] = [0x00, 0x01, 0x00, 0x00];

/// A translator under test. Encodes and decodes the suites' payloads in the
/// same way the peer node puts them on the wire.
pub trait WireForm: PublishSubscribePayload {
    type Translator: Translator<EndpointTypes = TopicTypes> + Default;

    /// The ROS 2 type on the topic.
    const TYPE_NAME: &'static str;

    /// The wire form of `value`.
    fn encode(value: <Self as PayloadShape>::Value) -> Vec<u8>;

    /// The value `wire` carries.
    fn decode(wire: &[u8]) -> <Self as PayloadShape>::Value;
}

/// The C struct of `std_msgs/msg/UInt64`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, ZeroCopySend)]
#[type_name("std_msgs/msg/UInt64")]
#[repr(C)]
pub struct UInt64 {
    data: u64,
}

impl From<u64> for UInt64 {
    fn from(data: u64) -> Self {
        Self { data }
    }
}

/// The payload for testing the plain struct translator. Local services
/// carry the C struct, ROS 2 topics its CDR serialization.
pub type PlainStruct = FixedSizePayload<UInt64>;

impl WireForm for PlainStruct {
    type Translator = PlainStructTranslator;

    const TYPE_NAME: &'static str = "std_msgs/msg/UInt64";

    fn encode(payload: UInt64) -> Vec<u8> {
        let mut wire = CDR_HEADER.to_vec();
        wire.extend_from_slice(&payload.data.to_le_bytes());
        wire
    }

    fn decode(wire: &[u8]) -> UInt64 {
        let (header, data) = wire.split_at(CDR_HEADER.len());
        assert_eq!(header, CDR_HEADER, "a message starts with the CDR header");
        let data: [u8; 8] = data.try_into().expect("a message carries one u64");
        UInt64::from(u64::from_le_bytes(data))
    }
}

/// One byte of a serialized `std_msgs/msg/String`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, ZeroCopySend)]
#[type_name("std_msgs/msg/String")]
#[repr(C)]
pub struct RosString(u8);

impl From<u8> for RosString {
    fn from(byte: u8) -> Self {
        Self(byte)
    }
}

/// The payload for testing the passthrough translator. Local services and
/// ROS 2 topics both carry the CDR serialization.
pub type Passthrough = SlicePayload<RosString>;

impl WireForm for Passthrough {
    type Translator = PassthroughTranslator;

    const TYPE_NAME: &'static str = "std_msgs/msg/String";

    fn encode(payload: Vec<RosString>) -> Vec<u8> {
        payload.into_iter().map(|byte| byte.0).collect()
    }

    fn decode(wire: &[u8]) -> Vec<RosString> {
        wire.iter().copied().map(RosString).collect()
    }
}
