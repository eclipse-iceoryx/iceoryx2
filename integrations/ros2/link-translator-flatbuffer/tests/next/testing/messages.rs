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

use ros_env::builtin_interfaces::msg::rmw::Time;
use ros_env::geometry_msgs::msg::rmw::{Point, Pose, PoseStamped, Quaternion};
use ros_env::std_msgs::msg::rmw::Header;

pub const HEADER: &str = "std_msgs/msg/Header";
pub const MULTI_ARRAY_DIMENSION: &str = "std_msgs/msg/MultiArrayDimension";
pub const POSE_STAMPED: &str = "geometry_msgs/msg/PoseStamped";
pub const TIME: &str = "builtin_interfaces/msg/Time";
pub const UINT8: &str = "std_msgs/msg/UInt8";
pub const WSTRING: &str = "example_interfaces/msg/WString";

pub fn pose_stamped_message() -> PoseStamped {
    PoseStamped {
        header: Header {
            stamp: Time {
                sec: 1_700_000_000,
                nanosec: 123_456_789,
            },
            frame_id: "base_link".into(),
        },
        pose: Pose {
            position: Point {
                x: 1.5,
                y: -2.5,
                z: 3.25,
            },
            orientation: Quaternion {
                x: 0.5,
                y: -0.5,
                z: 0.25,
                w: 0.75,
            },
        },
    }
}
