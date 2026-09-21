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

use iceoryx2_link_backend::service_description::{ServiceDescriptor, ServiceTypes};
use iceoryx2_link_backend::wire::sample::{LoanableSample, WritableSample, WriteError};
use iceoryx2_log::{fail, origin};

/// Carries the frames of one service in both directions.
pub trait Channel {
    type Error: Error;

    /// Send the bytes of one sample to every peer connected to the channel.
    fn send(&mut self, header: &[u8], payload: &[u8]) -> Result<(), Self::Error>;

    /// Receives the next pending sample into `loanable`, or returns `None`
    /// if nothing is pending.
    ///
    /// A frame `loanable` refuses should be dropped, and the refusal returned.
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

/// The size of the user header at the start of a frame of the described
/// service.
pub fn header_size(descriptor: &ServiceDescriptor) -> usize {
    match &descriptor.types {
        ServiceTypes::PublishSubscribe(types) => types.user_header.size,
        ServiceTypes::Event => 0,
    }
}

/// Acquire a loan from the `loanable` for the received bytes and populate
/// it with the provided bytes. Assumes header and payload bytes are
/// concatenated.
///
/// Fails with
///
/// * `Malformed` if `bytes` is shorter than the header, if the payload
///   length does not fit the service, or if `header_size` is not the
///   header size of the service the sample was loaned for
/// * `Exhausted` if no sample could be loaned
pub fn populate<L: LoanableSample>(
    header_size: usize,
    bytes: &[u8],
    loanable: L,
) -> Result<L::Sample, WriteError> {
    let origin = origin!("populate");

    if bytes.len() < header_size {
        fail!(
            from origin,
            with WriteError::Malformed,
            "A frame of {} bytes cannot hold a user header of {}", bytes.len(), header_size
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

#[cfg(test)]
mod tests {
    use super::*;

    use alloc::vec::Vec;
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_link_backend::wire::sample::LoanError;
    use iceoryx2_link_backend::wire::{Region, UnsupportedLength};

    const HEADER_SIZE: usize = 2;
    const BYTES: [u8; 6] = [1, 2, 3, 4, 5, 6];

    /// Buffers for a sample before its payload has a length.
    struct Buffer;

    impl LoanableSample for Buffer {
        type Sample = Buffers;

        fn loan(self, payload_len: usize) -> Result<Self::Sample, LoanError> {
            Ok(Buffers {
                header: Vec::new(),
                payload: alloc::vec![0; payload_len],
            })
        }
    }

    /// Buffers for a loaned sample, the header sized by its writer.
    struct Buffers {
        header: Vec<u8>,
        payload: Vec<u8>,
    }

    impl WritableSample for Buffers {
        fn payload(&mut self) -> &mut [u8] {
            &mut self.payload
        }

        fn header(&mut self, len: usize) -> Result<&mut [u8], UnsupportedLength> {
            self.header.for_length(len)
        }
    }

    #[test]
    fn bytes_are_split_into_header_and_payload() {
        let sample = populate(HEADER_SIZE, &BYTES, Buffer).expect("long enough");
        assert_that!(sample.header, eq BYTES[..HEADER_SIZE].to_vec());
        assert_that!(sample.payload, eq BYTES[HEADER_SIZE..].to_vec());
    }

    #[test]
    fn bytes_shorter_than_the_header_are_malformed() {
        let refused = populate(HEADER_SIZE, &BYTES[..HEADER_SIZE - 1], Buffer).err();
        assert_that!(refused, eq Some(WriteError::Malformed));
    }
}
