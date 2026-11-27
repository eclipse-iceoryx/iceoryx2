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

use flatbuffers::{Allocator, FlatBufferBuilder};
use iceoryx2_log::{fail, origin};

use super::{Compiled, cdr, flatbuffer};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecodeError {
    /// The CDR ends before the message does.
    Truncated,
    /// The CDR names an encapsulation other than plain CDR.
    UnsupportedEncapsulation,
    InvalidBool,
    InvalidString,
}

impl core::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "DecodeError::{self:?}")
    }
}

impl core::error::Error for DecodeError {}

impl From<flatbuffer::WriteError<cdr::ReadError>> for DecodeError {
    fn from(error: flatbuffer::WriteError<cdr::ReadError>) -> Self {
        match error {
            flatbuffer::WriteError::Values(error) => error.into(),
        }
    }
}

impl From<cdr::ReadError> for DecodeError {
    fn from(error: cdr::ReadError) -> Self {
        match error {
            cdr::ReadError::Truncated => DecodeError::Truncated,
            cdr::ReadError::UnsupportedEncapsulation => DecodeError::UnsupportedEncapsulation,
            cdr::ReadError::InvalidBool => DecodeError::InvalidBool,
            cdr::ReadError::InvalidString => DecodeError::InvalidString,
        }
    }
}

/// Decodes the CDR `cdr` of the message of `compiled` into a flatbuffer built
/// by `builder`.
///
/// Fails when the CDR does not hold the message.
pub fn decode<A: Allocator>(
    compiled: &Compiled,
    cdr: &[u8],
    builder: &mut FlatBufferBuilder<'_, A>,
) -> Result<(), DecodeError> {
    let origin = origin!("decode");

    let message = fail!(
        from origin,
        when cdr::locate(&compiled.cdr, cdr),
        to DecodeError,
        "Unable to locate the values in the CDR of {} bytes", cdr.len()
    );

    fail!(
        from origin,
        when flatbuffer::write(&compiled.flatbuffer, &message.reader(), builder),
        to DecodeError,
        "Unable to write the values of the CDR into the flatbuffer"
    );

    Ok(())
}
