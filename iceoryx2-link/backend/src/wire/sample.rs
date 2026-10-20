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
//!
//! The state machine is implemented once over the [`UninitializedSample`]
//! trait. The trait is implemented for every sample kind i.e. the
//! publish-subscribe sample, the request and the response.
//!
//! ```text
//!                                 loan_forward(len)      ┌────────────────────────┐   resize(len)
//!                            ┌──────────────────────────▶│ LoanedSample           │ ────────────────▶ currently not supported and is refused
//!                            │   plain or flatbuffer     │   impl WritableSample  │   as_mut()
//!   ┌────────────────────────┐                           │   impl ForwardRegion   │ ────────────────▶ writable header and payload
//!   │ UnloanedSample         │                           ├────────────────────────┤   assume_init()
//!   │   impl LoanableSample  │                           │ an UninitializedSample │ ────────────────▶ UninitializedSample::InitializedSample,
//!   ├────────────────────────┤                           │ with a fixed payload   │                   the sendable sample
//!   │ no memory yet          │                           │ length, written forward│
//!   └────────────────────────┘                           │ from the buffer start  │
//!                            │                           └────────────────────────┘
//!                            │                           ┌────────────────────────┐   resize(len)
//!                            │   loan_backward(len)      │ LoanedFlatbufferSample │ ────────────────▶ the payload grown from its end
//!                            └──────────────────────────▶│   impl WritableSample  │   as_mut()
//!                                flatbuffer only         │   impl BackwardRegion  │ ────────────────▶ writable header and payload
//!                                                        ├────────────────────────┤   assume_init()
//!                                                        │ an UninitializedSample │ ────────────────▶ UninitializedSample::InitializedSample,
//!                                                        │ with a growing payload │                   the sendable sample
//!                                                        │ length, written        │
//!                                                        │ backward from the end  │
//!                                                        └────────────────────────┘
//! ```

use alloc::vec::Vec;
use core::marker::PhantomData;
use core::mem::MaybeUninit;
use core::ops::{Deref, DerefMut, Range};

use iceoryx2::service::marker::{CustomHeaderMarker, CustomPayloadMarker};
use iceoryx2::service::static_config::message_type_details::TypeVariant;
use iceoryx2_log::{fail, origin};

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

/// A loaned sample whose header and payload are not yet written. The
/// kinds are the publish-subscribe sample, the request and the response.
pub trait UninitializedSample {
    /// The sample once its header and payload are written.
    type InitializedSample;
    /// The memory a flatbuffer payload grows in.
    type FlatbufferMemory: FlatbufferMemory;

    fn user_header(&self) -> &Header;

    fn user_header_mut(&mut self) -> &mut Header;

    fn payload(&self) -> &PayloadUninit;

    fn payload_mut(&mut self) -> &mut PayloadUninit;

    /// The memory a flatbuffer payload is built in. It grows from its end.
    fn flatbuffer_memory(&self) -> Self::FlatbufferMemory;

    /// Records where the serialized payload starts within the memory.
    fn finish_serialized(&mut self, payload: *const u8);

    /// The sample as initialized.
    ///
    /// # Safety
    ///
    /// The header and the payload must both have been written.
    unsafe fn assume_init(self) -> Self::InitializedSample;
}

/// Memory a flatbuffer payload is written into from its end.
pub trait FlatbufferMemory: Deref<Target = [u8]> + DerefMut {
    /// Grows the memory to `len` bytes. Its content stays at the end.
    ///
    /// Fails with `Exhausted` when no larger memory can be provided.
    fn grow(&mut self, len: usize) -> Result<(), ResizeError>;
}

/// Loans an uninitialized sample of the given payload size from the
/// local publisher.
pub type LoanFn<'a, U, E> = dyn FnMut(usize) -> Result<U, E> + 'a;

/// A sample for a given type that is not yet loaned.
///
/// A forward loan is possible for every payload type. A backward loan is
/// refused for anything but a flatbuffer. Only a flatbuffer payload may
/// start at an offset into the sample.
pub struct UnloanedSample<'a, U: UninitializedSample, E, L = &'a mut LoanFn<'a, U, E>> {
    types: &'a SampleTypes,
    loan: L,
    _loaned: PhantomData<fn() -> Result<U, E>>,
}

impl<'a, U: UninitializedSample, E, L: FnOnce(usize) -> Result<U, E>> UnloanedSample<'a, U, E, L> {
    pub fn new(types: &'a SampleTypes, loan: L) -> Self {
        Self {
            types,
            loan,
            _loaned: PhantomData,
        }
    }

    fn loan(self, payload_len: usize) -> Result<LoanedSample<U>, LoanError> {
        let origin = origin!("UnloanedSample::loan");

        // Check that the length requested fits the constraints of the payload
        // type description.
        if !fits(self.types, self.types.user_header.size, payload_len) {
            fail!(
                from origin,
                with LoanError::Malformed,
                "A payload of {} bytes does not fit the service description", payload_len
            );
        }

        // The sample is loaned for the requested length.
        let mut sample = fail!(
            from origin,
            when (self.loan)(payload_len),
            with LoanError::Exhausted,
            "Failed to loan a sample for a payload of {} bytes", payload_len
        );

        // The loaned payload has to be exactly that long.
        // TODO: Resize the loaned sample instead once slice samples can be
        // resized through the publisher's allocation strategy.
        let payload = payload_bytes_mut(&mut sample);
        if payload.len() != payload_len {
            fail!(
                from origin,
                with LoanError::NotResizable,
                "The sample holds a payload of {} bytes, not {}", payload.len(), payload_len
            );
        }

        Ok(LoanedSample {
            header_size: self.types.user_header.size,
            sample,
            payload_range: 0..payload_len,
        })
    }

    fn loan_flatbuffer(self, payload_len: usize) -> Result<LoanedFlatbufferSample<U>, LoanError> {
        let origin = origin!("UnloanedSample::loan_flatbuffer");

        // A flatbuffer is loaned as one element.
        let sample = fail!(
            from origin,
            when (self.loan)(1),
            with LoanError::Exhausted,
            "Failed to loan a sample for a flatbuffer"
        );

        // The memory of the sample grows from its end.
        let memory = sample.flatbuffer_memory();
        let mut loaned = LoanedFlatbufferSample {
            header_size: self.types.user_header.size,
            sample,
            memory,
            payload_len: 0,
        };

        // The payload takes the requested length.
        fail!(
            from origin,
            when loaned.resize(payload_len),
            with LoanError::Exhausted,
            "Failed to grow the loaned sample to a payload of {} bytes", payload_len
        );

        Ok(loaned)
    }
}

impl<U: UninitializedSample, E, L: FnOnce(usize) -> Result<U, E>> LoanableSample
    for UnloanedSample<'_, U, E, L>
{
    type ForwardWritableSample = LoanedSample<U>;
    type BackwardWritableSample = LoanedFlatbufferSample<U>;
    type InitializedSample = U::InitializedSample;

    fn loan_forward(self, payload_len: usize) -> Result<Self::ForwardWritableSample, LoanError> {
        match self.types.payload.identifier {
            // A flatbuffer loaned forward keeps the length it is loaned with.
            TypeIdentifier::Flatbuffer(_) => Ok(self.loan_flatbuffer(payload_len)?.into()),
            _ => self.loan(payload_len),
        }
    }

    fn loan_backward(self, payload_len: usize) -> Result<Self::BackwardWritableSample, LoanError> {
        let origin = origin!("UnloanedSample::loan_backward");

        match self.types.payload.identifier {
            TypeIdentifier::Flatbuffer(_) => self.loan_flatbuffer(payload_len),
            // Only a flatbuffer is written from its end.
            _ => {
                fail!(
                    from origin,
                    with LoanError::DirectionUnsupported,
                    "The payload of the service cannot be written from its end"
                );
            }
        }
    }
}

/// A sample loaned for a payload written from its front.
pub struct LoanedSample<U: UninitializedSample> {
    header_size: usize,
    sample: U,
    payload_range: Range<usize>,
}

impl<U: UninitializedSample> Deref for LoanedSample<U> {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        self.as_ref().payload
    }
}

impl<U: UninitializedSample> DerefMut for LoanedSample<U> {
    fn deref_mut(&mut self) -> &mut [u8] {
        self.as_mut().payload
    }
}

impl<U: UninitializedSample> ForwardRegion for LoanedSample<U> {
    fn resize(&mut self, len: usize) -> Result<(), ResizeError> {
        let origin = origin!("LoanedSample::resize");

        // TODO: Grow the loaned sample instead once samples can be resized
        // through the publisher's allocation strategy.
        let current = self.payload_range.len();
        if current != len {
            fail!(
                from origin,
                with ResizeError::NotResizable,
                "The sample holds a payload of {} bytes, not {}", current, len
            );
        }

        Ok(())
    }
}

impl<U: UninitializedSample> WritableSample for LoanedSample<U> {
    type InitializedSample = U::InitializedSample;

    fn as_ref(&self) -> SampleBytesRef<'_> {
        // SAFETY: the header size for this service is provided by the
        // service's own description.
        let header = unsafe { user_header_bytes(self.sample.user_header(), self.header_size) };
        let payload = payload_bytes_uninit(&self.sample);

        SampleBytesRef {
            header,
            payload: &payload[self.payload_range.clone()],
        }
    }

    fn as_mut(&mut self) -> SampleBytesRefMut<'_> {
        // SAFETY: the header size for this service is provided by the
        // service's own description.
        let header: *mut [u8] =
            unsafe { user_header_bytes_mut(&mut self.sample, self.header_size) };
        let payload: *mut [u8] =
            &mut payload_bytes_mut(&mut self.sample)[self.payload_range.clone()];

        // SAFETY: the header and the payload are disjoint regions of the
        // sample. The result borrows `self` for as long as they are held, so
        // nothing else can alias them.
        unsafe {
            SampleBytesRefMut {
                header: &mut *header,
                payload: &mut *payload,
            }
        }
    }

    unsafe fn assume_init(self) -> U::InitializedSample {
        // SAFETY: the caller wrote the header and the payload.
        unsafe { self.sample.assume_init() }
    }
}

/// A sample loaned for a flatbuffer written from its end.
///
/// Growing the payload moves the sample in shared memory. Every resize moves
/// the sample along with its memory, so `as_mut` stays valid.
pub struct LoanedFlatbufferSample<U: UninitializedSample> {
    header_size: usize,
    sample: U,
    memory: U::FlatbufferMemory,
    payload_len: usize,
}

impl<U: UninitializedSample> LoanedFlatbufferSample<U> {
    fn payload_offset(&self) -> usize {
        self.memory.len() - self.payload_len
    }
}

impl<U: UninitializedSample> From<LoanedFlatbufferSample<U>> for LoanedSample<U> {
    fn from(flatbuffer: LoanedFlatbufferSample<U>) -> Self {
        let payload_range = flatbuffer.payload_offset()..flatbuffer.memory.len();

        LoanedSample {
            header_size: flatbuffer.header_size,
            sample: flatbuffer.sample,
            payload_range,
        }
    }
}

impl<U: UninitializedSample> Deref for LoanedFlatbufferSample<U> {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        &self.memory[self.payload_offset()..]
    }
}

impl<U: UninitializedSample> DerefMut for LoanedFlatbufferSample<U> {
    fn deref_mut(&mut self) -> &mut [u8] {
        let payload = self.payload_offset();
        &mut self.memory[payload..]
    }
}

impl<U: UninitializedSample> BackwardRegion for LoanedFlatbufferSample<U> {
    fn resize(&mut self, len: usize) -> Result<(), ResizeError> {
        let origin = origin!("LoanedFlatbufferSample::resize");

        if len > self.memory.len() {
            fail!(
                from origin,
                when self.memory.grow(len),
                "Failed to grow the payload to {} bytes", len
            );
        }
        self.payload_len = len;
        let payload = self.payload_offset();
        self.sample
            .finish_serialized(self.memory[payload..].as_ptr());

        Ok(())
    }
}

impl<U: UninitializedSample> WritableSample for LoanedFlatbufferSample<U> {
    type InitializedSample = U::InitializedSample;

    fn as_ref(&self) -> SampleBytesRef<'_> {
        SampleBytesRef {
            // SAFETY: the header size for this service is provided by the
            // service's own description.
            header: unsafe { user_header_bytes(self.sample.user_header(), self.header_size) },
            payload: &self.memory[self.payload_offset()..],
        }
    }

    fn as_mut(&mut self) -> SampleBytesRefMut<'_> {
        let payload = self.payload_offset();
        SampleBytesRefMut {
            // SAFETY: the header size for this service is provided by the
            // service's own description.
            header: unsafe { user_header_bytes_mut(&mut self.sample, self.header_size) },
            payload: &mut self.memory[payload..],
        }
    }

    unsafe fn assume_init(self) -> U::InitializedSample {
        // SAFETY: the caller wrote the header and the payload.
        unsafe { self.sample.assume_init() }
    }
}

/// Whether a header and a payload of these lengths fit a sample of the
/// described service, depending on whether its payload type is a fixed-size,
/// dynamic or flatbuffer payload type.
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
/// to as per the service's description.
pub unsafe fn user_header_bytes(user_header: &Header, size: usize) -> &[u8] {
    unsafe { core::slice::from_raw_parts(user_header as *const Header as *const u8, size) }
}

/// Views a loaned sample's user header as writable bytes.
///
/// # Safety
///
/// `size` must be the user header size of the service the sample belongs
/// to as per the service's description.
pub unsafe fn user_header_bytes_mut<U: UninitializedSample>(
    sample: &mut U,
    size: usize,
) -> &mut [u8] {
    unsafe {
        core::slice::from_raw_parts_mut(sample.user_header_mut() as *mut Header as *mut u8, size)
    }
}

/// View of the uninitialized bytes of a sample's payload.
fn payload_bytes_uninit<U: UninitializedSample>(sample: &U) -> &[u8] {
    let payload = sample.payload();
    // SAFETY: the payload marker is one byte with no padding, so a slice
    // of markers is a slice of bytes of the same length.
    unsafe { core::slice::from_raw_parts(payload.as_ptr() as *const u8, payload.len()) }
}

pub fn payload_bytes_mut<U: UninitializedSample>(sample: &mut U) -> &mut [u8] {
    let payload = sample.payload_mut();
    // SAFETY: the payload marker is one byte with no padding, so a slice
    // of markers is a slice of bytes of the same length.
    unsafe { core::slice::from_raw_parts_mut(payload.as_mut_ptr() as *mut u8, payload.len()) }
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

/// Tests the handling of uninitialized memory over heap memory, which
/// is tracked by miri.
///
/// A shared-memory chunk cannot be validated by miri, but heap memory is the
/// stricter case. Its bytes are uninitialized where a shared-memory chunk's
/// already hold values.
///
/// Run them under miri with:
///
/// ```sh
/// cargo +nightly miri test -p iceoryx2-link-backend uninitialized_memory
/// ```
#[cfg(test)]
mod uninitialized_memory_tests {
    use super::*;

    use alloc::boxed::Box;
    use alloc::string::String;
    use core::convert::Infallible;

    use iceoryx2::service::static_config::message_type_details::TypeDetail;
    use iceoryx2_bb_testing::assert_that;

    use crate::service_description::TypeDescription;

    /// A sample over heap memory in the state iceoryx2 loans them.
    /// The header is initialized and the payload is [`MaybeUninit`].
    struct HeapSample {
        header: Header,
        payload: Box<PayloadUninit>,
    }

    impl HeapSample {
        fn uninit(len: usize) -> Self {
            Self {
                header: Header::default(),
                payload: Box::new_uninit_slice(len),
            }
        }
    }

    /// No flatbuffer is built in a heap sample.
    struct NoFlatbufferMemory;

    impl Deref for NoFlatbufferMemory {
        type Target = [u8];

        fn deref(&self) -> &[u8] {
            &[]
        }
    }

    impl DerefMut for NoFlatbufferMemory {
        fn deref_mut(&mut self) -> &mut [u8] {
            &mut []
        }
    }

    impl FlatbufferMemory for NoFlatbufferMemory {
        fn grow(&mut self, _: usize) -> Result<(), ResizeError> {
            Err(ResizeError::Exhausted)
        }
    }

    impl UninitializedSample for HeapSample {
        type InitializedSample = Box<Payload>;
        type FlatbufferMemory = NoFlatbufferMemory;

        fn user_header(&self) -> &Header {
            &self.header
        }

        fn user_header_mut(&mut self) -> &mut Header {
            &mut self.header
        }

        fn payload(&self) -> &PayloadUninit {
            &self.payload
        }

        fn payload_mut(&mut self) -> &mut PayloadUninit {
            &mut self.payload
        }

        fn flatbuffer_memory(&self) -> NoFlatbufferMemory {
            NoFlatbufferMemory
        }

        fn finish_serialized(&mut self, _: *const u8) {}

        unsafe fn assume_init(self) -> Box<Payload> {
            unsafe { self.payload.assume_init() }
        }
    }

    /// The loan forms byte views over the payload before it is written.
    #[test]
    fn a_forward_loan_over_uninitialized_memory_is_written_before_it_is_read() {
        const PAYLOAD: &[u8] = b"written";

        let types = SampleTypes {
            payload: TypeDescription {
                variant: TypeVariant::Dynamic,
                identifier: TypeIdentifier::Name(String::from("u8")),
                size: 1,
                alignment: 1,
            },
            user_header: TypeDescription::from(&TypeDetail::new::<()>(TypeVariant::FixedSize)),
        };
        let unloaned =
            UnloanedSample::new(&types, |len| Ok::<_, Infallible>(HeapSample::uninit(len)));

        let mut writable = unloaned
            .loan_forward(PAYLOAD.len())
            .expect("the sample is loaned");
        writable.as_mut().payload.copy_from_slice(PAYLOAD);
        assert_that!(writable.as_ref().payload, eq PAYLOAD);

        // SAFETY: the header is written by the loan and the payload above.
        let initialized = unsafe { writable.assume_init() };
        assert_that!(initialized.len(), eq PAYLOAD.len());
    }
}
