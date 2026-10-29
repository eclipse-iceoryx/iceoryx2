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

use iceoryx2_link_adapter::EndpointTypes;
use iceoryx2_link_backend::service_description::Identified;

use crate::config::{ServiceName, TopicName, TypeName};
use crate::qos::QosProfile;

/// The name of a ROS 2 endpoint.
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum EndpointName {
    Topic(TopicName),
    Service(ServiceName),
}

impl core::fmt::Display for EndpointName {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Topic(topic) => topic.fmt(f),
            Self::Service(service) => service.fmt(f),
        }
    }
}

/// The settings of a ROS 2 endpoint.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EndpointSettings {
    pub name: EndpointName,
    pub qos: QosProfile,
}

impl Identified for EndpointSettings {
    type Id = EndpointName;

    fn id(&self) -> EndpointName {
        self.name.clone()
    }
}

/// The message type of a ROS 2 endpoint.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct MessageType {
    pub type_name: TypeName,
}

/// The description of a ROS 2 endpoint.
pub type EndpointDescription =
    iceoryx2_link_adapter::EndpointDescription<EndpointSettings, EndpointTypes<MessageType>>;
