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
use ros_env::geometry_msgs::msg::rmw::{
    Point, Point32, Pose, PoseArray, PoseStamped, PoseWithCovariance, PoseWithCovarianceStamped,
    Quaternion,
};
use ros_env::sensor_msgs::msg::rmw::{ChannelFloat32, JointState, PointCloud};
use ros_env::std_msgs::msg::rmw::Header;
use ros_env::test_msgs::msg::rmw::{Arrays, Defaults};

pub const ARRAYS: &str = "test_msgs/msg/Arrays";
pub const BOUNDED_SEQUENCES: &str = "test_msgs/msg/BoundedSequences";
pub const CHANNEL_FLOAT32: &str = "sensor_msgs/msg/ChannelFloat32";
pub const HEADER: &str = "std_msgs/msg/Header";
pub const JOINT_STATE: &str = "sensor_msgs/msg/JointState";
pub const MULTI_ARRAY_DIMENSION: &str = "std_msgs/msg/MultiArrayDimension";
pub const POINT_CLOUD: &str = "sensor_msgs/msg/PointCloud";
pub const POSE_ARRAY: &str = "geometry_msgs/msg/PoseArray";
pub const POSE_STAMPED: &str = "geometry_msgs/msg/PoseStamped";
pub const POSE_WITH_COVARIANCE: &str = "geometry_msgs/msg/PoseWithCovariance";
pub const POSE_WITH_COVARIANCE_STAMPED: &str = "geometry_msgs/msg/PoseWithCovarianceStamped";
pub const TIME: &str = "builtin_interfaces/msg/Time";
pub const UINT8: &str = "std_msgs/msg/UInt8";
pub const WSTRING: &str = "example_interfaces/msg/WString";

pub fn pose_with_covariance_stamped_message() -> PoseWithCovarianceStamped {
    PoseWithCovarianceStamped {
        header: pose_stamped_message().header,
        pose: pose_with_covariance_message(),
    }
}

pub fn pose_with_covariance_message() -> PoseWithCovariance {
    PoseWithCovariance {
        pose: pose_stamped_message().pose,
        covariance: core::array::from_fn(|index| index as f64 * 0.5),
    }
}

pub fn joint_state_message() -> JointState {
    JointState {
        header: pose_stamped_message().header,
        name: vec!["shoulder".into(), "elbow".into(), "wrist_1".into()].into(),
        position: vec![0.1, -0.2, 0.3].into(),
        velocity: vec![].into(),
        effort: vec![1.0, 2.0, 3.0].into(),
    }
}

pub fn pose_array_message() -> PoseArray {
    let first = pose_stamped_message().pose;
    let second = Pose {
        position: Point {
            x: -4.0,
            y: 5.5,
            z: 6.25,
        },
        orientation: Quaternion {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 1.0,
        },
    };
    PoseArray {
        header: pose_stamped_message().header,
        poses: vec![first, second].into(),
    }
}

pub fn point_cloud_message() -> PointCloud {
    PointCloud {
        header: pose_stamped_message().header,
        points: vec![
            Point32 {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            },
            Point32 {
                x: -1.5,
                y: -2.5,
                z: -3.5,
            },
        ]
        .into(),
        channels: vec![
            ChannelFloat32 {
                name: "intensity".into(),
                values: vec![0.5, 0.75].into(),
            },
            ChannelFloat32 {
                name: "rgb".into(),
                values: vec![1.0, 2.0].into(),
            },
        ]
        .into(),
    }
}

pub fn arrays_message() -> Arrays {
    let mut message = Arrays {
        string_values: ["first".into(), "".into(), "third".into()],
        alignment_check: 42,
        ..Default::default()
    };
    message.basic_types_values[1].int32_value = -7;
    message.basic_types_values[2].float64_value = 2.5;
    message
}

pub fn arrays_message_of_zeros() -> Arrays {
    Arrays {
        defaults_values: core::array::from_fn(|_| Defaults {
            bool_value: Default::default(),
            byte_value: Default::default(),
            char_value: Default::default(),
            float32_value: Default::default(),
            float64_value: Default::default(),
            int8_value: Default::default(),
            uint8_value: Default::default(),
            int16_value: Default::default(),
            uint16_value: Default::default(),
            int32_value: Default::default(),
            uint32_value: Default::default(),
            int64_value: Default::default(),
            uint64_value: Default::default(),
        }),
        bool_values_default: Default::default(),
        byte_values_default: Default::default(),
        char_values_default: Default::default(),
        float32_values_default: Default::default(),
        float64_values_default: Default::default(),
        int8_values_default: Default::default(),
        uint8_values_default: Default::default(),
        int16_values_default: Default::default(),
        uint16_values_default: Default::default(),
        int32_values_default: Default::default(),
        uint32_values_default: Default::default(),
        int64_values_default: Default::default(),
        uint64_values_default: Default::default(),
        string_values_default: Default::default(),
        ..Default::default()
    }
}

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
