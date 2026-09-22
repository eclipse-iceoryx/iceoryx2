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

mod passthrough;
mod plain_struct;

pub use passthrough::Passthrough;
pub use plain_struct::PlainStruct;

use iceoryx2_integrations_ros2_link_adapter::TopicTypes;
use iceoryx2_link_adapter::Translator;
use iceoryx2_link_conformance_tests::parameters::{PayloadShape, PublishSubscribePayload};
use rosidl_runtime_rs::RmwMessage;

/// A translator under test with the local payload and the ROS 2 message
/// type it is tested with.
pub trait TranslationUnderTest {
    type Translator: Translator<EndpointTypes = TopicTypes> + Default;
    type Payload: PublishSubscribePayload;
    type Message: RmwMessage;

    /// Serializes `value` as a ROS 2 node publishes it.
    fn to_wire(value: <Self::Payload as PayloadShape>::Value) -> Vec<u8>;

    /// Deserializes a message a ROS 2 node published.
    fn from_wire(wire: &[u8]) -> <Self::Payload as PayloadShape>::Value;
}
