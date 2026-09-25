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
//! A [`Translator`] maps local types to a middleware's types and chooses
//! the transcoders converting samples between the two forms. The
//! [`SampleTranscoders`] name the regions that are transcoded and the
//! transcoder of each:
//!
//! ```rust,ignore
//! impl Translator for MyTranslator {
//!     type RemoteTypes = MyMiddlewareTypes;
//!     type Transcoders = SampleTranscoders<NoTranscoder, MyPayloadTranscoder>;
//!     type Error = MyError;
//!
//!     fn local(&self, remote: &MyMiddlewareTypes) -> Result<SampleTypes, MyError> {
//!         // The local types corresponding to the given remote types.
//!     }
//!
//!     fn remote(&self, local: &SampleTypes) -> Result<MyMiddlewareTypes, MyError> {
//!         // The remote types corresponding to the local type stored in
//!         // shared memory.
//!     }
//!
//!     fn transcoders(
//!         &self,
//!         local: &SampleTypes,
//!         remote: &MyMiddlewareTypes,
//!     ) -> Result<Self::Transcoders, MyError> {
//!         // Which regions are transcoded between the given pair of local
//!         // and remote types, and by what. A region without a transcoder
//!         // crosses unchanged.
//!         Ok(SampleTranscoders::TranscodePayload(MyPayloadTranscoder))
//!     }
//! }
//! ```
//!
//! A [`Transcoder`] converts the bytes of one region between local and
//! middleware form. A translator that never transcodes a region names
//! [`NoTranscoder`] as its transcoder:
//!
//! ```rust,ignore
//! impl<'a> Transcoder<SampleBytesRef<'a>> for MyPayloadTranscoder {
//!     type Error = MyError;
//!
//!     fn encode<R: Region>(&self, local: SampleBytesRef<'a>, into: &mut R) -> Result<(), TranscodeError<MyError>> {
//!         // Size the provided `into` region for the middleware form of the
//!         // `local` payload and write it there.
//!         //
//!         // Return:
//!         // * TranscodeError::Refused if `into` refuses the size
//!         // * TranscodeError::Transcoder on the transcoder's own error
//!     }
//!
//!     fn decode<R: Region>(&self, wire: SampleBytesRef<'a>, into: &mut R) -> Result<(), TranscodeError<MyError>> {
//!         // Same in the opposite direction.
//!     }
//! }
//! ```

mod transcoder;

pub use transcoder::{NoTranscoder, TranscodeError, Transcoder};

use core::convert::Infallible;
use core::error::Error;

use iceoryx2_link_backend::service_description::SampleTypes;

use crate::SampleBytesRef;

/// Decides which local types correspond to a middleware's remote types and
/// how the bytes of a sample are converted between the two forms.
pub trait Translator {
    /// The middleware's types of a sample.
    type RemoteTypes: Clone + PartialEq + 'static;
    type Transcoders: TranscodesSamples;
    type Error: Error;

    /// The local types of the samples the middleware carries with `remote`.
    fn local(&self, remote: &Self::RemoteTypes) -> Result<SampleTypes, Self::Error>;

    /// The middleware's types of the samples with the local types `local`.
    fn remote(&self, local: &SampleTypes) -> Result<Self::RemoteTypes, Self::Error>;

    /// The transcoders converting samples between `local` and `remote`.
    fn transcoders(
        &self,
        local: &SampleTypes,
        remote: &Self::RemoteTypes,
    ) -> Result<Self::Transcoders, Self::Error>;
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

/// The transcoders of a sample.
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

/// The translator of a middleware whose data already has local types,
/// passing everything through unchanged.
#[derive(Debug, Clone, Copy, Default)]
pub struct Passthrough;

impl Translator for Passthrough {
    type RemoteTypes = SampleTypes;
    type Transcoders = SampleTranscoders<NoTranscoder, NoTranscoder>;
    type Error = Infallible;

    fn local(&self, remote: &SampleTypes) -> Result<SampleTypes, Self::Error> {
        Ok(remote.clone())
    }

    fn remote(&self, local: &SampleTypes) -> Result<SampleTypes, Self::Error> {
        Ok(local.clone())
    }

    fn transcoders(
        &self,
        _: &SampleTypes,
        _: &SampleTypes,
    ) -> Result<Self::Transcoders, Self::Error> {
        Ok(SampleTranscoders::TranscodeNone)
    }
}
