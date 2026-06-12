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

//! Mapping between iceoryx2 service names and ROS 2 topic names.

const TOPIC_PREFIX: &str = "ros2://topics";

/// Convert a conventional iceoryx2 service name into a ROS 2 topic name.
pub fn topic(service_name: &str) -> Option<&str> {
    let topic = service_name.strip_prefix(TOPIC_PREFIX)?;
    if !topic.starts_with('/') || topic.len() == 1 {
        return None;
    }
    Some(topic)
}

/// Convert a ROS 2 topic name into a conventional iceoryx2 service name.
pub fn service_name(topic: &str) -> String {
    format!("{TOPIC_PREFIX}{topic}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn topic_extracts_from_ros_service_names() {
        assert_eq!(topic("ros2://topics/chatter"), Some("/chatter"));
        assert_eq!(
            topic("ros2://topics/Camera/FrontRight"),
            Some("/Camera/FrontRight")
        );
    }

    #[test]
    fn topic_rejects_non_ros_service_names() {
        for service in [
            "",
            "My/Funk/ServiceName",
            "ros2://topics",
            "ros2://topics/",
            "iox2://something",
            "/chatter",
        ] {
            assert_eq!(topic(service), None, "{service}");
        }
    }

    #[test]
    fn service_name_round_trips() {
        let name = service_name("/Camera/FrontRight");
        assert_eq!(name, "ros2://topics/Camera/FrontRight");
        assert_eq!(topic(&name), Some("/Camera/FrontRight"));
    }
}
