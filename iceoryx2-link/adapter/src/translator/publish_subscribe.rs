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

use super::SampleTranscodings;

/// How the samples of one publish-subscribe service are converted, if at
/// all.
#[derive(Debug)]
pub enum PublishSubscribeTranslation<X> {
    /// Every region crosses unchanged in both directions.
    Passthrough,
    /// `transcoder` converts the regions whose transcoding says so.
    Transcode {
        outbound: SampleTranscodings,
        inbound: SampleTranscodings,
        transcoder: X,
    },
}

impl<X> PublishSubscribeTranslation<X> {
    /// The transcodings of a sample on its way out.
    pub fn outbound(&self) -> SampleTranscodings {
        match self {
            Self::Passthrough => SampleTranscodings::PASSTHROUGH,
            Self::Transcode { outbound, .. } => *outbound,
        }
    }

    /// The transcodings of a middleware message on its way in.
    pub fn inbound(&self) -> SampleTranscodings {
        match self {
            Self::Passthrough => SampleTranscodings::PASSTHROUGH,
            Self::Transcode { inbound, .. } => *inbound,
        }
    }
}
