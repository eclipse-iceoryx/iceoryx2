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

use cdr::{CdrLe, Infinite};
use flatbuffers::{Allocator, FlatBufferBuilder};
use iceoryx2_link_adapter::{SampleBytesRef, TranscodeError, Transcoder};
use iceoryx2_link_backend::wire::region::{BackwardRegion, ForwardRegion, Region};
use iceoryx2_log::{fail, origin};
use ros_env::builtin_interfaces::msg::Time;
use ros_env::sensor_msgs::msg::JointState;
use ros_env::std_msgs::msg::Header;

use super::{RegionAllocator, TranscodeFailure};
use crate::flatbuffer_translator::schemas::{JointReadings, JointReadingsArgs, Timestamp};

/// Converts between a [`JointReadings`] flatbuffer and the CDR of a
/// `sensor_msgs/msg/JointState`.
#[derive(Debug, Default, Clone, Copy)]
pub struct JointReadingsTranscoder;

impl<'a> Transcoder<SampleBytesRef<'a>> for JointReadingsTranscoder {
    type Error = TranscodeFailure;

    fn encode<R: Region>(
        &self,
        local: SampleBytesRef<'a>,
        into: &mut R,
    ) -> Result<(), TranscodeError<Self::Error>> {
        let origin = origin!("JointReadingsTranscoder::encode");

        let readings = fail!(
            from origin,
            when flatbuffers::root::<JointReadings>(local.payload),
            with TranscodeError::Transcoder(TranscodeFailure::InvalidFlatbuffer),
            "The payload of {} bytes is not a JointReadings flatbuffer", local.payload.len()
        );

        let message = to_joint_state(&readings);
        let size = cdr::calc_serialized_size(&message) as usize;
        let mut region = into.forward();
        fail!(
            from origin,
            when region.resize(size),
            to TranscodeError<Self::Error>,
            "The payload region rejected the {} CDR bytes of the JointState", size
        );
        fail!(
            from origin,
            when cdr::serialize_into::<_, _, _, CdrLe>(&mut *region, &message, Infinite),
            with TranscodeError::Transcoder(TranscodeFailure::Serialize),
            "Failed to serialize the JointState to CDR"
        );

        Ok(())
    }

    fn decode<R: Region>(
        &self,
        wire: SampleBytesRef<'a>,
        into: &mut R,
    ) -> Result<(), TranscodeError<Self::Error>> {
        let origin = origin!("JointReadingsTranscoder::decode");

        let message: JointState = fail!(
            from origin,
            when cdr::deserialize(wire.payload),
            with TranscodeError::Transcoder(TranscodeFailure::Deserialize),
            "The {} wire bytes are not the CDR of a JointState", wire.payload.len()
        );
        // The flatbuffer is built into the payload from its end, starting
        // with as many bytes as the message has on the wire.
        let mut region = into.backward();
        fail!(
            from origin,
            when region.resize(wire.payload.len()),
            to TranscodeError<Self::Error>,
            "The payload region rejected the {} bytes to build the JointReadings in", wire.payload.len()
        );
        let mut builder = FlatBufferBuilder::new_in(RegionAllocator(region));
        let root = to_joint_readings(&mut builder, &message);
        builder.finish(root, None);

        // The payload is cut down to the finished flatbuffer.
        let (RegionAllocator(mut region), start) = builder.collapse_in();
        let len = region.len() - start;
        fail!(
            from origin,
            when region.resize(len),
            to TranscodeError<Self::Error>,
            "The payload region rejected the {} bytes of the JointReadings", len
        );

        Ok(())
    }
}

/// The JointState carrying the values of `readings`.
fn to_joint_state(readings: &JointReadings<'_>) -> JointState {
    let captured_at = readings.captured_at();

    JointState {
        header: Header {
            stamp: Time {
                sec: captured_at.seconds(),
                nanosec: captured_at.nanoseconds(),
            },
            frame_id: readings.frame().to_string(),
        },
        name: readings.joints().iter().map(str::to_string).collect(),
        position: readings.angles().iter().collect(),
        velocity: readings.speeds().iter().collect(),
        effort: readings.torques().iter().collect(),
    }
}

/// Builds the JointReadings carrying the values of `message` in the provided
/// flatbuffer `builder`.
fn to_joint_readings<'a, A: Allocator + 'a>(
    builder: &mut FlatBufferBuilder<'a, A>,
    message: &JointState,
) -> flatbuffers::WIPOffset<JointReadings<'a>> {
    let captured_at = Timestamp::new(message.header.stamp.sec, message.header.stamp.nanosec);
    let frame = builder.create_string(&message.header.frame_id);
    let names: Vec<flatbuffers::WIPOffset<&str>> = message
        .name
        .iter()
        .map(|name| builder.create_string(name))
        .collect();
    let joints = builder.create_vector(&names);
    let angles = builder.create_vector(&message.position);
    let speeds = builder.create_vector(&message.velocity);
    let torques = builder.create_vector(&message.effort);

    JointReadings::create(
        builder,
        &JointReadingsArgs {
            captured_at: Some(&captured_at),
            frame: Some(frame),
            joints: Some(joints),
            angles: Some(angles),
            speeds: Some(speeds),
            torques: Some(torques),
        },
    )
}
