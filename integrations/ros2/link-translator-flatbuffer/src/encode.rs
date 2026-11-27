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

use core::convert::Infallible;

use iceoryx2_link_backend::wire::region::{ForwardRegion, ResizeError};
use iceoryx2_log::{fail, origin};

use super::{Compiled, cdr, flatbuffer};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncodeError {
    /// The flatbuffer holds no root offset.
    Truncated,
    /// The region cannot hold the CDR.
    Region(ResizeError),
    /// A string or a sequence exceeds its length prefix.
    TooLong,
    /// A vector holds another number of elements than the array it is
    /// encoded as.
    ArrayLength { expected: usize, actual: usize },
}

impl core::fmt::Display for EncodeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "EncodeError::{self:?}")
    }
}

impl core::error::Error for EncodeError {}

impl From<flatbuffer::ReadError> for EncodeError {
    fn from(error: flatbuffer::ReadError) -> Self {
        match error {
            flatbuffer::ReadError::MissingRootOffset => EncodeError::Truncated,
        }
    }
}

impl From<cdr::WriteError<Infallible>> for EncodeError {
    fn from(error: cdr::WriteError<Infallible>) -> Self {
        match error {
            cdr::WriteError::Values(never) => match never {},
            cdr::WriteError::Region(error) => EncodeError::Region(error),
            cdr::WriteError::TooLong => EncodeError::TooLong,
            cdr::WriteError::ArrayLength { expected, actual } => {
                EncodeError::ArrayLength { expected, actual }
            }
        }
    }
}

/// Encodes `flatbuffer` as the CDR of the message of `compiled` into `region`.
///
/// Fails when the flatbuffer holds no root offset or the region cannot hold
/// the CDR.
pub fn encode<R: ForwardRegion>(
    compiled: &Compiled,
    flatbuffer: &[u8],
    region: R,
) -> Result<(), EncodeError> {
    let origin = origin!("encode");

    let reader = fail!(
        from origin,
        when flatbuffer::Reader::new(&compiled.flatbuffer, flatbuffer),
        to EncodeError,
        "Unable to read the flatbuffer of {} bytes", flatbuffer.len()
    );
    fail!(
        from origin,
        when cdr::write(&compiled.cdr, &reader, region),
        to EncodeError,
        "Unable to write the values of the flatbuffer as CDR"
    );

    Ok(())
}
