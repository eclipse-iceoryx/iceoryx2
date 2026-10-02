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

//! Manages loaning the samples of a local publish-subscribe service being
//! relayed to the backend.
//!
//! The types here implement the state machine of [`crate::wire::sample`] for
//! the one sample type a publish-subscribe service has.
//!
//! ```text
//!                                 loan_forward(len)      ┌────────────────────────┐   resize(len)
//!                            ┌──────────────────────────▶│ LoanedSample           │ ────────────────▶ currently not supported and is refused
//!                            │   plain or flatbuffer     │   impl WritableSample  │   as_mut()
//!   ┌────────────────────────┐                           │   impl ForwardRegion   │ ────────────────▶ writable header and payload
//!   │ UnloanedSample         │                           ├────────────────────────┤   assume_init()
//!   │   impl LoanableSample  │                           │ payload with a fixed   │ ────────────────▶ the initialized, sendable sample
//!   ├────────────────────────┤                           │ length, written        │
//!   │ no memory yet          │                           │ forward from the       │
//!   └────────────────────────┘                           │ beginning of the buffer│
//!                            │                           └────────────────────────┘
//!                            │                           ┌────────────────────────┐   resize(len)
//!                            │   loan_backward(len)      │ LoanedFlatbufferSample │ ────────────────▶ the payload grown from its end
//!                            └──────────────────────────▶│   impl WritableSample  │   as_mut()
//!                                flatbuffer only         │   impl BackwardRegion  │ ────────────────▶ writable header and payload
//!                                                        ├────────────────────────┤   assume_init()
//!                                                        │ payload with a growing │ ────────────────▶ the initialized, sendable sample
//!                                                        │ length, written        │
//!                                                        │ backward from the end  │
//!                                                        │ of the buffer          │
//!                                                        └────────────────────────┘
//! ```

use core::ops::{Deref, DerefMut, Range};

use iceoryx2::service::Service;
use iceoryx2_log::{fail, origin};

use crate::service_description::{SampleTypes, TypeIdentifier};
use crate::wire::region::{BackwardRegion, ForwardRegion, ResizeError};
use crate::wire::sample::{
    Header, LoanError, LoanableSample, Payload, PayloadUninit, SampleBytesRef, SampleBytesRefMut,
    WritableSample, fits, user_header_bytes,
};

pub type Sample<S> = iceoryx2::sample::Sample<S, Payload, Header>;
pub type SampleMut<S> = iceoryx2::sample_mut::SampleMut<S, Payload, Header>;
pub type SampleMutUninit<S> =
    iceoryx2::sample_mut_uninit::SampleMutUninit<S, PayloadUninit, Header>;
pub type FlatbufferMemory<S> = iceoryx2::sample_mut_uninit::FlatbufferMemory<S>;

pub type Publisher<S> = iceoryx2::port::publisher::Publisher<S, Payload, Header>;
pub type Subscriber<S> = iceoryx2::port::subscriber::Subscriber<S, Payload, Header>;

/// Loans an uninitialized sample of the given payload size from the
/// local publisher.
pub type LoanFn<'a, S, E> = dyn FnMut(usize) -> Result<SampleMutUninit<S>, E> + 'a;

/// Views a loaned sample's user header as writable bytes.
///
/// # Safety
///
/// `size` must be the user header size of the service the sample belongs
/// to, as its description states.
pub unsafe fn user_header_bytes_mut<S: Service>(
    sample: &mut SampleMutUninit<S>,
    size: usize,
) -> &mut [u8] {
    unsafe {
        core::slice::from_raw_parts_mut(sample.user_header_mut() as *mut Header as *mut u8, size)
    }
}

/// View of the initialized bytes of a sample's payload.
pub fn payload_bytes<S: Service>(sample: &Sample<S>) -> &[u8] {
    let payload = sample.payload();
    // SAFETY: the payload marker is one byte with no padding, so a slice
    // of markers is a slice of bytes of the same length.
    let bytes =
        unsafe { core::slice::from_raw_parts(payload.as_ptr() as *const u8, payload.len()) };
    &bytes[sample.header().payload_offset() as usize..]
}

/// View of the uninitialized bytes of a sample's payload.
fn payload_bytes_uninit<S: Service>(sample: &SampleMutUninit<S>) -> &[u8] {
    let payload = sample.payload();
    // SAFETY: the payload marker is one byte with no padding, so a slice
    // of markers is a slice of bytes of the same length.
    unsafe { core::slice::from_raw_parts(payload.as_ptr() as *const u8, payload.len()) }
}

pub fn payload_bytes_mut<S: Service>(sample: &mut SampleMutUninit<S>) -> &mut [u8] {
    let payload = sample.payload_mut();
    // SAFETY: the payload marker is one byte with no padding, so a slice
    // of markers is a slice of bytes of the same length.
    unsafe { core::slice::from_raw_parts_mut(payload.as_mut_ptr() as *mut u8, payload.len()) }
}

/// A sample for a given type that is not yet loaned.
///
/// A forward loan is possible for every payload type. A backward loan is
/// refused for anything but a flatbuffer, since currently only a flatbuffer
/// payload may start at an offset into the sample.
pub struct UnloanedSample<'a, 'b, S: Service, E> {
    types: &'a SampleTypes,
    loan: &'a mut LoanFn<'b, S, E>,
}

impl<'a, 'b, S: Service, E> UnloanedSample<'a, 'b, S, E> {
    pub fn new(types: &'a SampleTypes, loan: &'a mut LoanFn<'b, S, E>) -> Self {
        Self { types, loan }
    }

    fn loan(self, payload_len: usize) -> Result<LoanedSample<S>, LoanError> {
        let origin = origin!("UnloanedSample::loan");

        // The length has to be one the payload type of the service allows.
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
        // TODO: Resize the loaned sample instead, once slice samples can be
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

    fn loan_flatbuffer(self, payload_len: usize) -> Result<LoanedFlatbufferSample<S>, LoanError> {
        let origin = origin!("UnloanedSample::loan_flatbuffer");

        // A flatbuffer is loaned as one element.
        let sample = fail!(
            from origin,
            when (self.loan)(1),
            with LoanError::Exhausted,
            "Failed to loan a sample for a flatbuffer"
        );

        // The memory of the sample grows from its end.
        let memory = sample.__internal_create_resizable_memory();
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

impl<S: Service, E> LoanableSample for UnloanedSample<'_, '_, S, E> {
    type ForwardWritableSample = LoanedSample<S>;
    type BackwardWritableSample = LoanedFlatbufferSample<S>;
    type InitializedSample = SampleMut<S>;

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
pub struct LoanedSample<S: Service> {
    header_size: usize,
    sample: SampleMutUninit<S>,
    payload_range: Range<usize>,
}

impl<S: Service> Deref for LoanedSample<S> {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        self.as_ref().payload
    }
}

impl<S: Service> DerefMut for LoanedSample<S> {
    fn deref_mut(&mut self) -> &mut [u8] {
        self.as_mut().payload
    }
}

impl<S: Service> ForwardRegion for LoanedSample<S> {
    fn resize(&mut self, len: usize) -> Result<(), ResizeError> {
        let origin = origin!("LoanedSample::resize");

        // TODO: Grow the loaned sample instead, once samples can be resized
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

impl<S: Service> WritableSample for LoanedSample<S> {
    type InitializedSample = SampleMut<S>;

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
        // sample, and the result borrows `self` for as long as they are held,
        // so no other access to the sample can alias them.
        unsafe {
            SampleBytesRefMut {
                header: &mut *header,
                payload: &mut *payload,
            }
        }
    }

    unsafe fn assume_init(self) -> SampleMut<S> {
        // SAFETY: the caller wrote the header and the payload.
        unsafe { self.sample.assume_init() }
    }
}

/// A sample loaned for a flatbuffer written from its end.
///
/// Growing the payload moves the sample in shared memory. Every resize moves
/// the sample along with its memory, so `as_mut` stays valid.
pub struct LoanedFlatbufferSample<S: Service> {
    header_size: usize,
    sample: SampleMutUninit<S>,
    memory: FlatbufferMemory<S>,
    payload_len: usize,
}

impl<S: Service> LoanedFlatbufferSample<S> {
    fn payload_offset(&self) -> usize {
        self.memory.len() - self.payload_len
    }
}

impl<S: Service> From<LoanedFlatbufferSample<S>> for LoanedSample<S> {
    fn from(flatbuffer: LoanedFlatbufferSample<S>) -> Self {
        let payload_range = flatbuffer.payload_offset()..flatbuffer.memory.len();

        LoanedSample {
            header_size: flatbuffer.header_size,
            sample: flatbuffer.sample,
            payload_range,
        }
    }
}

impl<S: Service> Deref for LoanedFlatbufferSample<S> {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        &self.memory[self.payload_offset()..]
    }
}

impl<S: Service> DerefMut for LoanedFlatbufferSample<S> {
    fn deref_mut(&mut self) -> &mut [u8] {
        let payload = self.payload_offset();
        &mut self.memory[payload..]
    }
}

impl<S: Service> BackwardRegion for LoanedFlatbufferSample<S> {
    fn resize(&mut self, len: usize) -> Result<(), ResizeError> {
        let origin = origin!("LoanedFlatbufferSample::resize");

        if len > self.memory.len() {
            fail!(
                from origin,
                when self.memory.grow_downwards_with_size(len, 0),
                with ResizeError::Exhausted,
                "Failed to grow the payload to {} bytes", len
            );
        }
        self.payload_len = len;
        let payload = self.payload_offset();
        self.sample
            .__internal_finish_serialized(self.memory[payload..].as_ptr());

        Ok(())
    }
}

impl<S: Service> WritableSample for LoanedFlatbufferSample<S> {
    type InitializedSample = SampleMut<S>;

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

    unsafe fn assume_init(self) -> SampleMut<S> {
        // SAFETY: the caller wrote the header and the payload.
        unsafe { self.sample.assume_init() }
    }
}
