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
use ros_env::geometry_msgs::msg::{Point, PoseStamped, Quaternion};
use ros_env::std_msgs::msg::Header;

use super::{RegionAllocator, TranscodeFailure};
use crate::flatbuffer_translator::schemas::{Pose, PoseArgs, Quat, Vec3};

/// Converts between a [`Pose`] flatbuffer and the CDR of a
/// `geometry_msgs/msg/PoseStamped`.
#[derive(Debug, Default, Clone, Copy)]
pub struct PoseTranscoder;

impl<'a> Transcoder<SampleBytesRef<'a>> for PoseTranscoder {
    type Error = TranscodeFailure;

    fn encode<R: Region>(
        &self,
        local: SampleBytesRef<'a>,
        into: &mut R,
    ) -> Result<(), TranscodeError<Self::Error>> {
        let origin = origin!("PoseTranscoder::encode");

        let pose = fail!(
            from origin,
            when flatbuffers::root::<Pose>(local.payload),
            with TranscodeError::Transcoder(TranscodeFailure::InvalidFlatbuffer),
            "The payload of {} bytes is not a Pose flatbuffer", local.payload.len()
        );

        let message = to_pose_stamped(&pose);
        let size = cdr::calc_serialized_size(&message) as usize;
        let mut region = into.forward();
        fail!(
            from origin,
            when region.resize(size),
            to TranscodeError<Self::Error>,
            "The payload region rejected the {} CDR bytes of the PoseStamped", size
        );
        fail!(
            from origin,
            when cdr::serialize_into::<_, _, _, CdrLe>(&mut *region, &message, Infinite),
            with TranscodeError::Transcoder(TranscodeFailure::Serialize),
            "Failed to serialize the PoseStamped to CDR"
        );

        Ok(())
    }

    fn decode<R: Region>(
        &self,
        wire: SampleBytesRef<'a>,
        into: &mut R,
    ) -> Result<(), TranscodeError<Self::Error>> {
        let origin = origin!("PoseTranscoder::decode");

        let message: PoseStamped = fail!(
            from origin,
            when cdr::deserialize(wire.payload),
            with TranscodeError::Transcoder(TranscodeFailure::Deserialize),
            "The {} wire bytes are not the CDR of a PoseStamped", wire.payload.len()
        );

        // The flatbuffer is built into the payload from its end, starting
        // with as many bytes as the message has on the wire.
        let mut region = into.backward();
        fail!(
            from origin,
            when region.resize(wire.payload.len()),
            to TranscodeError<Self::Error>,
            "The payload region rejected the {} bytes to build the Pose in", wire.payload.len()
        );
        let mut builder = FlatBufferBuilder::new_in(RegionAllocator(region));
        let root = to_pose(&mut builder, &message);
        builder.finish(root, None);

        // The payload is cut down to the finished flatbuffer.
        let (RegionAllocator(mut region), start) = builder.collapse_in();
        let len = region.len() - start;
        fail!(
            from origin,
            when region.resize(len),
            to TranscodeError<Self::Error>,
            "The payload region rejected the {} bytes of the Pose", len
        );

        Ok(())
    }
}

/// The PoseStamped carrying the values of `pose`.
fn to_pose_stamped(pose: &Pose<'_>) -> PoseStamped {
    let position = pose.position();
    let rotation = pose.rotation();

    PoseStamped {
        header: Header {
            stamp: Time {
                sec: pose.seconds(),
                nanosec: pose.nanoseconds(),
            },
            frame_id: pose.frame().to_string(),
        },
        pose: ros_env::geometry_msgs::msg::Pose {
            position: Point {
                x: position.x(),
                y: position.y(),
                z: position.z(),
            },
            orientation: Quaternion {
                x: rotation.x(),
                y: rotation.y(),
                z: rotation.z(),
                w: rotation.w(),
            },
        },
    }
}

/// Builds the Pose carrying the values of `message` in `builder`.
fn to_pose<'a, A: Allocator + 'a>(
    builder: &mut FlatBufferBuilder<'a, A>,
    message: &PoseStamped,
) -> flatbuffers::WIPOffset<Pose<'a>> {
    let frame = builder.create_string(&message.header.frame_id);
    let position = Vec3::new(
        message.pose.position.x,
        message.pose.position.y,
        message.pose.position.z,
    );
    let rotation = Quat::new(
        message.pose.orientation.x,
        message.pose.orientation.y,
        message.pose.orientation.z,
        message.pose.orientation.w,
    );
    Pose::create(
        builder,
        &PoseArgs {
            seconds: message.header.stamp.sec,
            nanoseconds: message.header.stamp.nanosec,
            frame: Some(frame),
            position: Some(&position),
            rotation: Some(&rotation),
        },
    )
}
