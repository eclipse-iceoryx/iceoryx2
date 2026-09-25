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

use iceoryx2_link_adapter::{EndpointDescription, EndpointTypes};
use iceoryx2_link_backend::service_description::Identified;

use crate::config::{TopicName, TypeName};
use crate::qos::QosProfile;

/// The settings of a ROS 2 endpoint.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TopicSettings {
    pub topic: TopicName,
    pub qos: QosProfile,
}

impl Identified for TopicSettings {
    type Id = TopicName;

    fn id(&self) -> TopicName {
        self.topic.clone()
    }
}

/// The types of a ROS 2 endpoint.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TopicTypes {
    pub type_name: TypeName,
}

/// The description of a ROS 2 endpoint.
pub type TopicDescription = EndpointDescription<TopicSettings, EndpointTypes<TopicTypes>>;
