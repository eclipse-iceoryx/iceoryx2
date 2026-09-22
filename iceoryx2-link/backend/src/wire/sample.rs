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

//! Defines the state machine for loaning and populating a sample.
//!
//! ```text
//!   ┌──────────────────┐   loan(payload_len)   ┌──────────────────┐   into_sample()
//!   │  LoanableSample  │ ────────────────────▶ │  WritableSample  │ ────────────────▶ the sample to send
//!   │                  │                       │                  │
//!   │  no memory yet   │                       │  payload()       │
//!   │                  │                       │  header(len)     │
//!   └──────────────────┘                       └──────────────────┘
//!            │                                          │
//!            ▼ dropped                                  ▼ dropped
//!       nothing loaned                          the loan is returned
//! ```

use core::mem::MaybeUninit;

use iceoryx2::service::marker::{CustomHeaderMarker, CustomPayloadMarker};
use iceoryx2::service::static_config::message_type_details::TypeVariant;

use crate::service_description::SampleTypes;
use crate::wire::UnsupportedLength;

/// The untyped user header as it passes through a relay.
pub type Header = CustomHeaderMarker;
/// The untyped payload as it passes through a relay.
pub type Payload = [CustomPayloadMarker];
pub type PayloadUninit = [MaybeUninit<CustomPayloadMarker>];

/// Loans a sample for a payload length and provides a [`WritableSample`]
/// that can be populated.
pub trait LoanableSample {
    /// The loaned sample with writable header and payload.
    type Sample: WritableSample;

    /// The size of the header.
    fn header_size(&self) -> usize;

    /// Acquire a loan for the sample.
    ///
    /// Fails with:
    ///
    /// * `Malformed` if no `payload_len` is an invalid length for a sample of
    ///   this service
    /// * `Exhausted` if no sample could be loaned
    fn loan(self, payload_len: usize) -> Result<Self::Sample, LoanError>;
}

/// Why no sample was loaned.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoanError {
    /// No sample of this length can exist for the service.
    Malformed,
    /// The port had none to give.
    Exhausted,
    /// The loaned sample holds a payload of another length and cannot be
    /// resized to this one.
    NotResizable,
}

impl core::fmt::Display for LoanError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "LoanError::{self:?}")
    }
}

impl core::error::Error for LoanError {}

/// Provides the location to write a sample's header and payload.
pub trait WritableSample {
    /// The location into which the payload bytes should be written.
    fn payload(&mut self) -> &mut [u8];

    /// The location into which the header bytes should be written.
    fn header(&mut self, len: usize) -> Result<&mut [u8], UnsupportedLength>;
}

/// Why a sample refused a write.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WriteError {
    /// The bytes do not fit a sample of the service.
    Malformed,
    /// The port had no sample to give.
    Exhausted,
}

impl From<UnsupportedLength> for WriteError {
    fn from(_: UnsupportedLength) -> Self {
        Self::Malformed
    }
}

impl From<LoanError> for WriteError {
    fn from(refusal: LoanError) -> Self {
        match refusal {
            LoanError::Malformed | LoanError::NotResizable => Self::Malformed,
            LoanError::Exhausted => Self::Exhausted,
        }
    }
}

impl core::fmt::Display for WriteError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "WriteError::{self:?}")
    }
}

impl core::error::Error for WriteError {}

/// Whether a header and a payload of these lengths fit a sample of the
/// described service.
pub fn fits(types: &SampleTypes, header: usize, payload: usize) -> bool {
    if header != types.user_header.size {
        return false;
    }

    let size = types.payload.size;
    match types.payload.variant {
        TypeVariant::FixedSize => payload == size,
        TypeVariant::Dynamic => size > 0 && payload.is_multiple_of(size),
    }
}

/// Views a sample's user header as bytes.
///
/// # Safety
///
/// `size` must be the user header size of the service the sample belongs
/// to, as its description states.
pub unsafe fn user_header_bytes(user_header: &Header, size: usize) -> &[u8] {
    unsafe { core::slice::from_raw_parts(user_header as *const Header as *const u8, size) }
}

/// Views a sample's payload as bytes.
pub fn payload_bytes(payload: &Payload) -> &[u8] {
    // SAFETY: the payload marker is one byte with no padding, so a slice
    // of markers is a slice of bytes of the same length.
    unsafe { core::slice::from_raw_parts(payload.as_ptr() as *const u8, payload.len()) }
}
