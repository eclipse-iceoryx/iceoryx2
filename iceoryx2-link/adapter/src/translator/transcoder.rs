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

use core::error::Error;

use core::convert::Infallible;

use crate::{Never, Region, UnsupportedLength};

/// Why a transcode ended without the region written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TranscodeError<TranscoderError> {
    /// The region refused the length the transcoder needs.
    Refused,
    /// The transcoder failed with its own error.
    Transcoder(TranscoderError),
}

impl<TranscoderError: core::fmt::Display> core::fmt::Display for TranscodeError<TranscoderError> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Refused => write!(f, "TranscodeError::Refused"),
            Self::Transcoder(error) => write!(f, "TranscodeError::Transcoder({error})"),
        }
    }
}

impl<TranscoderError: Error> Error for TranscodeError<TranscoderError> {}

impl<TranscoderError> From<UnsupportedLength> for TranscodeError<TranscoderError> {
    fn from(_: UnsupportedLength) -> Self {
        Self::Refused
    }
}

/// Transcodes the provided bytes into the provided regions.
pub trait Transcoder<BytesRef> {
    type Error: Error;

    /// Writes the middleware's form of the region into `into`.
    fn encode<R: Region>(
        &self,
        local: BytesRef,
        into: &mut R,
    ) -> Result<(), TranscodeError<Self::Error>>;

    /// Writes the local form of the region into `into`.
    fn decode<R: Region>(
        &self,
        wire: BytesRef,
        into: &mut R,
    ) -> Result<(), TranscodeError<Self::Error>>;
}

/// The transcoder of a region that passes through, never asked.
pub type NoTranscoder = Never;

impl<BytesRef> Transcoder<BytesRef> for NoTranscoder {
    type Error = Infallible;

    fn encode<R: Region>(&self, _: BytesRef, _: &mut R) -> Result<(), TranscodeError<Self::Error>> {
        match *self {}
    }

    fn decode<R: Region>(&self, _: BytesRef, _: &mut R) -> Result<(), TranscodeError<Self::Error>> {
        match *self {}
    }
}
