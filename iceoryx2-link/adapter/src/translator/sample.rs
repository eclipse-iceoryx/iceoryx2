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

use iceoryx2_link_backend::service_description::SampleTypes;

use super::{Shape, Transcoder};
use crate::SampleBytesRef;

/// The shape of a sample with a header and a payload.
#[derive(Debug, Clone, Copy)]
pub struct SampleShape;

impl Shape for SampleShape {
    type LocalTypes = SampleTypes;
}

/// Which regions of a sample are transcoded between the local and the
/// middleware form, and the transcoder(s) used.
#[derive(Debug)]
pub enum SampleTranscoders<H, P> {
    /// Both regions pass through unchanged.
    TranscodeNone,
    /// The header is transcoded, the payload passes through.
    TranscodeHeader(H),
    /// The payload is transcoded, the header passes through.
    TranscodePayload(P),
    /// Both regions are transcoded.
    TranscodeBoth(H, P),
}

/// The transcoders of data with the [`SampleShape`].
pub trait TranscodesSamples {
    type HeaderTranscoder: for<'a> Transcoder<SampleBytesRef<'a>>;
    type PayloadTranscoder: for<'a> Transcoder<SampleBytesRef<'a>>;

    fn for_samples(&self) -> &SampleTranscoders<Self::HeaderTranscoder, Self::PayloadTranscoder>;
}

impl<H, P> TranscodesSamples for SampleTranscoders<H, P>
where
    H: for<'a> Transcoder<SampleBytesRef<'a>>,
    P: for<'a> Transcoder<SampleBytesRef<'a>>,
{
    type HeaderTranscoder = H;
    type PayloadTranscoder = P;

    fn for_samples(&self) -> &Self {
        self
    }
}
