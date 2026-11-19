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

use iceoryx2::service::marker::{flatbuffer_type_name, is_flatbuffer_type_name};
use iceoryx2::service::service_name::ServiceName;
use iceoryx2_integrations_ros2_link_adapter::mapping::static_mapping::{Config, Entry};

/// A static mapping entry whose flatbuffer payload type does not
/// correspond to its ROS 2 type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeMismatch {
    /// The payload type names no root type.
    RootTypeMissing { service_name: ServiceName },
    /// The payload type's root type corresponds to another ROS 2 type.
    Disagrees {
        service_name: ServiceName,
        payload_type: String,
        ros2_type: String,
    },
}

impl core::fmt::Display for TypeMismatch {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            TypeMismatch::RootTypeMissing { service_name } => write!(
                f,
                "Service '{}' has a flatbuffer payload type without a root type, a service records one",
                service_name.as_str()
            ),
            TypeMismatch::Disagrees {
                service_name,
                payload_type,
                ros2_type,
            } => write!(
                f,
                "Service '{}' has the payload type '{}', its entry defines the ROS 2 type '{}'",
                service_name.as_str(),
                payload_type,
                ros2_type
            ),
        }
    }
}

/// The entries of `config` whose flatbuffer payload type does not
/// correspond to its ROS 2 type.
///
/// Only entries with a flatbuffer payload type are checked. The request
/// and response of a service entry correspond to the service type with
/// `_Request` and `_Response` appended.
pub fn type_mismatches(config: &Config) -> Vec<TypeMismatch> {
    let mut mismatches = Vec::new();

    for entry in &config.entries {
        match entry {
            Entry::Topic(topic) => mismatches.extend(type_mismatch(
                &topic.iceoryx2.service_name,
                &topic.iceoryx2.payload_type,
                topic.ros2.type_name.as_str(),
            )),
            Entry::Service(service) => {
                let service_type = service.ros2.type_name.as_str();
                mismatches.extend(type_mismatch(
                    &service.iceoryx2.service_name,
                    &service.iceoryx2.request_type,
                    &format!("{service_type}_Request"),
                ));
                mismatches.extend(type_mismatch(
                    &service.iceoryx2.service_name,
                    &service.iceoryx2.response_type,
                    &format!("{service_type}_Response"),
                ));
            }
        }
    }

    mismatches
}

/// The mismatch between the flatbuffer `payload_type` of the service
/// `service_name` and `ros2_type`, if any.
fn type_mismatch(
    service_name: &ServiceName,
    payload_type: &str,
    ros2_type: &str,
) -> Option<TypeMismatch> {
    if !is_flatbuffer_type_name(payload_type) {
        return None;
    }
    let expected = flatbuffer_type_name(&ros2_type.replace('/', "."));
    if expected.is_some_and(|expected| expected == payload_type) {
        return None;
    }
    Some(if payload_type.contains('<') {
        TypeMismatch::Disagrees {
            service_name: *service_name,
            payload_type: payload_type.to_string(),
            ros2_type: ros2_type.to_string(),
        }
    } else {
        TypeMismatch::RootTypeMissing {
            service_name: *service_name,
        }
    })
}
