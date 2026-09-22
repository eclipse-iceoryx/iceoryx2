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

use crate::{LoanError, LoanableSample, Region, UnsupportedLength, WritableSample};

/// Why a transcode ended without the region written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TranscodeError<TranscoderError> {
    /// The length the transcoder needs does not fit the service.
    Malformed,
    /// The local port had no free sample to write into.
    Exhausted,
    /// The transcoder failed with its own error.
    Transcoder(TranscoderError),
}

impl<TranscoderError: core::fmt::Display> core::fmt::Display for TranscodeError<TranscoderError> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Malformed => write!(f, "TranscodeError::Malformed"),
            Self::Exhausted => write!(f, "TranscodeError::Exhausted"),
            Self::Transcoder(error) => write!(f, "TranscodeError::Transcoder({error})"),
        }
    }
}

impl<TranscoderError: Error> Error for TranscodeError<TranscoderError> {}

impl<TranscoderError> From<UnsupportedLength> for TranscodeError<TranscoderError> {
    fn from(_: UnsupportedLength) -> Self {
        Self::Malformed
    }
}

impl<TranscoderError> From<LoanError> for TranscodeError<TranscoderError> {
    fn from(refusal: LoanError) -> Self {
        match refusal {
            LoanError::Malformed | LoanError::NotResizable => Self::Malformed,
            LoanError::Exhausted => Self::Exhausted,
        }
    }
}

/// Whether one region of a sample is transcoded in one direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transcoding {
    /// The local form is the middleware's, bytes cross unchanged.
    Passthrough,
    /// The transcoder converts.
    Transcode,
}

/// Converts a sample's header between the local form and the
/// middleware's header form.
pub trait HeaderTranscoder {
    type Failure: Error;

    /// Writes the middleware's header form of a local header into `into`.
    fn encode<R: Region>(
        &self,
        header: &[u8],
        into: &mut R,
    ) -> Result<(), TranscodeError<Self::Failure>>;

    /// Writes the local header from the middleware's header form into
    /// the header of `writable`.
    fn decode<W: WritableSample>(
        &self,
        wire: &[u8],
        writable: &mut W,
    ) -> Result<(), TranscodeError<Self::Failure>>;
}

/// Converts a sample's payload between the local form and the
/// middleware's wire form.
pub trait PayloadTranscoder {
    type Failure: Error;

    /// Writes the middleware's wire form of a local payload into `into`.
    fn encode<R: Region>(
        &self,
        payload: &[u8],
        into: &mut R,
    ) -> Result<(), TranscodeError<Self::Failure>>;

    /// Loans `loanable` for the local payload's length and writes the
    /// local payload from the middleware's wire form.
    fn decode<L: LoanableSample>(
        &self,
        wire: &[u8],
        loanable: L,
    ) -> Result<L::Sample, TranscodeError<Self::Failure>>;
}

/// The transcoding of each region of a sample.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SampleTranscodings {
    pub header: Transcoding,
    pub payload: Transcoding,
}

impl SampleTranscodings {
    /// Every region crosses unchanged.
    pub const PASSTHROUGH: Self = Self {
        header: Transcoding::Passthrough,
        payload: Transcoding::Passthrough,
    };
    /// Every region is transcoded.
    pub const TRANSCODE: Self = Self {
        header: Transcoding::Transcode,
        payload: Transcoding::Transcode,
    };
}

/// Transcodes the regions of a sample.
pub trait SampleTranscoder {
    type HeaderTranscoder: HeaderTranscoder;
    type PayloadTranscoder: PayloadTranscoder;

    fn headers(&self) -> &Self::HeaderTranscoder;
    fn payloads(&self) -> &Self::PayloadTranscoder;
}

/// A sample transcoder from one transcoder per region.
#[derive(Debug, Clone, Copy, Default)]
pub struct SampleTranscoders<HeaderTranscoder, PayloadTranscoder> {
    pub headers: HeaderTranscoder,
    pub payloads: PayloadTranscoder,
}

impl<H: HeaderTranscoder, P: PayloadTranscoder> SampleTranscoder for SampleTranscoders<H, P> {
    type HeaderTranscoder = H;
    type PayloadTranscoder = P;

    fn headers(&self) -> &H {
        &self.headers
    }

    fn payloads(&self) -> &P {
        &self.payloads
    }
}

/// The transcoder of a region the translation passes through, never
/// asked.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoTranscoder;

impl HeaderTranscoder for NoTranscoder {
    type Failure = core::convert::Infallible;

    fn encode<R: Region>(&self, _: &[u8], _: &mut R) -> Result<(), TranscodeError<Self::Failure>> {
        unreachable!("the translation passes the header through, it is never encoded")
    }

    fn decode<W: WritableSample>(
        &self,
        _: &[u8],
        _: &mut W,
    ) -> Result<(), TranscodeError<Self::Failure>> {
        unreachable!("the translation passes the header through, it is never decoded")
    }
}

impl PayloadTranscoder for NoTranscoder {
    type Failure = core::convert::Infallible;

    fn encode<R: Region>(&self, _: &[u8], _: &mut R) -> Result<(), TranscodeError<Self::Failure>> {
        unreachable!("the translation passes the payload through, it is never encoded")
    }

    fn decode<L: LoanableSample>(
        &self,
        _: &[u8],
        _: L,
    ) -> Result<L::Sample, TranscodeError<Self::Failure>> {
        unreachable!("the translation passes the payload through, it is never decoded")
    }
}

impl SampleTranscoder for NoTranscoder {
    type HeaderTranscoder = NoTranscoder;
    type PayloadTranscoder = NoTranscoder;

    fn headers(&self) -> &NoTranscoder {
        self
    }

    fn payloads(&self) -> &NoTranscoder {
        self
    }
}
