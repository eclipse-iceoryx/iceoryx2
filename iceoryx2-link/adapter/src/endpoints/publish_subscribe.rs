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

use crate::{LoanError, LoanableSample, UnsupportedLength, WriteError};

/// Why a take ended without a message written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TakeError<Failure> {
    /// The sample refused the message, which is dropped.
    Rejected(WriteError),
    /// The endpoints failed.
    Failed(Failure),
}

impl<Failure: core::fmt::Display> core::fmt::Display for TakeError<Failure> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Rejected(refusal) => write!(f, "TakeError::Rejected({refusal})"),
            Self::Failed(error) => write!(f, "TakeError::Failed({error})"),
        }
    }
}

impl<Failure: Error> Error for TakeError<Failure> {}

impl<Failure> From<UnsupportedLength> for TakeError<Failure> {
    fn from(refusal: UnsupportedLength) -> Self {
        Self::Rejected(refusal.into())
    }
}

impl<Failure> From<LoanError> for TakeError<Failure> {
    fn from(refusal: LoanError) -> Self {
        Self::Rejected(refusal.into())
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
