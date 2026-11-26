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
use ros_env::std_msgs::msg::rmw::MultiArrayDimension;

use crate::schemas::multi_array_dimension::std_msgs::msg as multi_array_dimension;
use crate::schemas::time::builtin_interfaces::msg as time;

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

pub fn empty_flatbuffer() -> Vec<u8> {
    let mut builder = FlatBufferBuilder::new();
    let table = builder.start_table();
    let root = builder.end_table(table);
    builder.finish_minimal(root);
    builder.finished_data().to_vec()
}
