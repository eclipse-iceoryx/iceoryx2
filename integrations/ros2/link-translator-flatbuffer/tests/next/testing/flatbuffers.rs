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
use ros_env::geometry_msgs::msg::rmw::PoseStamped;
use ros_env::std_msgs::msg::rmw::{Header, MultiArrayDimension};

use crate::schemas::multi_array_dimension::std_msgs::msg as multi_array_dimension;
use crate::schemas::time::builtin_interfaces::msg as time;
use crate::schemas::{header, header_struct, pose_stamped};

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
