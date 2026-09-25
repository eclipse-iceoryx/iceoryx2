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

use core::convert::Infallible;

use iceoryx2_link_backend::service_description::SampleTypes;

use super::{LocalTypes, NoTranscoder, SampleShape, SampleTranscoders, Translator};

/// The translator of a middleware whose data already has local types,
/// passing everything through unchanged.
#[derive(Debug, Clone, Copy, Default)]
pub struct Passthrough;

impl Translator<SampleShape> for Passthrough {
    type RemoteTypes = SampleTypes;
    type Transcoders = SampleTranscoders<NoTranscoder, NoTranscoder>;
    type Error = Infallible;

    fn local(&self, remote: &SampleTypes) -> Result<LocalTypes<SampleShape>, Self::Error> {
        Ok(remote.clone())
    }

    fn remote(&self, local: &LocalTypes<SampleShape>) -> Result<SampleTypes, Self::Error> {
        Ok(local.clone())
    }

    fn transcoders(
        &self,
        _: &LocalTypes<SampleShape>,
        _: &SampleTypes,
    ) -> Result<Self::Transcoders, Self::Error> {
        Ok(SampleTranscoders::TranscodeNone)
    }
}
