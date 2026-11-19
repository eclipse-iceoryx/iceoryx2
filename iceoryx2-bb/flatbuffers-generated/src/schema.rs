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

/// `namespace Benchmark;`
pub mod benchmark {
    /// Root type `Benchmark.PayloadData`.
    pub const PAYLOAD_DATA: &[u8] = include_bytes!("../schema/payload_data.bfbs");
}

/// `namespace Testing;`
pub mod testing {
    /// Root type `Testing.BoundedData`.
    pub const BOUNDED_DATA: &[u8] = include_bytes!("../schema/bounded_data.bfbs");
    /// Root type `Testing.IncompatibleData`.
    pub const INCOMPATIBLE_DATA: &[u8] = include_bytes!("../schema/incompatible_data.bfbs");
    /// Root type `Testing.UnboundedData`.
    pub const UNBOUNDED_DATA: &[u8] = include_bytes!("../schema/unbounded_data.bfbs");
    /// Root type `Testing.UnboundedData` (another layout).
    pub const UNBOUNDED_DATA_ALT: &[u8] = include_bytes!("../schema/unbounded_data_alt.bfbs");
    /// Root type `Testing.DataProps`.
    pub const DATA_PROPS: &[u8] = include_bytes!("../schema/data_props.bfbs");
    /// Declares no `root_type`.
    pub const NO_ROOT_TYPE: &[u8] = include_bytes!("../schema/no_root_type.bfbs");
}

/// Schemas corresponding to ROS 2 message types (a module per package).
pub mod ros2 {
    /// `namespace builtin_interfaces.msg;`
    pub mod builtin_interfaces {
        pub mod msg {
            /// Root type `builtin_interfaces.msg.Time`.
            pub const TIME_TABLE: &[u8] = include_bytes!("../schema/time_table.bfbs");
        }
    }

    /// `namespace geometry_msgs.msg;`
    pub mod geometry_msgs {
        pub mod msg {
            /// Root type `geometry_msgs.msg.PoseStamped`.
            pub const POSE_STAMPED: &[u8] = include_bytes!("../schema/pose_stamped.bfbs");
        }
    }

    /// `namespace sensor_msgs.msg;`
    pub mod sensor_msgs {
        pub mod msg {
            /// Root type `sensor_msgs.msg.JointState`.
            pub const JOINT_STATE: &[u8] = include_bytes!("../schema/joint_state.bfbs");
            /// Root type `sensor_msgs.msg.JointState`.
            pub const JOINT_STATE_FLOAT_POSITION: &[u8] =
                include_bytes!("../schema/joint_state_float_position.bfbs");
            /// Root type `sensor_msgs.msg.JointState`.
            pub const JOINT_STATE_WITHOUT_EFFORT: &[u8] =
                include_bytes!("../schema/joint_state_without_effort.bfbs");
        }
    }

    /// `namespace std_msgs.msg;`
    pub mod std_msgs {
        pub mod msg {
            /// Root type `std_msgs.msg.UInt8`.
            pub const UINT8_ENUM: &[u8] = include_bytes!("../schema/uint8_enum.bfbs");
        }
    }
}
