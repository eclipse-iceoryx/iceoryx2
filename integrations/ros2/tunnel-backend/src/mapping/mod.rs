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

//! Mapping between iceoryx2 services and their ROS 2 representation.

mod prefix_mapping;

pub use prefix_mapping::PrefixMapping;

use crate::config::{TopicName, TypeName};
use crate::qos::QosProfile;

/// The concrete EndpointDescription for ROS 2 endpoints.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TopicDescription {
    pub topic: TopicName,
    pub type_name: TypeName,
    pub qos: QosProfile,
}
