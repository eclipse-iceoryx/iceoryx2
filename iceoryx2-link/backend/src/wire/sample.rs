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
//!   ┌──────────────────┐   loan_forward(len)    ┌──────────────────┐   as_mut()
//!   │  LoanableSample  │   loan_backward(len)   │  WritableSample  │ ────────────────▶ writable header and payload
//!   │                  │ ─────────────────────▶ │                  │   resize(len)
//!   │  no memory yet   │                        │  a loan for the  │ ────────────────▶ the payload at another length
//!   │                  │                        │  payload length  │   assume_init()
//!   └──────────────────┘                        └──────────────────┘ ────────────────▶ the initialized, sendable sample
//!            │                                           │
//!            ▼ dropped                                   ▼ dropped
//!
//!       nothing loaned                           the loan is returned
//! ```

use alloc::vec::Vec;
use core::mem::MaybeUninit;
use core::ops::{Deref, DerefMut};

use iceoryx2::service::marker::{CustomHeaderMarker, CustomPayloadMarker};
use iceoryx2::service::static_config::message_type_details::TypeVariant;

use crate::service_description::{SampleTypes, TypeIdentifier};
use crate::wire::region::{BackwardRegion, ForwardRegion, ResizeError};

/// The untyped user header as it passes through a relay.
pub type Header = CustomHeaderMarker;
/// The untyped payload as it passes through a relay.
pub type Payload = [CustomPayloadMarker];
pub type PayloadUninit = [MaybeUninit<CustomPayloadMarker>];

/// The lengths of a sample's header and payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SampleLengths {
    pub header: usize,
    pub payload: usize,
}

/// Reference to the bytes of a sample's header and payload.
#[derive(Debug, Clone, Copy)]
pub struct SampleBytesRef<'a> {
    pub header: &'a [u8],
    pub payload: &'a [u8],
}

/// Mutable reference to the bytes of a sample's header and payload.
#[derive(Debug)]
pub struct SampleBytesRefMut<'a> {
    pub header: &'a mut [u8],
    pub payload: &'a mut [u8],
}

/// The bytes of a sample's header and payload in heap buffers.
#[derive(Debug, Default)]
pub struct SampleBytes {
    pub header: Vec<u8>,
    pub payload: Vec<u8>,
}

impl SampleBytes {
    pub fn as_ref(&self) -> SampleBytesRef<'_> {
        SampleBytesRef {
            header: &self.header,
            payload: &self.payload,
        }
    }

    pub fn as_mut(&mut self) -> SampleBytesRefMut<'_> {
        SampleBytesRefMut {
            header: &mut self.header,
            payload: &mut self.payload,
        }
    }
}

/// A sample of the local service that holds no memory yet.
pub trait LoanableSample {
    /// The loaned sample whose payload is written from its front.
    type ForwardWritableSample: WritableSample<InitializedSample = Self::InitializedSample>
        + ForwardRegion;
    /// The loaned sample whose payload is written from its end.
    type BackwardWritableSample: WritableSample<InitializedSample = Self::InitializedSample>
        + BackwardRegion;
    /// The loaned sample once its header and payload are written.
    type InitializedSample;

    /// Acquire a loan for the sample with a payload of `payload_len` bytes
    /// written from its front.
    ///
    /// Fails with:
    ///
    /// * `Malformed` if `payload_len` is an invalid length for a sample of
    ///   this service
    /// * `Exhausted` if no sample could be loaned
    fn loan_forward(self, payload_len: usize) -> Result<Self::ForwardWritableSample, LoanError>;

    /// Acquire a loan for the sample with a payload of `payload_len` bytes
    /// written from its end.
    ///
    /// Fails with:
    ///
    /// * `Malformed` if `payload_len` is an invalid length for a sample of
    ///   this service
    /// * `Exhausted` if no sample could be loaned
    /// * `DirectionUnsupported` if the payload of this service cannot be
    ///   written from its end
    fn loan_backward(self, payload_len: usize) -> Result<Self::BackwardWritableSample, LoanError>;
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
    /// The payload of the service cannot be written from the requested
    /// direction.
    DirectionUnsupported,
}

impl core::fmt::Display for LoanError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "LoanError::{self:?}")
    }
}

impl core::error::Error for LoanError {}

impl From<LoanError> for ResizeError {
    fn from(refusal: LoanError) -> Self {
        match refusal {
            LoanError::Malformed => ResizeError::UnsupportedLength,
            LoanError::NotResizable => ResizeError::NotResizable,
            LoanError::Exhausted => ResizeError::Exhausted,
            LoanError::DirectionUnsupported => ResizeError::DirectionUnsupported,
        }
    }
}

/// Provides the locations to write a sample's header and payload.
pub trait WritableSample {
    /// The sample once its header and payload are written.
    type InitializedSample;

    /// Retrieve a reference to the bytes of the header and payload.
    fn as_ref(&self) -> SampleBytesRef<'_>;

    /// The locations into which the header and the payload bytes should
    /// be written.
    fn as_mut(&mut self) -> SampleBytesRefMut<'_>;

    /// The sample as initialized.
    ///
    /// # Safety
    ///
    /// The header and the payload must both have been written.
    unsafe fn assume_init(self) -> Self::InitializedSample;
}

impl Deref for SampleBytes {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        &self.payload
    }
}

impl DerefMut for SampleBytes {
    fn deref_mut(&mut self) -> &mut [u8] {
        &mut self.payload
    }
}

impl ForwardRegion for SampleBytes {
    fn resize(&mut self, len: usize) -> Result<(), ResizeError> {
        self.payload.resize(len, 0);
        Ok(())
    }
}

impl BackwardRegion for SampleBytes {
    fn resize(&mut self, len: usize) -> Result<(), ResizeError> {
        let current = self.payload.len();
        if len > current {
            self.payload.resize(len, 0);
            self.payload.rotate_right(len - current);
        } else {
            self.payload.drain(..current - len);
        }
        Ok(())
    }
}

impl WritableSample for SampleBytes {
    type InitializedSample = SampleBytes;

    fn as_ref(&self) -> SampleBytesRef<'_> {
        SampleBytes::as_ref(self)
    }

    fn as_mut(&mut self) -> SampleBytesRefMut<'_> {
        SampleBytes::as_mut(self)
    }

    unsafe fn assume_init(self) -> SampleBytes {
        self
    }
}

/// Whether a header and a payload of these lengths fit a sample of the
/// described service.
pub fn fits(types: &SampleTypes, header: usize, payload: usize) -> bool {
    if header != types.user_header.size {
        return false;
    }

    // A flatbuffer holds any number of serialized bytes.
    if let TypeIdentifier::Flatbuffer(_) = types.payload.identifier {
        return true;
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

#[cfg(test)]
mod tests {
    use super::*;

    use iceoryx2::service::static_config::message_type_details::TypeDetail;
    use iceoryx2_bb_testing::assert_that;

    use crate::service_description::{Schema, TypeDescription};

    fn no_user_header() -> TypeDescription {
        TypeDescription::from(&TypeDetail::new::<()>(TypeVariant::FixedSize))
    }

    #[test]
    fn sample_bytes_resized_forward_keep_their_payload_at_the_front() {
        const WRITTEN: &[u8] = b"abcd";
        const GROWN: usize = 8;
        const SHRUNK: usize = 3;

        let mut sample = SampleBytes::default();
        ForwardRegion::resize(&mut sample, WRITTEN.len()).expect("heap buffers resize");
        sample.copy_from_slice(WRITTEN);

        ForwardRegion::resize(&mut sample, GROWN).expect("heap buffers resize");
        assert_that!(&sample[..WRITTEN.len()], eq WRITTEN);

        ForwardRegion::resize(&mut sample, SHRUNK).expect("heap buffers resize");
        assert_that!(sample.payload, eq WRITTEN[..SHRUNK].to_vec());
    }

    #[test]
    fn sample_bytes_resized_backward_keep_their_payload_at_the_end() {
        const WRITTEN: &[u8] = b"wxyz";
        const GROWN: usize = 8;
        const SHRUNK: usize = 3;

        let mut sample = SampleBytes::default();
        BackwardRegion::resize(&mut sample, WRITTEN.len()).expect("heap buffers resize");
        sample.copy_from_slice(WRITTEN);

        BackwardRegion::resize(&mut sample, GROWN).expect("heap buffers resize");
        assert_that!(&sample[GROWN - WRITTEN.len()..], eq WRITTEN);

        BackwardRegion::resize(&mut sample, SHRUNK).expect("heap buffers resize");
        assert_that!(sample.payload, eq WRITTEN[WRITTEN.len() - SHRUNK..].to_vec());
    }

    #[test]
    fn a_flatbuffer_payload_fits_any_number_of_bytes() {
        const SCHEMA: &[u8] = b"the binary schema";
        const LENGTHS: [usize; 3] = [0, 1, 1234];

        let types = SampleTypes {
            payload: TypeDescription {
                variant: TypeVariant::FixedSize,
                identifier: TypeIdentifier::Flatbuffer(Schema::new(SCHEMA.to_vec())),
                size: 1,
                alignment: 1,
            },
            user_header: no_user_header(),
        };

        for length in LENGTHS {
            assert_that!(fits(&types, 0, length), eq true);
        }
        assert_that!(fits(&types, 1, LENGTHS[2]), eq false);
    }
}
