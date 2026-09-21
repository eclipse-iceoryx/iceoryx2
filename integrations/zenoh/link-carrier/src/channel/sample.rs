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

use iceoryx2_link_backend::wire::sample::LoanableSample;
use iceoryx2_link_carrier::{SampleChannel, SampleReceiveError, populate};
use iceoryx2_log::{fail, origin};

use super::{Error, ZenohChannel};

/// The samples of one service over zenoh.
pub struct ZenohSampleChannel(pub(crate) ZenohChannel);

impl SampleChannel for ZenohSampleChannel {
    type Error = Error;

    fn send(&mut self, bytes: &[&[u8]]) -> Result<(), Self::Error> {
        self.0.put(bytes)
    }

    fn receive<L: LoanableSample>(
        &mut self,
        loanable: L,
    ) -> Result<Option<L::Sample>, SampleReceiveError<Self::Error>> {
        let origin = origin!("ZenohSampleChannel::receive");

        let Some(bytes) = self.0.pop() else {
            return Ok(None);
        };

        let sample = fail!(
            from origin,
            when populate(&bytes, loanable),
            to SampleReceiveError<Error>,
            "Dropped {} bytes", bytes.len()
        );

        Ok(Some(sample))
    }
}
