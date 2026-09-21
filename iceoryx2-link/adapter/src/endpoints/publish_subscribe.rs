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

use crate::{LoanError, LoanableSample, UnsupportedLength};

/// Why a take ended without a message written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TakeError<EndpointsError> {
    /// The message does not fit the service's header and payload sizes.
    Malformed,
    /// The local port had no free sample to write into.
    Exhausted,
    /// The endpoints failed with their own error.
    Endpoints(EndpointsError),
}

impl<EndpointsError: core::fmt::Display> core::fmt::Display for TakeError<EndpointsError> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Malformed => write!(f, "TakeError::Malformed"),
            Self::Exhausted => write!(f, "TakeError::Exhausted"),
            Self::Endpoints(error) => write!(f, "TakeError::Endpoints({error})"),
        }
    }
}

impl<EndpointsError: Error> Error for TakeError<EndpointsError> {}

impl<EndpointsError> From<UnsupportedLength> for TakeError<EndpointsError> {
    fn from(_: UnsupportedLength) -> Self {
        Self::Malformed
    }
}

impl<EndpointsError> From<LoanError> for TakeError<EndpointsError> {
    fn from(refusal: LoanError) -> Self {
        match refusal {
            LoanError::Malformed | LoanError::NotResizable => Self::Malformed,
            LoanError::Exhausted => Self::Exhausted,
        }
    }
}

/// The gateway's publisher and subscription on the middleware.
pub trait PublishSubscribeEndpoints {
    type Failure: Error;

    /// Publishes a message, `header` in the middleware's header form and
    /// `payload` in its wire form.
    fn publish(&mut self, header: &[u8], payload: &[u8]) -> Result<(), Self::Failure>;

    /// Takes the pending message if any, into `loanable`.
    ///
    /// The payload is written in the wire form and the header as the
    /// middleware's header form.
    ///
    /// Returns the written sample, or `None` if nothing was pending.
    /// A message `loanable` refuses is taken and dropped, and the refusal
    /// returned.
    fn take<L: LoanableSample>(
        &mut self,
        loanable: L,
    ) -> Result<Option<L::Sample>, TakeError<Self::Failure>>;
}
