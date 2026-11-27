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

use core::ops::{Deref, DerefMut};

use flatbuffers::{Allocator, FlatBufferBuilder};
use iceoryx2_integrations_ros2_link_adapter::TypeName;
use iceoryx2_link_adapter::{SampleBytesRef, TranscodeError, Transcoder};
use iceoryx2_link_backend::wire::region::{BackwardRegion, Region, ResizeError};
use iceoryx2_log::{fail, origin};

use crate::{Compiled, DecodeError, EncodeError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TranscodeFailure {
    Encode(EncodeError),
    Decode(DecodeError),
}

impl core::fmt::Display for TranscodeFailure {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "TranscodeFailure::{self:?}")
    }
}

impl core::error::Error for TranscodeFailure {}

/// Transcodes between a flatbuffer and the CDR of a ROS 2 message through a
/// pre-compiled model.
#[derive(Debug)]
pub struct FlatbufferTranscoder {
    pub(crate) type_name: TypeName,
    pub(crate) compiled: Compiled,
}

impl<'a> Transcoder<SampleBytesRef<'a>> for FlatbufferTranscoder {
    type Error = TranscodeFailure;

    fn encode<R: Region>(
        &self,
        local: SampleBytesRef<'a>,
        into: &mut R,
    ) -> Result<(), TranscodeError<Self::Error>> {
        let origin = origin!("FlatbufferTranscoder::encode");

        // The CDR is written into the payload from its front.
        match crate::encode(&self.compiled, local.payload, into.forward()) {
            Ok(()) => Ok(()),
            Err(EncodeError::Region(_)) => {
                fail!(
                    from origin,
                    with TranscodeError::Refused,
                    "The payload region rejected the CDR of a '{}'", self.type_name.as_str()
                );
            }
            Err(error) => {
                fail!(
                    from origin,
                    with TranscodeError::Transcoder(TranscodeFailure::Encode(error)),
                    "The payload of {} bytes does not encode as a '{}'", local.payload.len(), self.type_name.as_str()
                );
            }
        }
    }

    fn decode<R: Region>(
        &self,
        wire: SampleBytesRef<'a>,
        into: &mut R,
    ) -> Result<(), TranscodeError<Self::Error>> {
        let origin = origin!("FlatbufferTranscoder::decode");

        // The flatbuffer is built into the payload from its end.
        let mut region = into.backward();
        // Use the payload length as initial guess.
        fail!(
            from origin,
            when region.resize(wire.payload.len()),
            to TranscodeError<Self::Error>,
            "The payload region rejected the {} bytes to build a '{}' in", wire.payload.len(), self.type_name.as_str()
        );

        // Build the flatbuffer
        let mut builder = FlatBufferBuilder::new_in(RegionAllocator(region));
        if let Err(error) = crate::decode(&self.compiled, wire.payload, &mut builder) {
            fail!(
                from origin,
                with TranscodeError::Transcoder(TranscodeFailure::Decode(error)),
                "The {} wire bytes do not decode as a '{}'", wire.payload.len(), self.type_name.as_str()
            );
        }

        // The payload is cut down to the finished flatbuffer's size.
        let (RegionAllocator(mut region), start) = builder.collapse_in();
        let len = region.len() - start;
        fail!(
            from origin,
            when region.resize(len),
            to TranscodeError<Self::Error>,
            "The payload region rejected the {} bytes of a '{}'", len, self.type_name.as_str()
        );

        Ok(())
    }
}

/// Implementation of a flatbuffer allocator over a backwards-growing region.
struct RegionAllocator<B: BackwardRegion>(B);

impl<B: BackwardRegion> Deref for RegionAllocator<B> {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        &self.0
    }
}

impl<B: BackwardRegion> DerefMut for RegionAllocator<B> {
    fn deref_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

// SAFETY: a backward region keeps its bytes at the end when it is resized,
// which is where the builder expects them after a grow.
unsafe impl<B: BackwardRegion> Allocator for RegionAllocator<B> {
    type Error = ResizeError;

    fn grow_downwards(&mut self) -> Result<(), ResizeError> {
        let grown = core::cmp::max(1, 2 * self.0.len());
        self.0.resize(grown)
    }

    fn len(&self) -> usize {
        self.0.len()
    }
}
