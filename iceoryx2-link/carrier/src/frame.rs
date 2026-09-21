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

use alloc::vec::Vec;

use iceoryx2_link_backend::wire::sample::{LoanableSample, WritableSample, WriteError};
use iceoryx2_log::{fail, origin};

/// A publish-subscribe sample as it crosses a channel, the user header
/// bytes followed by the payload bytes.
#[derive(Debug, Clone, Copy)]
pub struct Frame<'a> {
    pub header: &'a [u8],
    pub payload: &'a [u8],
}

impl<'a> Frame<'a> {
    /// The frame as one buffer, header then payload.
    pub fn to_bytes(&self) -> Vec<u8> {
        [self.header, self.payload].concat()
    }

    /// The frame in `bytes` split at `header_size`, malformed if `bytes`
    /// is shorter than that.
    pub fn split(bytes: &'a [u8], header_size: usize) -> Result<Self, Malformed> {
        let origin = origin!("Frame::split");
        if bytes.len() < header_size {
            fail!(
                from origin,
                with Malformed,
                "A frame of {} bytes cannot hold a user header of {}", bytes.len(), header_size
            );
        }
        let (header, payload) = bytes.split_at(header_size);
        Ok(Self { header, payload })
    }

    /// Loans `loanable` for the payload and writes the frame into it.
    pub fn write_into<L: LoanableSample>(&self, loanable: L) -> Result<L::Sample, WriteError> {
        let origin = origin!("Frame::write_into");
        let mut writable = fail!(
            from origin,
            when loanable.loan(self.payload.len()),
            to WriteError,
            "A payload of {} bytes was refused", self.payload.len()
        );
        writable.payload().copy_from_slice(self.payload);
        let header = fail!(
            from origin,
            when writable.header(self.header.len()),
            to WriteError,
            "A header of {} bytes was refused", self.header.len()
        );
        header.copy_from_slice(self.header);
        Ok(writable)
    }
}

/// The bytes are shorter than the service's user header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Malformed;

impl core::fmt::Display for Malformed {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Malformed")
    }
}

impl core::error::Error for Malformed {}

#[cfg(test)]
mod tests {
    use super::*;

    use iceoryx2_bb_testing::assert_that;

    const HEADER_SIZE: usize = 2;
    const BYTES: [u8; 6] = [1, 2, 3, 4, 5, 6];

    #[test]
    fn bytes_split_into_header_and_payload() {
        let frame = Frame::split(&BYTES, HEADER_SIZE).expect("long enough");
        assert_that!(frame.header, eq & BYTES[..HEADER_SIZE]);
        assert_that!(frame.payload, eq & BYTES[HEADER_SIZE..]);
    }

    #[test]
    fn bytes_shorter_than_the_header_are_malformed() {
        assert_that!(Frame::split(&BYTES[..HEADER_SIZE - 1], HEADER_SIZE).is_err(), eq true);
    }
}
