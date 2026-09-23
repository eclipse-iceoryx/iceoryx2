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

use crate::{SampleBytes, SampleBytesRef, SampleBytesRefMut, SampleLengths};

/// Where a taken sample's bytes are written.
pub trait TakeDestination<'a> {
    /// The locations for a sample of `lengths`.
    ///
    /// Returns `None` if the destination cannot honor the requested lengths.
    fn for_lengths(self, lengths: SampleLengths) -> Option<SampleBytesRefMut<'a>>;
}

impl<'a> TakeDestination<'a> for &'a mut SampleBytes {
    fn for_lengths(self, lengths: SampleLengths) -> Option<SampleBytesRefMut<'a>> {
        self.header.resize(lengths.header, 0);
        self.payload.resize(lengths.payload, 0);
        Some(self.as_mut())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TakeOutcome {
    /// The sample was written into the destination.
    Taken,
    /// The destination declined the sample.
    Declined,
    /// A sample was taken and dropped as one not meant for the link. More
    /// may be pending.
    Skipped,
    /// Nothing was pending.
    Empty,
}

/// The gateway's publisher and subscription on the middleware.
pub trait PublishSubscribeEndpoints {
    type Failure: Error;

    /// Publishes the bytes of a sample, provided in its wire form.
    fn publish(&mut self, sample: SampleBytesRef<'_>) -> Result<(), Self::Failure>;

    /// Takes the pending sample, if any, in the middleware's wire form into
    /// the locations `destination` provides for its lengths.
    fn take<'a>(
        &mut self,
        destination: impl TakeDestination<'a>,
    ) -> Result<TakeOutcome, Self::Failure>;
}
