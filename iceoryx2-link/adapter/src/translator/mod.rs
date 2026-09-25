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
//! the transcoders converting between the two forms, for data of one
//! [`Shape`]. Samples have the [`SampleShape`], a header and a payload,
//! and are converted by [`SampleTranscoders`], which name the regions that
//! are transcoded and the transcoder of each:
//!
//! ```rust,ignore
//! impl Translator<SampleShape> for MyTranslator {
//!     type RemoteTypes = MyMiddlewareTypes;
//!     type Transcoders = SampleTranscoders<NoTranscoder, MyPayloadTranscoder>;
//!     type Error = MyError;
//!
//!     fn local(&self, remote: &MyMiddlewareTypes) -> Result<LocalTypes<SampleShape>, MyError> {
//!         // The local types corresponding to the given remote types.
//!     }
//!
//!     fn remote(&self, local: &LocalTypes<SampleShape>) -> Result<MyMiddlewareTypes, MyError> {
//!         // The remote types corresponding to the local type stored in
//!         // shared memory.
//!     }
//!
//!     fn transcoders(
//!         &self,
//!         local: &LocalTypes<SampleShape>,
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

mod passthrough;
mod sample;
mod transcoder;

pub use passthrough::Passthrough;
pub use sample::{SampleShape, SampleTranscoders, TranscodesSamples};
pub use transcoder::{NoTranscoder, TranscodeError, Transcoder};

use core::error::Error;

/// One shape of data that a translator can translate.
pub trait Shape {
    /// The local types matching the shape of the data.
    type LocalTypes: Clone + PartialEq;
}

/// The local types of data with the shape `S`.
pub type LocalTypes<S> = <S as Shape>::LocalTypes;

/// Decides which local types correspond to a middleware's remote types and
/// how the bytes are converted between the two forms for data of one
/// [`Shape`].
pub trait Translator<S: Shape> {
    /// The middleware's types of the data.
    type RemoteTypes: Clone + PartialEq + 'static;
    type Transcoders;
    type Error: Error;

    /// The local types of the data the middleware carries with `remote`.
    fn local(&self, remote: &Self::RemoteTypes) -> Result<S::LocalTypes, Self::Error>;

    /// The middleware's types of the data with the local types `local`.
    fn remote(&self, local: &S::LocalTypes) -> Result<Self::RemoteTypes, Self::Error>;

    /// The transcoders converting the data between `local` and `remote`.
    fn transcoders(
        &self,
        local: &S::LocalTypes,
        remote: &Self::RemoteTypes,
    ) -> Result<Self::Transcoders, Self::Error>;
}
