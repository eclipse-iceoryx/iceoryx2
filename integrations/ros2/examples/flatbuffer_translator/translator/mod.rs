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

use iceoryx2::service::marker::Flatbuffer;
use iceoryx2::service::static_config::message_type_details::{TypeDetail, TypeVariant};
use iceoryx2_integrations_ros2_link_adapter::translator::{
    EmptyHeader, TranslationError as HeaderError,
};
use iceoryx2_integrations_ros2_link_adapter::{MirroredHeader, TopicTypes, TypeName};
use iceoryx2_link_adapter::{
    LocalTypes, SampleBytesRef, SampleShape, SampleTranscoders, TranscodeError, Transcoder,
    Translator,
};
use iceoryx2_link_backend::service_description::{
    SampleTypes, Schema, TypeDescription, TypeIdentifier,
};
use iceoryx2_link_backend::wire::region::Region;
use iceoryx2_log::{fail, origin};

pub mod transcoders;

use transcoders::{JointReadingsTranscoder, PoseTranscoder, TranscodeFailure};

use crate::flatbuffer_translator::schemas::{JOINT_READINGS_SCHEMA, POSE_SCHEMA};

/// The ROS 2 type the [`JointReadings`](crate::flatbuffer_translator::schemas::JointReadings)
/// flatbuffer translates to.
pub const JOINT_STATE: &str = "sensor_msgs/msg/JointState";

/// The ROS 2 type the [`Pose`](crate::flatbuffer_translator::schemas::Pose) flatbuffer translates to.
pub const POSE_STAMPED: &str = "geometry_msgs/msg/PoseStamped";

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum TranslationError {
    /// The topic's type is none of the two the translator pairs.
    UnknownType,
    /// The local payload is not a flatbuffer.
    NotAFlatbuffer,
    /// The local payload's schema is none of the two the translator pairs.
    UnknownSchema,
    /// The local schema and the topic's type are not a pair.
    Mismatch,
    /// The local user header is neither the RosHeader nor absent.
    Header(HeaderError),
}

impl core::fmt::Display for TranslationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "TranslationError::{self:?}")
    }
}

impl core::error::Error for TranslationError {}

impl From<HeaderError> for TranslationError {
    fn from(error: HeaderError) -> Self {
        Self::Header(error)
    }
}

/// Translator between two example flatbuffer schemas and corresponding ROS 2
/// messages.
#[derive(Debug, Default, Clone, Copy)]
pub struct FlatbufferTranslator {
    /// The user header to use for the services created to mirror topics.
    pub header: MirroredHeader,
}

impl Translator<SampleShape> for FlatbufferTranslator {
    type RemoteTypes = TopicTypes;
    /// Specify the transcoder for both the user header and the payload.
    type Transcoders = SampleTranscoders<EmptyHeader, FlatbufferTranscoder>;
    type Error = TranslationError;

    /// The local types corresponding to a topic's type.
    fn local(&self, remote: &TopicTypes) -> Result<LocalTypes<SampleShape>, TranslationError> {
        let origin = origin!("FlatbufferTranslator::local");

        // Map a topic type to a schema.
        let schema = match remote.type_name.as_str() {
            JOINT_STATE => JOINT_READINGS_SCHEMA,
            POSE_STAMPED => POSE_SCHEMA,
            other => {
                fail!(
                    from origin,
                    with TranslationError::UnknownType,
                    "Topic type '{}' has no flatbuffer the translator pairs it with", other
                );
            }
        };

        Ok(SampleTypes {
            payload: payload_type(schema),
            user_header: self.header.type_description(),
        })
    }

    /// The topic type corresponding to a service's local types.
    fn remote(&self, local: &LocalTypes<SampleShape>) -> Result<TopicTypes, TranslationError> {
        let origin = origin!("FlatbufferTranslator::remote");

        // Only flatbuffer payloads are translated.
        let schema = fail!(
            from origin,
            when flatbuffer_schema(local),
            "Payload '{}' is not translated", local.payload.identifier.type_name()
        );

        // Map a schema to a topic type.
        let type_name = if schema.bytes() == JOINT_READINGS_SCHEMA {
            JOINT_STATE
        } else if schema.bytes() == POSE_SCHEMA {
            POSE_STAMPED
        } else {
            fail!(
                from origin,
                with TranslationError::UnknownSchema,
                "The payload's schema of {} bytes is none the translator pairs", schema.bytes().len()
            );
        };

        Ok(TopicTypes {
            type_name: TypeName::new(type_name).expect("the paired type names are valid"),
        })
    }

    /// Provides the transcoders to use for the given pair of service and
    /// topic types.
    fn transcoders(
        &self,
        local: &LocalTypes<SampleShape>,
        remote: &TopicTypes,
    ) -> Result<Self::Transcoders, TranslationError> {
        let origin = origin!("FlatbufferTranslator::transcoders");

        let schema = fail!(
            from origin,
            when flatbuffer_schema(local),
            "Payload '{}' is not translated", local.payload.identifier.type_name()
        );

        // Determine which transcoder to use for the payload based on the
        // topic name and whether the schema type used by the service is
        // correct.
        let payload_transcoder = match remote.type_name.as_str() {
            JOINT_STATE if schema.bytes() == JOINT_READINGS_SCHEMA => {
                FlatbufferTranscoder::JointReadings(JointReadingsTranscoder)
            }
            POSE_STAMPED if schema.bytes() == POSE_SCHEMA => {
                FlatbufferTranscoder::Pose(PoseTranscoder)
            }
            other => {
                fail!(
                    from origin,
                    with TranslationError::Mismatch,
                    "The payload's schema of {} bytes is not paired with topic type '{}'",
                    schema.bytes().len(), other
                );
            }
        };

        //
        let header = fail!(
            from origin,
            when MirroredHeader::of(local),
            to TranslationError,
            "Header '{}' is not the RosHeader, ROS 2 cannot fill it", local.user_header.identifier.type_name()
        );

        Ok(match header {
            MirroredHeader::RosHeader => SampleTranscoders::TranscodePayload(payload_transcoder),
            MirroredHeader::None => {
                SampleTranscoders::TranscodeBoth(EmptyHeader, payload_transcoder)
            }
        })
    }
}

/// The available flatbuffer transcoders.
#[derive(Debug, Clone, Copy)]
pub enum FlatbufferTranscoder {
    JointReadings(JointReadingsTranscoder),
    Pose(PoseTranscoder),
}

impl<'a> Transcoder<SampleBytesRef<'a>> for FlatbufferTranscoder {
    type Error = TranscodeFailure;

    fn encode<R: Region>(
        &self,
        local: SampleBytesRef<'a>,
        into: &mut R,
    ) -> Result<(), TranscodeError<Self::Error>> {
        match self {
            FlatbufferTranscoder::JointReadings(transcoder) => transcoder.encode(local, into),
            FlatbufferTranscoder::Pose(transcoder) => transcoder.encode(local, into),
        }
    }

    fn decode<R: Region>(
        &self,
        wire: SampleBytesRef<'a>,
        into: &mut R,
    ) -> Result<(), TranscodeError<Self::Error>> {
        match self {
            FlatbufferTranscoder::JointReadings(transcoder) => transcoder.decode(wire, into),
            FlatbufferTranscoder::Pose(transcoder) => transcoder.decode(wire, into),
        }
    }
}

/// The description of a flatbuffer payload of `schema`.
fn payload_type(schema: &[u8]) -> TypeDescription {
    let flatbuffer = TypeDetail::new::<Flatbuffer<()>>(TypeVariant::FixedSize);
    TypeDescription {
        variant: flatbuffer.variant(),
        identifier: TypeIdentifier::Flatbuffer(Schema::new(schema.to_vec())),
        size: flatbuffer.size(),
        alignment: flatbuffer.alignment(),
    }
}

/// The schema of the flatbuffer payload of `types`.
fn flatbuffer_schema(types: &SampleTypes) -> Result<&Schema, TranslationError> {
    let origin = origin!("flatbuffer_schema");

    match &types.payload.identifier {
        TypeIdentifier::Flatbuffer(schema) => Ok(schema),
        TypeIdentifier::Name(name) => {
            fail!(
                from origin,
                with TranslationError::NotAFlatbuffer,
                "Payload '{}' is not a flatbuffer", name
            );
        }
    }
}
