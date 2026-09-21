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

use iceoryx2::service::Service;

use crate::wire::publish_subscribe::Sample;
use crate::wire::sample::LoanableSample;

/// Relays publish-subscribe payloads over the backend.
pub trait PublishSubscribeRelay<S: Service> {
    type SendError: Error;
    type ReceiveError: Error;

    /// Sends a sample taken from the local subscriber to the opposing
    /// side.
    fn send(&mut self, sample: &Sample<S>) -> Result<(), Self::SendError>;

    /// Receives one sample from the opposing side into `loanable`.
    fn receive<L: LoanableSample>(
        &mut self,
        loanable: L,
    ) -> Result<ReceiveOutcome<L::Sample>, Self::ReceiveError>;
}

/// The outcome of receiving one sample.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReceiveOutcome<W> {
    /// A sample was written.
    Sample(W),
    /// A sample was taken and dropped as one not meant for the link. More
    /// may be pending.
    Skipped,
    /// Nothing was pending.
    Empty,
}
