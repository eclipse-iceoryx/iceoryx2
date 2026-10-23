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

mod endpoint_description;
mod event;
mod publish_subscribe;
mod unsupported;

pub use endpoint_description::{EndpointDescription, EndpointTypes};
pub use event::EventEndpoints;
pub use publish_subscribe::PublishSubscribeEndpoints;
pub use unsupported::UnsupportedEndpoints;

use crate::{SampleBytes, SampleBytesRefMut, SampleLengths};

/// Where a taken sample's bytes are written.
pub trait SampleDestination<'a> {
    /// The locations for a sample of `lengths`.
    ///
    /// Returns `None` if the destination cannot honor the requested lengths.
    fn for_lengths(self, lengths: SampleLengths) -> Option<SampleBytesRefMut<'a>>;
}

impl<'a> SampleDestination<'a> for &'a mut SampleBytes {
    fn for_lengths(self, lengths: SampleLengths) -> Option<SampleBytesRefMut<'a>> {
        self.header.resize(lengths.header, 0);
        self.payload.resize(lengths.payload, 0);
        Some(self.as_mut())
    }
}

/// The possible outcomes taking from the middleware may provide.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TakeOutcome<Details> {
    /// The sample was written into the destination with optional details
    /// describing the take.
    Taken(Details),
    /// The destination declined the sample.
    Declined,
    /// A sample was taken and dropped as one not meant for the link. More
    /// may be pending.
    Skipped,
    /// Nothing was pending.
    Empty,
}
