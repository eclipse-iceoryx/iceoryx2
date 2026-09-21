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

use iceoryx2_link_backend::wire::sample::{LoanableSample, WritableSample, WriteError};
use iceoryx2_log::{fail, origin};

/// A channel to carry one service's samples between local and remote peers.
pub trait SampleChannel {
    type Error: Error;

    /// Sends the bytes of one sample, given as consecutive slices, to every
    /// peer connected to the channel.
    fn send(&mut self, bytes: &[&[u8]]) -> Result<(), Self::Error>;

    /// Receives the next pending sample into `loanable`, or returns `None`
    /// if nothing is pending.
    ///
    /// Bytes `loanable` refuses should be dropped, and the refusal returned.
    fn receive<L: LoanableSample>(
        &mut self,
        loanable: L,
    ) -> Result<Option<L::Sample>, SampleReceiveError<Self::Error>>;
}

/// Reasons for receiving a sample may fail.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SampleReceiveError<ChannelError> {
    /// The received bytes do not fit the service's header and payload
    /// sizes. The bytes were dropped.
    Malformed,
    /// The local port had no free sample to write into. The bytes were
    /// dropped.
    Exhausted,
    /// The channel failed with its own error.
    Channel(ChannelError),
}

impl<ChannelError: core::fmt::Display> core::fmt::Display for SampleReceiveError<ChannelError> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Malformed => write!(f, "SampleReceiveError::Malformed"),
            Self::Exhausted => write!(f, "SampleReceiveError::Exhausted"),
            Self::Channel(error) => write!(f, "SampleReceiveError::Channel({error})"),
        }
    }
}

impl<ChannelError: Error> Error for SampleReceiveError<ChannelError> {}

impl<ChannelError> From<WriteError> for SampleReceiveError<ChannelError> {
    fn from(refusal: WriteError) -> Self {
        match refusal {
            WriteError::Malformed => Self::Malformed,
            WriteError::Exhausted => Self::Exhausted,
        }
    }
}

/// Acquire a loan from the `loanable` for the received bytes and populate
/// it with the provided bytes. Assumes header and payload bytes are
/// concatenated, the header `loanable.header_size()` bytes long.
///
/// Fails with
///
/// * `Malformed` if `bytes` is shorter than the header, if the payload
///   length does not fit the service, or if `header_size` is not the
///   header size of the service the sample was loaned for
/// * `Exhausted` if no sample could be loaned
pub fn populate<L: LoanableSample>(bytes: &[u8], loanable: L) -> Result<L::Sample, WriteError> {
    let origin = origin!("populate");

    let header_size = loanable.header_size();
    if bytes.len() < header_size {
        fail!(
            from origin,
            with WriteError::Malformed,
            "{} bytes cannot hold a user header of {}", bytes.len(), header_size
        );
    }
    let (header, payload) = bytes.split_at(header_size);

    let mut writable = fail!(
        from origin,
        when loanable.loan(payload.len()),
        to WriteError,
        "A payload of {} bytes was refused", payload.len()
    );
    writable.payload().copy_from_slice(payload);

    let into_header = fail!(
        from origin,
        when writable.header(header.len()),
        to WriteError,
        "A header of {} bytes was refused", header.len()
    );
    into_header.copy_from_slice(header);

    Ok(writable)
}
