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

//! Transcodes samples between the local service and the middleware's forms.
//!
//! A sample received from the local service is either passed directly to the
//! endpoint, or transcoded into a scratch, which is passed instead.
//!
//! ```text
//! ── Transcoding::encode ───────────────────────────────────────────────────
//!
//!   sample from the local port
//!            │
//!            │
//!     SampleBytesRef
//!            │
//!            ├── as is ────────────────────────────────────────▶ sent via the endpoint
//!            │
//!            │               ┌────────────────────────┐
//!            └── encoded ──▶ │ Transcoding::scratch   │ ───────▶ sent via the endpoint
//!                            │   SampleBytes          │
//!                            │     header: Vec<u8>    │
//!                            │       impl Region      │
//!                            │     payload: Vec<u8>   │
//!                            │       impl Region      │
//!                            └────────────────────────┘
//! ```
//!
//! A message received from the middleware is either written directly into a
//! sample loaned from the local service, or into a scratch buffer and then
//! decoded into the sample.
//!
//! Both paths write through a pending loan, which acquires the loan once the
//! payload length and the direction it is written in are known.
//!
//! ```text
//! ── Transcoding::decode ───────────────────────────────────────────────────
//!
//!                         input message from the endpoint
//!                                     │
//!                                     │
//!                 ┌───────────────────┴────────────────────┐
//!               as-is                                   decoded
//!                 │                                        │
//!                 ▼                                        ▼
//!     ┌────────────────────────┐               ┌────────────────────────┐
//!     │ Destination::Loan      │               │ Destination::Scratch   │
//!     │   LoanBackedDestination│               │   Transcoding::scratch │
//!     │   impl TakeDestination │               │   impl TakeDestination │
//!     │ Destination::Split     │               └────────────────────────┘
//!     │   SplitDestination     │                            │
//!     │   impl TakeDestination │                            │
//!     └────────────────────────┘                            │
//!                 │                                         │
//!   the endpoint fills the sample            the endpoint fills the scratch
//!                 │                                         │
//!                 │                                         │
//!                 │                          ┌──────────────────────────┐
//!                 │                          │ LoanBackedRegion         │
//!                 │                          │   impl ForwardRegion     │
//!                 │                          │   impl BackwardRegion    │
//!                 │                          └──────────────────────────┘
//!                 │                                        │
//!                 │                          a transcoder fills the sample
//!                 └───────────────────┬────────────────────┘
//!                                     │
//!                                     │  loans and fills
//!                                     │
//!                                     │
//!                                     ▼                   sized forward   ┌─────────────────────────────────────────┐  resized
//!                         ┌────────────────────────┐   ┌────────────────▶ │ PendingLoan::Loaned                     │ ◀──┐
//!                         │ PendingLoan::Unloaned  │   │                  │   LoanedWritableSample::WrittenForward  │ ───┘
//!                         │   impl LoanableSample  │───┤                  │     impl WritableSample                 │ ──as_mut()──▶ SampleBytesRefMut ─▶ fixed location to write the header and payload
//!                         └────────────────────────┘   │                  │     impl ForwardRegion                  │ ──forward()──▶ ForwardRegion ─────▶ forward-resizable region to write the payload
//!                                     │                │                  │                                         │ ──assume_init()──▶ SampleMut, the output, sent via the local port
//!                                     │                │                  └─────────────────────────────────────────┘
//!                                     │                │  sized backward  ┌─────────────────────────────────────────┐  resized
//!                                     │                └────────────────▶ │ PendingLoan::Loaned                     │ ◀──┐
//!                                     │                                   │   LoanedWritableSample::WrittenBackward │ ───┘
//!                                     │                                   │     impl WritableSample                 │ ──as_mut()──▶ SampleBytesRefMut ─▶ fixed location to write the header and payload
//!                                     │                                   │     impl BackwardRegion                 │ ──backward()──▶ BackwardRegion ───▶ backward-resizable region to write the payload
//!                                     │                                   │                                         │ ──assume_init()──▶ SampleMut, the output, sent via the local port
//!                                     │                                   └─────────────────────────────────────────┘
//!                                     │  loan refused
//!                                     │                                   ┌─────────────────────────────────────────┐
//!                                     └─────────────────────────────────▶ │ PendingLoan::Refused                    │ ─────────────▶ the receive fails
//!                                                                         │   LoanError                             │
//!                                                                         └─────────────────────────────────────────┘
//! ```

use alloc::vec::Vec;
use core::ops::{Deref, DerefMut};

use iceoryx2_link_adapter::{
    LoanableSample, SampleBytes, SampleBytesRef, SampleBytesRefMut, SampleLengths,
    SampleTranscoders, TakeDestination, TranscodeError, Transcoder, TranscodesSamples,
};
use iceoryx2_link_backend::service_description::SampleTypes;
use iceoryx2_link_backend::wire::region::{BackwardRegion, ForwardRegion, Region, ResizeError};
use iceoryx2_link_backend::wire::sample::WritableSample;
use iceoryx2_log::{fail, fatal_panic, origin};

use crate::relay::{ReceiveError, SendError};

/// Transcodes samples between the local service and the middleware's forms.
pub(super) struct Transcoding<X: TranscodesSamples> {
    transcoders: X,
    header_size: usize,
    /// Reused buffers holding the middleware form of the regions being
    /// transcoded.
    scratch: SampleBytes,
}

impl<X: TranscodesSamples> Transcoding<X> {
    pub(super) fn new(transcoders: X, types: &SampleTypes) -> Self {
        Self {
            transcoders,
            header_size: types.user_header.size,
            scratch: SampleBytes::default(),
        }
    }

    /// Encodes `local` into the middleware's form.
    pub(super) fn encode<'a>(
        &'a mut self,
        local: SampleBytesRef<'a>,
    ) -> Result<SampleBytesRef<'a>, SendError> {
        let scratch = &mut self.scratch;
        Ok(match self.transcoders.for_samples() {
            SampleTranscoders::TranscodeNone => local,
            SampleTranscoders::TranscodeHeader(header_transcoder) => {
                encode_header(header_transcoder, local, &mut scratch.header)?;
                SampleBytesRef {
                    header: &scratch.header,
                    payload: local.payload,
                }
            }
            SampleTranscoders::TranscodePayload(payload_transcoder) => {
                encode_payload(payload_transcoder, local, &mut scratch.payload)?;
                SampleBytesRef {
                    header: local.header,
                    payload: &scratch.payload,
                }
            }
            SampleTranscoders::TranscodeBoth(header_transcoder, payload_transcoder) => {
                encode_header(header_transcoder, local, &mut scratch.header)?;
                encode_payload(payload_transcoder, local, &mut scratch.payload)?;
                scratch.as_ref()
            }
        })
    }

    /// Begins decoding a message into a sample loaned from `loanable`.
    pub(super) fn decode<L: LoanableSample>(&mut self, loanable: L) -> PendingDecode<'_, L, X> {
        PendingDecode {
            loan: PendingLoan::new(loanable),
            transcoding: self,
        }
    }
}

/// A decode of one message into a loaned sample that awaits the message.
pub(super) struct PendingDecode<'t, L: LoanableSample, X: TranscodesSamples> {
    loan: PendingLoan<L>,
    transcoding: &'t mut Transcoding<X>,
}

impl<L: LoanableSample, X: TranscodesSamples> PendingDecode<'_, L, X> {
    /// Where the endpoints take the message to.
    pub(super) fn input(&mut self) -> Destination<'_, '_, L> {
        let scratch = &mut self.transcoding.scratch;
        match self.transcoding.transcoders.for_samples() {
            SampleTranscoders::TranscodeNone => Destination::Loan(LoanBackedDestination::new(
                &mut self.loan,
                self.transcoding.header_size,
            )),
            SampleTranscoders::TranscodeHeader(_) => {
                Destination::Split(SplitDestination::new(&mut self.loan, &mut scratch.header))
            }
            SampleTranscoders::TranscodePayload(_) | SampleTranscoders::TranscodeBoth(_, _) => {
                Destination::Scratch(scratch)
            }
        }
    }

    /// The decoded sample after the message was taken into the input.
    pub(super) fn output(self) -> Result<L::InitializedSample, ReceiveError> {
        let Self {
            mut loan,
            transcoding,
        } = self;
        let wire = transcoding.scratch.as_ref();
        let writable_sample = match transcoding.transcoders.for_samples() {
            SampleTranscoders::TranscodeNone => loan.into_writable_sample(),
            SampleTranscoders::TranscodeHeader(header_transcoder) => {
                let mut writable_sample = loan.into_writable_sample();
                decode_header(header_transcoder, wire, &mut writable_sample)?;
                writable_sample
            }
            SampleTranscoders::TranscodePayload(payload_transcoder) => {
                decode_payload(payload_transcoder, wire, &mut loan)?;
                let mut writable_sample = loan.into_writable_sample();
                copy_header(wire, &mut writable_sample, transcoding.header_size)?;
                writable_sample
            }
            SampleTranscoders::TranscodeBoth(header_transcoder, payload_transcoder) => {
                decode_payload(payload_transcoder, wire, &mut loan)?;
                let mut writable_sample = loan.into_writable_sample();
                decode_header(header_transcoder, wire, &mut writable_sample)?;
                writable_sample
            }
        };

        // SAFETY: the header and the payload were both written into the
        // loaned sample above.
        Ok(unsafe { writable_sample.assume_init() })
    }

    /// The error a declined take stands for.
    pub(super) fn refusal(self) -> ReceiveError {
        let origin = origin!("PendingDecode::refusal");
        match self.transcoding.transcoders.for_samples() {
            SampleTranscoders::TranscodeNone | SampleTranscoders::TranscodeHeader(_) => {
                self.loan.refusal()
            }
            SampleTranscoders::TranscodePayload(_) | SampleTranscoders::TranscodeBoth(_, _) => {
                fatal_panic!(
                    from origin,
                    "The endpoints reported a declined take into the relay's scratch buffers, which accept any length. This cannot occur with correctly implemented endpoints."
                );
            }
        }
    }
}

/// Where a pending decode takes the message to.
pub(super) enum Destination<'l, 's, L: LoanableSample> {
    Loan(LoanBackedDestination<'l, L>),
    Split(SplitDestination<'l, 's, L>),
    Scratch(&'s mut SampleBytes),
}

impl<'a, L: LoanableSample> TakeDestination<'a> for &'a mut Destination<'_, '_, L> {
    fn for_lengths(self, lengths: SampleLengths) -> Option<SampleBytesRefMut<'a>> {
        match self {
            Destination::Loan(destination) => destination.for_lengths(lengths),
            Destination::Split(destination) => destination.for_lengths(lengths),
            Destination::Scratch(scratch) => (&mut **scratch).for_lengths(lengths),
        }
    }
}

/// Encodes the `local` header into `scratch`.
fn encode_header<T: for<'a> Transcoder<SampleBytesRef<'a>>>(
    header_transcoder: &T,
    local: SampleBytesRef<'_>,
    scratch: &mut Vec<u8>,
) -> Result<(), SendError> {
    let origin = origin!("Transcoding::encode_header");

    fail!(
        from origin,
        when header_transcoder.encode(local, scratch),
        with SendError::Transcode,
        "Failed to encode the header of a sample"
    );

    Ok(())
}

/// Encodes the `local` payload into `scratch`.
fn encode_payload<T: for<'a> Transcoder<SampleBytesRef<'a>>>(
    payload_transcoder: &T,
    local: SampleBytesRef<'_>,
    scratch: &mut Vec<u8>,
) -> Result<(), SendError> {
    let origin = origin!("Transcoding::encode_payload");

    fail!(
        from origin,
        when payload_transcoder.encode(local, scratch),
        with SendError::Transcode,
        "Failed to encode the payload of a sample"
    );

    Ok(())
}

/// Decodes the header of `wire` into the loaned sample.
fn decode_header<T: for<'a> Transcoder<SampleBytesRef<'a>>, W: WritableSample>(
    transcoder: &T,
    wire: SampleBytesRef<'_>,
    loaned: &mut W,
) -> Result<(), ReceiveError> {
    let origin = origin!("Transcoding::decode_header");

    let mut header = loaned.as_mut().header;
    match transcoder.decode(wire, &mut header) {
        Ok(()) => Ok(()),
        Err(TranscodeError::Refused) => {
            fail!(
                from origin,
                with ReceiveError::Malformed,
                "The header of a message does not fit the service"
            );
        }
        Err(TranscodeError::Transcoder(_)) => {
            fail!(
                from origin,
                with ReceiveError::Transcode,
                "Failed to decode the header of a message"
            );
        }
    }
}

/// Decodes the payload of `wire` into `loan`, which loans for its size.
fn decode_payload<T: for<'a> Transcoder<SampleBytesRef<'a>>, L: LoanableSample>(
    transcoder: &T,
    wire: SampleBytesRef<'_>,
    loan: &mut PendingLoan<L>,
) -> Result<(), ReceiveError> {
    let origin = origin!("Transcoding::decode_payload");

    match transcoder.decode(wire, loan) {
        Ok(()) => Ok(()),
        Err(TranscodeError::Refused) => {
            fail!(
                from origin,
                with loan.refusal(),
                "The payload of a message was refused"
            );
        }
        Err(TranscodeError::Transcoder(_)) => {
            fail!(
                from origin,
                with ReceiveError::Transcode,
                "Failed to decode the payload of a message"
            );
        }
    }
}

/// Copies the header of `wire` into the loaned sample as it is.
fn copy_header<W: WritableSample>(
    wire: SampleBytesRef<'_>,
    writable_sample: &mut W,
    header_size: usize,
) -> Result<(), ReceiveError> {
    let origin = origin!("Transcoding::copy_header");

    if wire.header.len() != header_size {
        fail!(
            from origin,
            with ReceiveError::Malformed,
            "A header of {} bytes does not fit the service", wire.header.len()
        );
    }
    writable_sample.as_mut().header.copy_from_slice(wire.header);

    Ok(())
}

/// A loan that is deferred until its size is known.
///
/// The size and direction in which the bytes shall be filled must be specified
/// when acquiring the loan.
pub(super) enum PendingLoan<L: LoanableSample> {
    Unloaned(L),
    Loaned(LoanedWritableSample<L>),
    Refused(ReceiveError),
}

impl<L: LoanableSample> PendingLoan<L> {
    pub(super) fn new(loanable: L) -> Self {
        Self::Unloaned(loanable)
    }

    /// Loans for the requested size, or if already loaned, attempts to resize
    /// the loan forward.
    fn loan_or_resize_forward(&mut self, len: usize) -> Result<(), ResizeError> {
        let origin = origin!("PendingLoan::loan_or_resize_forward");

        match core::mem::replace(self, Self::Refused(ReceiveError::Malformed)) {
            Self::Unloaned(loanable) => match loanable.loan_forward(len) {
                Ok(loaned) => {
                    *self = Self::Loaned(LoanedWritableSample::WrittenForward(loaned));
                    Ok(())
                }
                Err(refusal) => {
                    let refusal = ResizeError::from(refusal);
                    *self = Self::Refused(refusal.into());
                    fail!(
                        from origin,
                        with refusal,
                        "Failed to loan a sample for a payload of {} bytes", len
                    );
                }
            },
            Self::Loaned(LoanedWritableSample::WrittenForward(mut loaned)) => {
                match loaned.resize(len) {
                    Ok(()) => {
                        *self = Self::Loaned(LoanedWritableSample::WrittenForward(loaned));
                        Ok(())
                    }
                    Err(refusal) => {
                        *self = Self::Refused(refusal.into());
                        fail!(
                            from origin,
                            with refusal,
                            "The loaned payload cannot be resized to {} bytes", len
                        );
                    }
                }
            }
            Self::Refused(refusal) => {
                *self = Self::Refused(refusal);
                fail!(
                    from origin,
                    with ResizeError::UnsupportedLength,
                    "The loan was refused for the given length"
                );
            }
            other => {
                *self = other;
                fail!(
                    from origin,
                    with ResizeError::DirectionUnsupported,
                    "The sample is loaned for a payload written from its end"
                );
            }
        }
    }

    /// Loans for the requested size, or if already loaned, attempts to resize
    /// the loan backward.
    fn loan_or_resize_backward(&mut self, len: usize) -> Result<(), ResizeError> {
        let origin = origin!("PendingLoan::loan_or_resize_backward");

        match core::mem::replace(self, Self::Refused(ReceiveError::Malformed)) {
            Self::Unloaned(loanable) => match loanable.loan_backward(len) {
                Ok(loaned) => {
                    *self = Self::Loaned(LoanedWritableSample::WrittenBackward(loaned));
                    Ok(())
                }
                Err(refusal) => {
                    let refusal = ResizeError::from(refusal);
                    *self = Self::Refused(refusal.into());
                    fail!(
                        from origin,
                        with refusal,
                        "Failed to loan a sample for a payload of {} bytes", len
                    );
                }
            },
            Self::Loaned(LoanedWritableSample::WrittenBackward(mut loaned)) => {
                match loaned.resize(len) {
                    Ok(()) => {
                        *self = Self::Loaned(LoanedWritableSample::WrittenBackward(loaned));
                        Ok(())
                    }
                    Err(refusal) => {
                        *self = Self::Refused(refusal.into());
                        fail!(
                            from origin,
                            with refusal,
                            "The loaned payload cannot be resized to {} bytes", len
                        );
                    }
                }
            }
            Self::Refused(refusal) => {
                *self = Self::Refused(refusal);
                fail!(
                    from origin,
                    with ResizeError::UnsupportedLength,
                    "The loan was refused for the given length"
                );
            }
            other => {
                *self = other;
                fail!(
                    from origin,
                    with ResizeError::DirectionUnsupported,
                    "The sample is loaned for a payload written from its front"
                );
            }
        }
    }

    /// Reference to the bytes of the payload.
    fn payload(&self) -> &[u8] {
        match self {
            Self::Loaned(loaned) => loaned.as_ref().payload,
            Self::Unloaned(_) | Self::Refused(_) => &[],
        }
    }

    /// Mutable reference to the bytes of the payload.
    fn payload_mut(&mut self) -> &mut [u8] {
        match self {
            Self::Loaned(loaned) => loaned.as_mut().payload,
            Self::Unloaned(_) | Self::Refused(_) => &mut [],
        }
    }

    /// Retrieve a writable sample backed by a loan.
    ///
    /// Panics if the loan was never made.
    fn into_writable_sample(self) -> LoanedWritableSample<L> {
        let origin = origin!("PendingLoan::into_writable_sample");
        match self {
            Self::Loaned(loaned) => loaned,
            Self::Unloaned(_) | Self::Refused(_) => {
                fatal_panic!(from origin, "A message was written without a loan")
            }
        }
    }

    /// Why the loan refused.
    ///
    /// Panics unless the loan was refused.
    pub(super) fn refusal(&self) -> ReceiveError {
        let origin = origin!("PendingLoan::refusal");
        match self {
            Self::Refused(refusal) => *refusal,
            Self::Unloaned(_) | Self::Loaned(_) => {
                fatal_panic!(
                    from origin,
                    "A refusal was reported despite a successful loan. This is a coding error."
                )
            }
        }
    }
}

impl<L: LoanableSample> Region for PendingLoan<L> {
    type ForwardRegion<'a>
        = LoanBackedRegion<'a, L>
    where
        Self: 'a;
    type BackwardRegion<'a>
        = LoanBackedRegion<'a, L>
    where
        Self: 'a;

    fn forward(&mut self) -> Self::ForwardRegion<'_> {
        LoanBackedRegion(self)
    }

    fn backward(&mut self) -> Self::BackwardRegion<'_> {
        LoanBackedRegion(self)
    }
}

/// Regions of the loaned bytes.
pub(super) struct LoanBackedRegion<'a, L: LoanableSample>(&'a mut PendingLoan<L>);

impl<L: LoanableSample> Deref for LoanBackedRegion<'_, L> {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        self.0.payload()
    }
}

impl<L: LoanableSample> DerefMut for LoanBackedRegion<'_, L> {
    fn deref_mut(&mut self) -> &mut [u8] {
        self.0.payload_mut()
    }
}

impl<L: LoanableSample> ForwardRegion for LoanBackedRegion<'_, L> {
    fn resize(&mut self, len: usize) -> Result<(), ResizeError> {
        self.0.loan_or_resize_forward(len)
    }
}

impl<L: LoanableSample> BackwardRegion for LoanBackedRegion<'_, L> {
    fn resize(&mut self, len: usize) -> Result<(), ResizeError> {
        self.0.loan_or_resize_backward(len)
    }
}

/// The loaned sample, writable from either end.
pub(super) enum LoanedWritableSample<L: LoanableSample> {
    WrittenForward(L::ForwardWritableSample),
    WrittenBackward(L::BackwardWritableSample),
}

impl<L: LoanableSample> WritableSample for LoanedWritableSample<L> {
    type InitializedSample = L::InitializedSample;

    fn as_ref(&self) -> SampleBytesRef<'_> {
        match self {
            Self::WrittenForward(loaned) => loaned.as_ref(),
            Self::WrittenBackward(loaned) => loaned.as_ref(),
        }
    }

    fn as_mut(&mut self) -> SampleBytesRefMut<'_> {
        match self {
            Self::WrittenForward(loaned) => loaned.as_mut(),
            Self::WrittenBackward(loaned) => loaned.as_mut(),
        }
    }

    unsafe fn assume_init(self) -> L::InitializedSample {
        match self {
            // SAFETY: the caller wrote the header and the payload.
            Self::WrittenForward(loaned) => unsafe { loaned.assume_init() },
            // SAFETY: the caller wrote the header and the payload.
            Self::WrittenBackward(loaned) => unsafe { loaned.assume_init() },
        }
    }
}

/// A take destination that writes both regions into a loaned sample.
///
/// The size of the header must match that expected by the service.
pub(super) struct LoanBackedDestination<'l, L: LoanableSample> {
    loan: &'l mut PendingLoan<L>,
    header_size: usize,
}

impl<'l, L: LoanableSample> LoanBackedDestination<'l, L> {
    fn new(loan: &'l mut PendingLoan<L>, header_size: usize) -> Self {
        Self { loan, header_size }
    }
}

impl<'a, L: LoanableSample> TakeDestination<'a> for &'a mut LoanBackedDestination<'_, L> {
    fn for_lengths(self, lengths: SampleLengths) -> Option<SampleBytesRefMut<'a>> {
        if lengths.header != self.header_size {
            *self.loan = PendingLoan::Refused(ReceiveError::Malformed);
            return None;
        }
        self.loan.loan_or_resize_forward(lengths.payload).ok()?;
        match self.loan {
            PendingLoan::Loaned(loaned) => Some(loaned.as_mut()),
            _ => None,
        }
    }
}

/// A take destination that writes the payload into the loaned sample and
/// a header that needs decoding into a scratch buffer.
pub(super) struct SplitDestination<'l, 's, L: LoanableSample> {
    loan: &'l mut PendingLoan<L>,
    scratch: &'s mut Vec<u8>,
}

impl<'l, 's, L: LoanableSample> SplitDestination<'l, 's, L> {
    fn new(loan: &'l mut PendingLoan<L>, scratch: &'s mut Vec<u8>) -> Self {
        Self { loan, scratch }
    }
}

impl<'a, L: LoanableSample> TakeDestination<'a> for &'a mut SplitDestination<'_, '_, L> {
    fn for_lengths(self, lengths: SampleLengths) -> Option<SampleBytesRefMut<'a>> {
        self.loan.loan_or_resize_forward(lengths.payload).ok()?;
        self.scratch.resize(lengths.header, 0);
        Some(SampleBytesRefMut {
            header: self.scratch,
            payload: self.loan.payload_mut(),
        })
    }
}
