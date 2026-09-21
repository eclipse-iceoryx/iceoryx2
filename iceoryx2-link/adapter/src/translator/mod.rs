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

//! Decides which local types correspond to a middleware's types, and how
//! the samples of a service are converted between the two forms, for
//! each messaging pattern.
//!
//! The publish-subscribe translation is
//! [`PublishSubscribeTranslation::Passthrough`] when the local form is
//! already the middleware's. Otherwise it states for the header and for
//! the payload, in each direction, whether the transcoder runs, and it
//! provides the transcoder:
//!
//! ```rust,ignore
//! impl Translator for MyTranslator {
//!     type EndpointTypes = MyEndpointTypes;
//!     type Error = MyError;
//!     type Transcoder = SampleTranscoders<NoHeader, MyPayloadTranscoder>;
//!
//!     fn local(&self, remote: &MyEndpointTypes) -> Result<ServiceTypes, MyError> {
//!         // The local types standing for the middleware's.
//!     }
//!
//!     fn remote(&self, local: &ServiceTypes) -> Result<MyEndpointTypes, MyError> {
//!         // The middleware's types standing for the local ones.
//!     }
//!
//!     fn publish_subscribe(
//!         &self,
//!         local: &ServiceTypes,
//!         remote: &MyEndpointTypes,
//!     ) -> Result<PublishSubscribeTranslation<Self::Transcoder>, MyError> {
//!         // The translation for a publish-subscribe service between the
//!         // `ServiceTypes` shared by the local service and `EndpointTypes`
//!         // communicated over the endpoint.
//!     }
//! }
//! ```
//!
//! A sample transcoder pairs a [`HeaderTranscoder`] with a
//! [`PayloadTranscoder`] as [`SampleTranscoders`], one per region. A region's
//! transcoder converts the bytes it is given into the region it is given.
//! It first asks the region for the converted form's length, which the
//! region may refuse, then writes:
//!
//! ```rust,ignore
//! impl PayloadTranscoder for MyPayloadTranscoder {
//!     type Failure = MyError;
//!
//!     fn encode<R: Region>(&self, payload: &[u8], into: &mut R) -> Result<(), TranscodeError<MyError>> {
//!         // Resize the region to the length required for the wire form and
//!         // write the payload in wire form.
//!         //
//!         // Wrap failures in TranscodeError::Transcoder.
//!     }
//!
//!     fn decode<L: LoanableSample>(&self, wire: &[u8], loanable: L) -> Result<L::Sample, TranscodeError<MyError>> {
//!         // Loan for the local form's length, write the local form of the
//!         // wire bytes into the sample's payload and return the sample.
//!         //
//!         // Wrap failures in TranscodeError::Transcoder.
//!     }
//! }
//! ```

mod publish_subscribe;
mod transcoder;

pub use publish_subscribe::PublishSubscribeTranslation;
pub use transcoder::{
    HeaderTranscoder, NoTranscoder, PayloadTranscoder, SampleTranscoder, SampleTranscoders,
    SampleTranscodings, TranscodeError, Transcoding,
};

use core::error::Error;

use iceoryx2_link_backend::service_description::ServiceTypes;

/// Decides which local types correspond to a middleware's types, and how
/// the regions of a sample are converted between the two forms.
pub trait Translator {
    /// The middleware's types of an endpoint.
    type EndpointTypes: Clone + PartialEq + 'static;
    type Error: Error;
    type Transcoder: SampleTranscoder;

    /// The local types an endpoint's data has.
    fn local(&self, remote: &Self::EndpointTypes) -> Result<ServiceTypes, Self::Error>;

    /// The endpoint types a local service's data has.
    fn remote(&self, local: &ServiceTypes) -> Result<Self::EndpointTypes, Self::Error>;

    /// The translation of a publish-subscribe service's samples between
    /// the local types and the endpoint's types.
    fn publish_subscribe(
        &self,
        local: &ServiceTypes,
        remote: &Self::EndpointTypes,
    ) -> Result<PublishSubscribeTranslation<Self::Transcoder>, Self::Error>;
}

/// The translator of a middleware whose data already has local types,
/// passing everything through unchanged.
#[derive(Debug, Clone, Copy, Default)]
pub struct Passthrough;

impl Translator for Passthrough {
    type EndpointTypes = ServiceTypes;
    type Error = core::convert::Infallible;
    type Transcoder = NoTranscoder;

    fn local(&self, remote: &ServiceTypes) -> Result<ServiceTypes, Self::Error> {
        Ok(remote.clone())
    }

    fn remote(&self, local: &ServiceTypes) -> Result<ServiceTypes, Self::Error> {
        Ok(local.clone())
    }

    fn publish_subscribe(
        &self,
        _: &ServiceTypes,
        _: &ServiceTypes,
    ) -> Result<PublishSubscribeTranslation<Self::Transcoder>, Self::Error> {
        Ok(PublishSubscribeTranslation::Passthrough)
    }
}
