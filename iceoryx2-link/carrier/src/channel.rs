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

use iceoryx2_link_backend::wire::sample::{LoanableSample, WriteError};

use crate::Frame;

/// Carries the frames of one service in both directions.
pub trait Channel {
    type Error: Error;

    /// Sends one frame to every peer on the channel.
    fn send(&mut self, frame: Frame<'_>) -> Result<(), Self::Error>;

    /// Receives the next pending frame into `loanable`, or returns `None`
    /// if nothing is pending.
    ///
    /// The frame's header bytes go to the header and its payload bytes to
    /// the payload. A frame `loanable` refuses is consumed and dropped, and
    /// the refusal returned.
    fn receive<L: LoanableSample>(
        &mut self,
        loanable: L,
    ) -> Result<Option<L::Sample>, ReceiveError<Self::Error>>;
}

/// Why a receive ended without a frame written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReceiveError<Failure> {
    /// The sample refused the frame, which is dropped.
    Rejected(WriteError),
    /// The channel failed.
    Failed(Failure),
}

impl<Failure: core::fmt::Display> core::fmt::Display for ReceiveError<Failure> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Rejected(refusal) => write!(f, "ReceiveError::Rejected({refusal})"),
            Self::Failed(error) => write!(f, "ReceiveError::Failed({error})"),
        }
    }
}

impl<Failure: Error> Error for ReceiveError<Failure> {}

impl<Failure> From<WriteError> for ReceiveError<Failure> {
    fn from(refusal: WriteError) -> Self {
        Self::Rejected(refusal)
    }
}
