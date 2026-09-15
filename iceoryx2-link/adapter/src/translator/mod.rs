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

    /// How the samples of a publish-subscribe service are
    /// converted between the local types and the endpoint's types.
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
