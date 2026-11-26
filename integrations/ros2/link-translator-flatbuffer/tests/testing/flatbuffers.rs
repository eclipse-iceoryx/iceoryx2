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

use flatbuffers::FlatBufferBuilder;
use ros_env::builtin_interfaces::msg::rmw::Time;
use ros_env::geometry_msgs::msg::rmw::{
    Pose, PoseArray, PoseStamped, PoseWithCovariance, PoseWithCovarianceStamped,
};
use ros_env::sensor_msgs::msg::rmw::{ChannelFloat32, JointState, PointCloud};
use ros_env::std_msgs::msg::rmw::{Header, MultiArrayDimension};

use crate::schemas::channel_float32::sensor_msgs::msg as channel_float32;
use crate::schemas::joint_state::sensor_msgs::msg as joint_state;
use crate::schemas::multi_array_dimension::std_msgs::msg as multi_array_dimension;
use crate::schemas::pose_array::geometry_msgs::msg as pose_array;
use crate::schemas::pose_array_struct::geometry_msgs::msg as pose_array_struct;
use crate::schemas::pose_with_covariance::geometry_msgs::msg as pose_with_covariance;
use crate::schemas::pose_with_covariance_stamped::geometry_msgs::msg as pose_with_covariance_stamped;
use crate::schemas::time::builtin_interfaces::msg as time;
use crate::schemas::{self, header, header_struct, point_cloud, pose_stamped};

pub trait ToFlatbuffer {
    fn to_flatbuffer(&self) -> Vec<u8>;
}

impl ToFlatbuffer for Time {
    fn to_flatbuffer(&self) -> Vec<u8> {
        let mut builder = FlatBufferBuilder::new();
        let root = time::Time::create(
            &mut builder,
            &time::TimeArgs {
                sec: self.sec,
                nanosec: self.nanosec,
            },
        );
        builder.finish_minimal(root);
        builder.finished_data().to_vec()
    }
}

impl ToFlatbuffer for MultiArrayDimension {
    fn to_flatbuffer(&self) -> Vec<u8> {
        let mut builder = FlatBufferBuilder::new();
        let label = builder.create_string(&self.label.to_string());
        let root = multi_array_dimension::MultiArrayDimension::create(
            &mut builder,
            &multi_array_dimension::MultiArrayDimensionArgs {
                stride: self.stride,
                label: Some(label),
                size: self.size,
            },
        );
        builder.finish_minimal(root);
        builder.finished_data().to_vec()
    }
}

impl ToFlatbuffer for Header {
    fn to_flatbuffer(&self) -> Vec<u8> {
        let mut builder = FlatBufferBuilder::new();
        let frame_id = builder.create_string(&self.frame_id.to_string());
        let stamp = header::builtin_interfaces::msg::Time::create(
            &mut builder,
            &header::builtin_interfaces::msg::TimeArgs {
                sec: self.stamp.sec,
                nanosec: self.stamp.nanosec,
            },
        );
        let root = header::std_msgs::msg::Header::create(
            &mut builder,
            &header::std_msgs::msg::HeaderArgs {
                stamp: Some(stamp),
                frame_id: Some(frame_id),
            },
        );
        builder.finish_minimal(root);
        builder.finished_data().to_vec()
    }
}

impl ToFlatbuffer for PoseStamped {
    fn to_flatbuffer(&self) -> Vec<u8> {
        let mut builder = FlatBufferBuilder::new();
        let frame_id = builder.create_string(&self.header.frame_id.to_string());
        let stamp = pose_stamped::builtin_interfaces::msg::Time::new(
            self.header.stamp.sec,
            self.header.stamp.nanosec,
        );
        let header = pose_stamped::std_msgs::msg::Header::create(
            &mut builder,
            &pose_stamped::std_msgs::msg::HeaderArgs {
                stamp: Some(&stamp),
                frame_id: Some(frame_id),
            },
        );
        let position = pose_stamped::geometry_msgs::msg::Point::new(
            self.pose.position.x,
            self.pose.position.y,
            self.pose.position.z,
        );
        let orientation = pose_stamped::geometry_msgs::msg::Quaternion::new(
            self.pose.orientation.x,
            self.pose.orientation.y,
            self.pose.orientation.z,
            self.pose.orientation.w,
        );
        let pose = pose_stamped::geometry_msgs::msg::Pose::new(&position, &orientation);
        let root = pose_stamped::geometry_msgs::msg::PoseStamped::create(
            &mut builder,
            &pose_stamped::geometry_msgs::msg::PoseStampedArgs {
                header: Some(header),
                pose: Some(&pose),
            },
        );
        builder.finish_minimal(root);
        builder.finished_data().to_vec()
    }
}

impl ToFlatbuffer for ChannelFloat32 {
    fn to_flatbuffer(&self) -> Vec<u8> {
        let mut builder = FlatBufferBuilder::new();
        let name = builder.create_string(&self.name.to_string());
        let values = builder.create_vector(self.values.as_slice());
        let root = channel_float32::ChannelFloat32::create(
            &mut builder,
            &channel_float32::ChannelFloat32Args {
                name: Some(name),
                values: Some(values),
            },
        );
        builder.finish_minimal(root);
        builder.finished_data().to_vec()
    }
}

impl ToFlatbuffer for PoseWithCovariance {
    fn to_flatbuffer(&self) -> Vec<u8> {
        pose_with_covariance_flatbuffer(&self.pose, &self.covariance)
    }
}

impl ToFlatbuffer for PoseWithCovarianceStamped {
    fn to_flatbuffer(&self) -> Vec<u8> {
        let mut builder = FlatBufferBuilder::new();
        let frame_id = builder.create_string(&self.header.frame_id.to_string());
        let stamp = schemas::pose_with_covariance_stamped::builtin_interfaces::msg::Time::new(
            self.header.stamp.sec,
            self.header.stamp.nanosec,
        );
        let header = schemas::pose_with_covariance_stamped::std_msgs::msg::Header::create(
            &mut builder,
            &schemas::pose_with_covariance_stamped::std_msgs::msg::HeaderArgs {
                stamp: Some(&stamp),
                frame_id: Some(frame_id),
            },
        );
        let position = pose_with_covariance_stamped::Point::new(
            self.pose.pose.position.x,
            self.pose.pose.position.y,
            self.pose.pose.position.z,
        );
        let orientation = pose_with_covariance_stamped::Quaternion::new(
            self.pose.pose.orientation.x,
            self.pose.pose.orientation.y,
            self.pose.pose.orientation.z,
            self.pose.pose.orientation.w,
        );
        let pose = pose_with_covariance_stamped::Pose::new(&position, &orientation);
        let pose =
            pose_with_covariance_stamped::PoseWithCovariance::new(&pose, &self.pose.covariance);
        let root = pose_with_covariance_stamped::PoseWithCovarianceStamped::create(
            &mut builder,
            &pose_with_covariance_stamped::PoseWithCovarianceStampedArgs {
                header: Some(header),
                pose: Some(&pose),
            },
        );
        builder.finish_minimal(root);
        builder.finished_data().to_vec()
    }
}

impl ToFlatbuffer for JointState {
    fn to_flatbuffer(&self) -> Vec<u8> {
        let mut builder = FlatBufferBuilder::new();
        let frame_id = builder.create_string(&self.header.frame_id.to_string());
        let stamp = schemas::joint_state::builtin_interfaces::msg::Time::new(
            self.header.stamp.sec,
            self.header.stamp.nanosec,
        );
        let header = schemas::joint_state::std_msgs::msg::Header::create(
            &mut builder,
            &schemas::joint_state::std_msgs::msg::HeaderArgs {
                stamp: Some(&stamp),
                frame_id: Some(frame_id),
            },
        );
        let names: Vec<_> = self
            .name
            .as_slice()
            .iter()
            .map(|name| builder.create_string(&name.to_string()))
            .collect();
        let name = builder.create_vector(&names);
        let position = builder.create_vector(self.position.as_slice());
        let velocity = builder.create_vector(self.velocity.as_slice());
        let effort = builder.create_vector(self.effort.as_slice());
        let root = joint_state::JointState::create(
            &mut builder,
            &joint_state::JointStateArgs {
                header: Some(header),
                name: Some(name),
                position: Some(position),
                velocity: Some(velocity),
                effort: Some(effort),
            },
        );
        builder.finish_minimal(root);
        builder.finished_data().to_vec()
    }
}

impl ToFlatbuffer for PoseArray {
    fn to_flatbuffer(&self) -> Vec<u8> {
        let mut builder = FlatBufferBuilder::new();
        let frame_id = builder.create_string(&self.header.frame_id.to_string());
        let stamp = schemas::pose_array::builtin_interfaces::msg::Time::new(
            self.header.stamp.sec,
            self.header.stamp.nanosec,
        );
        let header = schemas::pose_array::std_msgs::msg::Header::create(
            &mut builder,
            &schemas::pose_array::std_msgs::msg::HeaderArgs {
                stamp: Some(&stamp),
                frame_id: Some(frame_id),
            },
        );
        let poses: Vec<_> = self
            .poses
            .as_slice()
            .iter()
            .map(|pose| {
                let position =
                    pose_array::Point::new(pose.position.x, pose.position.y, pose.position.z);
                let orientation = pose_array::Quaternion::new(
                    pose.orientation.x,
                    pose.orientation.y,
                    pose.orientation.z,
                    pose.orientation.w,
                );
                pose_array::Pose::create(
                    &mut builder,
                    &pose_array::PoseArgs {
                        position: Some(&position),
                        orientation: Some(&orientation),
                    },
                )
            })
            .collect();
        let poses = builder.create_vector(&poses);
        let root = pose_array::PoseArray::create(
            &mut builder,
            &pose_array::PoseArrayArgs {
                header: Some(header),
                poses: Some(poses),
            },
        );
        builder.finish_minimal(root);
        builder.finished_data().to_vec()
    }
}

impl ToFlatbuffer for PointCloud {
    fn to_flatbuffer(&self) -> Vec<u8> {
        let mut builder = FlatBufferBuilder::new();
        let frame_id = builder.create_string(&self.header.frame_id.to_string());
        let stamp = point_cloud::builtin_interfaces::msg::Time::new(
            self.header.stamp.sec,
            self.header.stamp.nanosec,
        );
        let header = point_cloud::std_msgs::msg::Header::create(
            &mut builder,
            &point_cloud::std_msgs::msg::HeaderArgs {
                stamp: Some(&stamp),
                frame_id: Some(frame_id),
            },
        );
        let points: Vec<_> = self
            .points
            .as_slice()
            .iter()
            .map(|point| {
                point_cloud::geometry_msgs::msg::Point32::create(
                    &mut builder,
                    &point_cloud::geometry_msgs::msg::Point32Args {
                        x: point.x,
                        y: point.y,
                        z: point.z,
                    },
                )
            })
            .collect();
        let points = builder.create_vector(&points);
        let channels: Vec<_> = self
            .channels
            .as_slice()
            .iter()
            .map(|channel| {
                let name = builder.create_string(&channel.name.to_string());
                let values = builder.create_vector(channel.values.as_slice());
                point_cloud::sensor_msgs::msg::ChannelFloat32::create(
                    &mut builder,
                    &point_cloud::sensor_msgs::msg::ChannelFloat32Args {
                        name: Some(name),
                        values: Some(values),
                    },
                )
            })
            .collect();
        let channels = builder.create_vector(&channels);
        let root = point_cloud::sensor_msgs::msg::PointCloud::create(
            &mut builder,
            &point_cloud::sensor_msgs::msg::PointCloudArgs {
                header: Some(header),
                points: Some(points),
                channels: Some(channels),
            },
        );
        builder.finish_minimal(root);
        builder.finished_data().to_vec()
    }
}

pub fn pose_with_covariance_flatbuffer(pose: &Pose, covariance: &[f64]) -> Vec<u8> {
    let mut builder = FlatBufferBuilder::new();
    let covariance = builder.create_vector(covariance);
    let position =
        pose_with_covariance::Point::new(pose.position.x, pose.position.y, pose.position.z);
    let orientation = pose_with_covariance::Quaternion::new(
        pose.orientation.x,
        pose.orientation.y,
        pose.orientation.z,
        pose.orientation.w,
    );
    let pose = pose_with_covariance::Pose::new(&position, &orientation);
    let root = pose_with_covariance::PoseWithCovariance::create(
        &mut builder,
        &pose_with_covariance::PoseWithCovarianceArgs {
            pose: Some(&pose),
            covariance: Some(covariance),
        },
    );
    builder.finish_minimal(root);
    builder.finished_data().to_vec()
}

pub fn pose_array_flatbuffer_with_struct_poses(message: &PoseArray) -> Vec<u8> {
    let mut builder = FlatBufferBuilder::new();
    let frame_id = builder.create_string(&message.header.frame_id.to_string());
    let stamp = schemas::pose_array_struct::builtin_interfaces::msg::Time::new(
        message.header.stamp.sec,
        message.header.stamp.nanosec,
    );
    let header = schemas::pose_array_struct::std_msgs::msg::Header::create(
        &mut builder,
        &schemas::pose_array_struct::std_msgs::msg::HeaderArgs {
            stamp: Some(&stamp),
            frame_id: Some(frame_id),
        },
    );
    let poses: Vec<_> = message
        .poses
        .as_slice()
        .iter()
        .map(|pose| {
            let position =
                pose_array_struct::Point::new(pose.position.x, pose.position.y, pose.position.z);
            let orientation = pose_array_struct::Quaternion::new(
                pose.orientation.x,
                pose.orientation.y,
                pose.orientation.z,
                pose.orientation.w,
            );
            pose_array_struct::Pose::new(&position, &orientation)
        })
        .collect();
    let poses = builder.create_vector(&poses);
    let root = pose_array_struct::PoseArray::create(
        &mut builder,
        &pose_array_struct::PoseArrayArgs {
            header: Some(header),
            poses: Some(poses),
        },
    );
    builder.finish_minimal(root);
    builder.finished_data().to_vec()
}

pub fn header_flatbuffer_with_a_struct_stamp(message: &Header) -> Vec<u8> {
    let mut builder = FlatBufferBuilder::new();
    let frame_id = builder.create_string(&message.frame_id.to_string());
    let stamp =
        header_struct::builtin_interfaces::msg::Time::new(message.stamp.sec, message.stamp.nanosec);
    let root = header_struct::std_msgs::msg::Header::create(
        &mut builder,
        &header_struct::std_msgs::msg::HeaderArgs {
            stamp: Some(&stamp),
            frame_id: Some(frame_id),
        },
    );
    builder.finish_minimal(root);
    builder.finished_data().to_vec()
}

pub fn header_flatbuffer_without_a_stamp(message: &Header) -> Vec<u8> {
    let mut builder = FlatBufferBuilder::new();
    let frame_id = builder.create_string(&message.frame_id.to_string());
    let root = header::std_msgs::msg::Header::create(
        &mut builder,
        &header::std_msgs::msg::HeaderArgs {
            stamp: None,
            frame_id: Some(frame_id),
        },
    );
    builder.finish_minimal(root);
    builder.finished_data().to_vec()
}

pub fn empty_flatbuffer() -> Vec<u8> {
    let mut builder = FlatBufferBuilder::new();
    let table = builder.start_table();
    let root = builder.end_table(table);
    builder.finish_minimal(root);
    builder.finished_data().to_vec()
}
