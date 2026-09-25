// Copyright (c) 2025 Contributors to the Eclipse Foundation
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

use iceoryx2_link_backend::wire::sample::SampleBytesRef;
use iceoryx2_link_carrier::SampleChannel;

use super::{Error, ZenohChannel};

/// The samples of one service over zenoh.
pub struct ZenohSampleChannel(pub(crate) ZenohChannel);

impl SampleChannel for ZenohSampleChannel {
    type Error = Error;

    fn send(&mut self, sample: SampleBytesRef<'_>) -> Result<(), Self::Error> {
        self.0.send(&[sample.header, sample.payload])
    }

    fn receive(&mut self) -> Result<Option<&[u8]>, Self::Error> {
        Ok(self.0.receive())
    }
}
