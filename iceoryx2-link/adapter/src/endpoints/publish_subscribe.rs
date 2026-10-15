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

use super::{SampleDestination, TakeOutcome};
use crate::SampleBytesRef;

/// The gateway's publisher and subscription on the middleware.
pub trait PublishSubscribeEndpoints {
    type Failure: Error;

    /// Publishes the bytes of a sample, provided in its wire form.
    fn publish(&mut self, sample: SampleBytesRef<'_>) -> Result<(), Self::Failure>;

    /// Takes the pending sample, if any, in the middleware's wire form into
    /// the locations `destination` provides for its lengths.
    fn take<'a>(
        &mut self,
        destination: impl SampleDestination<'a>,
    ) -> Result<TakeOutcome<()>, Self::Failure>;
}
