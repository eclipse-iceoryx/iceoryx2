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

use std::path::PathBuf;

use iceoryx2::prelude::{Flatbuffer, Path};
use iceoryx2::service::static_config::message_type_details::{TypeDetail, TypeVariant};
use iceoryx2_bb_flatbuffers_generated::schema::NO_NAMESPACE;
use iceoryx2_bb_flatbuffers_generated::schema::ros2::geometry_msgs::msg::{
    POLYGON, POSE_STAMPED, POSE_WITH_COVARIANCE,
};
use iceoryx2_bb_flatbuffers_generated::schema::ros2::sensor_msgs::msg::JOINT_STATE;
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing_macros::test;
use iceoryx2_integrations_ros2_link_adapter::{MessageType, MirroredHeader, TypeName};
use iceoryx2_integrations_ros2_link_translator_flatbuffer::{
    FlatbufferTranslator, TranslationError,
};
use iceoryx2_link_adapter::{SampleBytesRef, SampleTranscoders, Transcoder, Translator};
use iceoryx2_link_backend::service_description::{
    SampleTypes, Schema, TypeDescription, TypeIdentifier,
};
use ros_env::builtin_interfaces::msg::rmw::Time;
use ros_env::geometry_msgs::msg::rmw::{
    Point, Point32, Polygon, Pose, PoseWithCovariance, Quaternion,
};
use ros_env::sensor_msgs::msg::rmw::JointState;
use ros_env::std_msgs::msg::rmw::Header;
use rosidl_runtime_rs::RmwMessage;

use crate::testing::serialize;

const JOINT_STATE_TYPE: &str = "sensor_msgs/msg/JointState";
const POSE_STAMPED_TYPE: &str = "geometry_msgs/msg/PoseStamped";
const POSE_WITH_COVARIANCE_TYPE: &str = "geometry_msgs/msg/PoseWithCovariance";
const POLYGON_TYPE: &str = "geometry_msgs/msg/Polygon";

/// A directory of schema files, removed when dropped.
struct SchemaDirectory(PathBuf);

impl SchemaDirectory {
    /// The directory of the test `test`, holding `files`.
    fn with(test: &str, files: &[(&str, &[u8])]) -> Self {
        let directory = std::env::temp_dir().join(format!(
            "iceoryx2_flatbuffer_translator_{}_{test}",
            std::process::id()
        ));
        std::fs::create_dir_all(&directory).expect("creates the directory");
        for (name, bytes) in files {
            std::fs::write(directory.join(name), bytes).expect("writes the schema");
        }
        Self(directory)
    }

    fn path(&self) -> Path {
        Path::new_normalized(self.0.to_str().expect("is UTF-8").as_bytes()).expect("is a path")
    }
}

impl Drop for SchemaDirectory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn translator(schema_path: Option<Path>) -> FlatbufferTranslator {
    FlatbufferTranslator {
        header: MirroredHeader::None,
        schema_path,
    }
}

fn message_type(type_name: &str) -> MessageType {
    MessageType {
        type_name: TypeName::new(type_name).expect("is a ROS 2 type name"),
    }
}

fn flatbuffer_types(schema: &[u8]) -> SampleTypes {
    let flatbuffer = TypeDetail::new::<Flatbuffer<()>>(TypeVariant::FixedSize);
    SampleTypes {
        payload: TypeDescription {
            variant: flatbuffer.variant(),
            identifier: TypeIdentifier::Flatbuffer(
                Schema::new(schema.to_vec()).expect("is a schema"),
            ),
            size: flatbuffer.size(),
            alignment: flatbuffer.alignment(),
        },
        user_header: MirroredHeader::None.type_description(),
    }
}

fn named_types(name: &str) -> SampleTypes {
    SampleTypes {
        payload: TypeDescription {
            variant: TypeVariant::FixedSize,
            identifier: TypeIdentifier::Name(name.to_string()),
            size: 8,
            alignment: 8,
        },
        user_header: MirroredHeader::None.type_description(),
    }
}

#[test]
fn the_remote_type_is_the_root_type_with_slashes() {
    let translator = translator(None);

    let remote = translator
        .remote(&flatbuffer_types(JOINT_STATE))
        .expect("translates");

    assert_that!(remote, eq message_type(JOINT_STATE_TYPE));
}

#[test]
fn a_payload_that_is_no_flatbuffer_has_no_remote_type() {
    let translator = translator(None);

    let error = translator
        .remote(&named_types("some/struct"))
        .expect_err("refuses");

    assert_that!(error, eq TranslationError::NotAFlatbuffer);
}

#[test]
fn a_root_type_that_names_no_ros2_type_has_no_remote_type() {
    let translator = translator(None);

    let error = translator
        .remote(&flatbuffer_types(NO_NAMESPACE))
        .expect_err("refuses");

    assert_that!(error, eq TranslationError::InvalidRootType);
}

#[test]
fn the_local_types_carry_the_schema_found_for_the_remote_type() {
    let schemas = SchemaDirectory::with(
        "the_local_types_carry_the_schema_found_for_the_remote_type",
        &[("JointState.bfbs", JOINT_STATE)],
    );
    let translator = translator(Some(schemas.path()));

    let local = translator
        .local(&message_type(JOINT_STATE_TYPE))
        .expect("translates");

    assert_that!(local, eq flatbuffer_types(JOINT_STATE));
}

#[test]
fn the_schema_is_found_by_its_snake_case_name() {
    let schemas = SchemaDirectory::with(
        "the_schema_is_found_by_its_snake_case_name",
        &[("joint_state.bfbs", JOINT_STATE)],
    );
    let translator = translator(Some(schemas.path()));

    let local = translator
        .local(&message_type(JOINT_STATE_TYPE))
        .expect("translates");

    assert_that!(local, eq flatbuffer_types(JOINT_STATE));
}

#[test]
fn a_remote_type_has_no_local_types_without_a_schema_path() {
    let translator = translator(None);

    let error = translator
        .local(&message_type(JOINT_STATE_TYPE))
        .expect_err("refuses");

    assert_that!(error, eq TranslationError::NoSchemaPath);
}

#[test]
fn a_remote_type_without_a_schema_has_no_local_types() {
    let schemas = SchemaDirectory::with(
        "a_remote_type_without_a_schema_has_no_local_types",
        &[("PoseStamped.bfbs", POSE_STAMPED)],
    );
    let translator = translator(Some(schemas.path()));

    let error = translator
        .local(&message_type(JOINT_STATE_TYPE))
        .expect_err("refuses");

    assert_that!(error, eq TranslationError::SchemaNotFound);
}

#[test]
fn a_schema_file_with_another_root_type_is_refused() {
    let schemas = SchemaDirectory::with(
        "a_schema_file_with_another_root_type_is_refused",
        &[("JointState.bfbs", POSE_STAMPED)],
    );
    let translator = translator(Some(schemas.path()));

    let error = translator
        .local(&message_type(JOINT_STATE_TYPE))
        .expect_err("refuses");

    assert_that!(error, eq TranslationError::Mismatch);
}

#[test]
fn transcoders_refuse_a_remote_type_other_than_the_root_type() {
    let translator = translator(None);

    let error = translator
        .transcoders(
            &flatbuffer_types(JOINT_STATE),
            &message_type(POSE_STAMPED_TYPE),
        )
        .expect_err("refuses");

    assert_that!(error, eq TranslationError::Mismatch);
}

/// The rmw bytes of `message` after decoding them into a flatbuffer and
/// encoding that back, through the payload transcoder of `schema`.
fn round_trip<M: RmwMessage>(schema: &[u8], type_name: &str, message: &M) -> (Vec<u8>, Vec<u8>) {
    let translator = translator(None);
    let rmw_bytes = serialize(message);

    let transcoders = translator
        .transcoders(&flatbuffer_types(schema), &message_type(type_name))
        .expect("translates");
    let SampleTranscoders::TranscodeBoth(_, payload) = transcoders else {
        panic!("a service without a header transcodes both");
    };

    let mut flatbuffer = Vec::new();
    payload
        .decode(
            SampleBytesRef {
                header: &[],
                payload: &rmw_bytes,
            },
            &mut flatbuffer,
        )
        .expect("decodes");
    let mut encoded_bytes = Vec::new();
    payload
        .encode(
            SampleBytesRef {
                header: &[],
                payload: &flatbuffer,
            },
            &mut encoded_bytes,
        )
        .expect("encodes");

    // The rmw reports its size estimate as the length and zero-fills past
    // the message.
    assert_that!(encoded_bytes.len(), le rmw_bytes.len());
    encoded_bytes.resize(rmw_bytes.len(), 0);
    (rmw_bytes, encoded_bytes)
}

fn pose() -> Pose {
    Pose {
        position: Point {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        },
        orientation: Quaternion {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 1.0,
        },
    }
}

#[test]
fn transcoders_carry_a_message_through_both_directions() {
    let message = JointState {
        header: Header {
            stamp: Time {
                sec: 1_700_000_000,
                nanosec: 123_456_789,
            },
            frame_id: "base_link".into(),
        },
        name: vec!["shoulder".into(), "elbow".into()].into(),
        position: vec![0.1, -0.2].into(),
        velocity: vec![].into(),
        effort: vec![1.0, 2.0].into(),
    };

    let (rmw_bytes, encoded_bytes) = round_trip(JOINT_STATE, JOINT_STATE_TYPE, &message);

    assert_that!(encoded_bytes, eq rmw_bytes);
}

#[test]
fn transcoders_carry_a_fixed_array_through_both_directions() {
    let message = PoseWithCovariance {
        pose: pose(),
        covariance: core::array::from_fn(|index| index as f64 * 0.5),
    };

    let (rmw_bytes, encoded_bytes) =
        round_trip(POSE_WITH_COVARIANCE, POSE_WITH_COVARIANCE_TYPE, &message);

    assert_that!(encoded_bytes, eq rmw_bytes);
}

#[test]
fn transcoders_carry_a_sequence_of_structs_through_both_directions() {
    let message = Polygon {
        points: vec![
            Point32 {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            },
            Point32 {
                x: 4.0,
                y: 5.0,
                z: 6.0,
            },
        ]
        .into(),
    };

    let (rmw_bytes, encoded_bytes) = round_trip(POLYGON, POLYGON_TYPE, &message);

    assert_that!(encoded_bytes, eq rmw_bytes);
}
