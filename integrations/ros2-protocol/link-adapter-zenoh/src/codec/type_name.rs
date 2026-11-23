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

const ROS_SEPARATOR: &str = "/";
const RMW_SEPARATOR: &str = "::";
const DDS_NAMESPACE: &str = "dds_";
const RMW_NAME_SUFFIX: &str = "_";
const REQUEST_SUFFIX: &str = "_Request";
const RESPONSE_SUFFIX: &str = "_Response";

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TypeNameError {
    InvalidPartCount,
    EmptyPart,
    MissingDdsNamespace,
    MissingNameSuffix,
}

impl core::fmt::Display for TypeNameError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "TypeNameError::{self:?}")
    }
}

impl core::error::Error for TypeNameError {}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TypeName {
    package: String,
    interface: String,
    name: String,
}

impl TypeName {
    /// Parses a ROS 2 type name, such as `std_msgs/msg/String`.
    pub fn from_ros(type_name: &str) -> Result<Self, TypeNameError> {
        let parts: Vec<&str> = type_name.split(ROS_SEPARATOR).collect();
        let [package, interface, name] = parts.as_slice() else {
            return Err(TypeNameError::InvalidPartCount);
        };
        Self::new(package, interface, name)
    }

    /// Parses an rmw type name, such as `std_msgs::msg::dds_::String_`.
    pub fn from_rmw(type_name: &str) -> Result<Self, TypeNameError> {
        let parts: Vec<&str> = type_name.split(RMW_SEPARATOR).collect();
        let [package, interface, namespace, name] = parts.as_slice() else {
            return Err(TypeNameError::InvalidPartCount);
        };
        if *namespace != DDS_NAMESPACE {
            return Err(TypeNameError::MissingDdsNamespace);
        }
        let Some(name) = name.strip_suffix(RMW_NAME_SUFFIX) else {
            return Err(TypeNameError::MissingNameSuffix);
        };
        Self::new(package, interface, name)
    }

    fn new(package: &str, interface: &str, name: &str) -> Result<Self, TypeNameError> {
        if package.is_empty() || interface.is_empty() || name.is_empty() {
            return Err(TypeNameError::EmptyPart);
        }
        Ok(Self {
            package: package.to_string(),
            interface: interface.to_string(),
            name: name.to_string(),
        })
    }

    pub fn ros(&self) -> String {
        format!(
            "{}{ROS_SEPARATOR}{}{ROS_SEPARATOR}{}",
            self.package, self.interface, self.name
        )
    }

    pub fn rmw(&self) -> String {
        format!(
            "{}{RMW_SEPARATOR}{}{RMW_SEPARATOR}{DDS_NAMESPACE}{RMW_SEPARATOR}{}{RMW_NAME_SUFFIX}",
            self.package, self.interface, self.name
        )
    }

    pub fn request(&self) -> String {
        format!("{}{REQUEST_SUFFIX}", self.ros())
    }

    pub fn response(&self) -> String {
        format!("{}{RESPONSE_SUFFIX}", self.ros())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ROS_MESSAGE: &str = "std_msgs/msg/String";
    const RMW_MESSAGE: &str = "std_msgs::msg::dds_::String_";
    const ROS_SERVICE: &str = "example_interfaces/srv/AddTwoInts";
    const RMW_SERVICE: &str = "example_interfaces::srv::dds_::AddTwoInts_";

    #[test]
    fn ros_message_name_converts_to_rmw() {
        assert_eq!(
            TypeName::from_ros(ROS_MESSAGE).map(|t| t.rmw()),
            Ok(RMW_MESSAGE.to_string())
        );
    }

    #[test]
    fn rmw_message_name_converts_to_ros() {
        assert_eq!(
            TypeName::from_rmw(RMW_MESSAGE).map(|t| t.ros()),
            Ok(ROS_MESSAGE.to_string())
        );
    }

    #[test]
    fn ros_service_name_converts_to_rmw() {
        assert_eq!(
            TypeName::from_ros(ROS_SERVICE).map(|t| t.rmw()),
            Ok(RMW_SERVICE.to_string())
        );
    }

    #[test]
    fn rmw_service_name_converts_to_ros() {
        assert_eq!(
            TypeName::from_rmw(RMW_SERVICE).map(|t| t.ros()),
            Ok(ROS_SERVICE.to_string())
        );
    }

    #[test]
    fn service_names_its_request_and_response() {
        let service = TypeName::from_rmw(RMW_SERVICE);

        assert_eq!(
            service.as_ref().map(|t| t.request()),
            Ok("example_interfaces/srv/AddTwoInts_Request".to_string())
        );
        assert_eq!(
            service.as_ref().map(|t| t.response()),
            Ok("example_interfaces/srv/AddTwoInts_Response".to_string())
        );
    }

    #[test]
    fn ros_name_with_wrong_part_count_is_rejected() {
        for input in ["", "String", "std_msgs/String", "std_msgs/msg/String/x"] {
            assert_eq!(
                TypeName::from_ros(input),
                Err(TypeNameError::InvalidPartCount),
                "{input}"
            );
        }
    }

    #[test]
    fn ros_name_with_empty_part_is_rejected() {
        for input in ["/msg/String", "std_msgs//String", "std_msgs/msg/"] {
            assert_eq!(
                TypeName::from_ros(input),
                Err(TypeNameError::EmptyPart),
                "{input}"
            );
        }
    }

    #[test]
    fn rmw_name_with_wrong_part_count_is_rejected() {
        for input in [
            "",
            "std_msgs::msg::String_",
            "std_msgs::msg::dds_::String_::x",
            ROS_MESSAGE,
        ] {
            assert_eq!(
                TypeName::from_rmw(input),
                Err(TypeNameError::InvalidPartCount),
                "{input}"
            );
        }
    }

    #[test]
    fn rmw_name_without_dds_namespace_is_rejected() {
        assert_eq!(
            TypeName::from_rmw("std_msgs::msg::detail::String_"),
            Err(TypeNameError::MissingDdsNamespace)
        );
    }

    #[test]
    fn rmw_name_without_suffix_is_rejected() {
        assert_eq!(
            TypeName::from_rmw("std_msgs::msg::dds_::String"),
            Err(TypeNameError::MissingNameSuffix)
        );
    }

    #[test]
    fn rmw_name_with_empty_part_is_rejected() {
        for input in [
            "::msg::dds_::String_",
            "std_msgs::::dds_::String_",
            "std_msgs::msg::dds_::_",
        ] {
            assert_eq!(
                TypeName::from_rmw(input),
                Err(TypeNameError::EmptyPart),
                "{input}"
            );
        }
    }
}
