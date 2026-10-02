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

//! Relays publish-subscribe samples between the local service and the
//! middleware's endpoints.
//!
//! A sample recevied from the local service is either passed directly to the
//! endpoint, or transcoded into a scratch, which is passed instead.
//!
//! ```text
//!   sample from the local port
//!            │
//!            │
//!     SampleBytesRef
//!            │
//!            ├── as is ────────────────────────────────────────▶ sent via the endpoint
//!            │
//!            │               ┌────────────────────────┐
//!            └── encoded ──▶ │ SampleBytes            │ ───────▶ sent via the endpoint
//!                            │   header: Vec<u8>      │
//!                            │     impl Region        │
//!                            │   payload: Vec<u8>     │
//!                            │     impl Region        │
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
//!                         message from the endpoint
//!                                     │
//!                                     │
//!                 ┌───────────────────┴────────────────────┐
//!               as-is                                   decoded
//!                 │                                        │
//!                 ▼                                        ▼
//!     ┌────────────────────────┐               ┌────────────────────────┐
//!     │ LoanBackedDestination  │               │ SampleBytes            │
//!     │   impl TakeDestination │               │   impl TakeDestination │
//!     │ SplitDestination       │               └────────────────────────┘
//!     │   impl TakeDestination │                            │
//!     └────────────────────────┘                            ▼
//!                 │                          ┌──────────────────────────┐
//!                 │                          │ LoanBackedRegion         │
//!                 │                          │   impl ForwardRegion     │
//!                 │                          │   impl BackwardRegion    │
//!                 │                          └──────────────────────────┘
//!                 │                                        │
//!   the endpoint fills the sample            a transcoder fills the sample
//!                 └───────────────────┬────────────────────┘
//!                                     │
//!                                     │  loans and fills
//!                                     │
//!                                     ▼
//!                                                         sized forward   ┌─────────────────────────────────────────┐  resized
//!                         ┌────────────────────────┐   ┌────────────────▶ │ PendingLoan::Loaned                     │ ◀──┐
//!                         │ PendingLoan::Unloaned  │   │                  │   LoanedWritableSample::WrittenForward  │ ───┘
//!                         │   impl LoanableSample  │───┤                  │     impl WritableSample                 │
//!                         └────────────────────────┘   │                  │     impl ForwardRegion                  │ ──assume_init()──▶ SampleMut sent via the local port
//!                                     │                │                  └─────────────────────────────────────────┘
//!                                     │                │  sized backward  ┌─────────────────────────────────────────┐  resized
//!                                     │                └────────────────▶ │ PendingLoan::Loaned                     │ ◀──┐
//!                                     │                                   │   LoanedWritableSample::WrittenBackward │ ───┘
//!                                     │                                   │     impl WritableSample                 │
//!                                     │                                   │     impl BackwardRegion                 │ ──assume_init()──▶ SampleMut sent via the local port
//!                                     │                                   └─────────────────────────────────────────┘
//!                                     │  loan refused
//!                                     │                                   ┌─────────────────────────────────────────┐
//!                                     └─────────────────────────────────▶ │ PendingLoan::Refused                    │ ─────────────▶ the receive fails
//!                                                                         │   LoanError                             │
//!                                                                         └─────────────────────────────────────────┘
//! ```

use alloc::vec::Vec;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};

use iceoryx2::service::Service;
use iceoryx2_link_adapter::Mapping;
use iceoryx2_link_adapter::{
    Adapter, EndpointDescription, EndpointTypes, LoanableSample, PublishSubscribeEndpoints,
    SampleBytes, SampleBytesRef, SampleBytesRefMut, SampleLengths, TakeDestination, TakeOutcome,
    TranscodeError, Transcoder,
};
use iceoryx2_link_adapter::{SampleShape, SampleTranscoders, TranscodesSamples, Translator};
use iceoryx2_link_backend::relay::{PublishSubscribeRelay, ReceiveOutcome, RelayBuilder};
use iceoryx2_link_backend::service_description::{PublishSubscribeDescription, SampleTypes};
use iceoryx2_link_backend::wire::publish_subscribe::Sample;
use iceoryx2_link_backend::wire::publish_subscribe::payload_bytes;
use iceoryx2_link_backend::wire::region::{BackwardRegion, ForwardRegion, Region, ResizeError};
use iceoryx2_link_backend::wire::sample::{WritableSample, user_header_bytes};
use iceoryx2_log::{fail, fatal_panic, origin};

use crate::relay::{CreationError, ReceiveError, SendError};

/// Creates relays over an adapter's endpoints.
pub struct Builder<'a, S, A, M: Mapping, T: Translator<SampleShape>> {
    pub(super) adapter: &'a mut A,
    pub(super) translator: &'a T,
    pub(super) publish_subscribe_description: PublishSubscribeDescription<'a>,
    /// The endpoint description of the service opened on the middleware.
    pub(super) endpoint_description:
        &'a EndpointDescription<M::EndpointSettings, EndpointTypes<T::RemoteTypes>>,
    pub(super) _service: PhantomData<S>,
}

impl<S, A, M, T> RelayBuilder for Builder<'_, S, A, M, T>
where
    S: Service,
    M: Mapping,
    T: Translator<SampleShape, Transcoders: TranscodesSamples>,
    A: Adapter<
            EndpointSettings = M::EndpointSettings,
            EndpointTypes = EndpointTypes<T::RemoteTypes>,
        >,
{
    type CreationError = CreationError;
    type Relay = Relay<S, A::PublishSubscribeEndpoints, T>;

    fn create(self) -> Result<Self::Relay, Self::CreationError> {
        let origin = origin!("Builder::create");
        let publish_subscribe_description = self.publish_subscribe_description;

        let EndpointTypes::PublishSubscribe(remote_types) = &self.endpoint_description.types else {
            fatal_panic!(
                from origin,
                "The endpoint of service {} is not a publish-subscribe endpoint", publish_subscribe_description.name()
            );
        };
        let transcoders = fail!(
            from origin,
            when self
                .translator
                .transcoders(publish_subscribe_description.types(), remote_types),
            with CreationError::PublishSubscribeTranslation,
            "No transcoders for service {}", publish_subscribe_description.name()
        );
        let endpoints = fail!(
            from origin,
            when self.adapter.publish_subscribe(self.endpoint_description),
            with CreationError::Endpoints,
            "Failed to open the endpoints of service {}", publish_subscribe_description.name()
        );

        Ok(Relay::new(
            endpoints,
            transcoders,
            publish_subscribe_description.types().clone(),
        ))
    }
}

/// Moves publish-subscribe samples over the gateway's endpoints,
/// translating them on the way.
pub struct Relay<S, E, X: Translator<SampleShape>> {
    endpoints: E,
    transcoders: X::Transcoders,
    types: SampleTypes,
    /// Reused buffers holding the middleware form of the regions being
    /// transcoded.
    scratch: SampleBytes,
    _service: PhantomData<S>,
}

impl<
    S: Service,
    E: PublishSubscribeEndpoints,
    X: Translator<SampleShape, Transcoders: TranscodesSamples>,
> Relay<S, E, X>
{
    pub(crate) fn new(endpoints: E, transcoders: X::Transcoders, types: SampleTypes) -> Self {
        Self {
            endpoints,
            transcoders,
            types,
            scratch: SampleBytes::default(),
            _service: PhantomData,
        }
    }
}

impl<
    S: Service,
    E: PublishSubscribeEndpoints,
    X: Translator<SampleShape, Transcoders: TranscodesSamples>,
> PublishSubscribeRelay<S> for Relay<S, E, X>
{
    type SendError = SendError;
    type ReceiveError = ReceiveError;

    fn send(&mut self, sample: &Sample<S>) -> Result<(), Self::SendError> {
        let origin = origin!("Relay::send");

        let local = SampleBytesRef {
            // SAFETY: the sample belongs to the service this relay was built
            // for, whose description states the user header size.
            header: unsafe { user_header_bytes(sample.user_header(), self.types.user_header.size) },
            payload: payload_bytes(sample),
        };

        // Encode the transcoded regions into the scratch, the others are
        // published from the sample.
        let scratch = &mut self.scratch;
        let wire = match self.transcoders.for_samples() {
            SampleTranscoders::TranscodeNone => local,
            SampleTranscoders::TranscodeHeader(header_transcoder) => {
                Self::encode_header(header_transcoder, local, &mut scratch.header)?;
                SampleBytesRef {
                    header: &scratch.header,
                    payload: local.payload,
                }
            }
            SampleTranscoders::TranscodePayload(payload_transcoder) => {
                Self::encode_payload(payload_transcoder, local, &mut scratch.payload)?;
                SampleBytesRef {
                    header: local.header,
                    payload: &scratch.payload,
                }
            }
            SampleTranscoders::TranscodeBoth(header_transcoder, payload_transcoder) => {
                Self::encode_header(header_transcoder, local, &mut scratch.header)?;
                Self::encode_payload(payload_transcoder, local, &mut scratch.payload)?;
                scratch.as_ref()
            }
        };

        fail!(
            from origin,
            when self.endpoints.publish(wire),
            with SendError::Endpoints,
            "Failed to publish a message"
        );

        Ok(())
    }

    fn receive<L: LoanableSample>(
        &mut self,
        loanable: L,
    ) -> Result<ReceiveOutcome<L::InitializedSample>, Self::ReceiveError> {
        let header_size = self.types.user_header.size;
        match self.transcoders.for_samples() {
            SampleTranscoders::TranscodeNone => {
                Self::receive_with_no_region_transcoded(&mut self.endpoints, header_size, loanable)
            }
            SampleTranscoders::TranscodeHeader(header_transcoder) => {
                Self::receive_with_header_transcoded(
                    &mut self.endpoints,
                    &mut self.scratch,
                    header_transcoder,
                    loanable,
                )
            }
            SampleTranscoders::TranscodePayload(payload_transcoder) => {
                Self::receive_with_payload_transcoded(
                    &mut self.endpoints,
                    &mut self.scratch,
                    header_size,
                    payload_transcoder,
                    loanable,
                )
            }
            SampleTranscoders::TranscodeBoth(header_transcoder, payload_transcoder) => {
                Self::receive_with_both_regions_transcoded(
                    &mut self.endpoints,
                    &mut self.scratch,
                    header_transcoder,
                    payload_transcoder,
                    loanable,
                )
            }
        }
    }
}

impl<
    S: Service,
    E: PublishSubscribeEndpoints,
    X: Translator<SampleShape, Transcoders: TranscodesSamples>,
> Relay<S, E, X>
{
    /// Takes a message straight into the loan.
    fn receive_with_no_region_transcoded<L: LoanableSample>(
        endpoints: &mut E,
        header_size: usize,
        loanable: L,
    ) -> Result<ReceiveOutcome<L::InitializedSample>, ReceiveError> {
        let origin = origin!("Relay::receive_with_no_region_transcoded");

        let mut loan = PendingLoan::new(loanable);
        let outcome = fail!(
            from origin,
            when endpoints.take(&mut LoanBackedDestination::new(&mut loan, header_size)),
            with ReceiveError::Endpoints,
            "Failed to take a message"
        );
        match outcome {
            TakeOutcome::Taken => {
                // SAFETY: the take wrote the header and the payload into the
                // loaned sample.
                Ok(ReceiveOutcome::Sample(unsafe {
                    loan.into_writable_sample().assume_init()
                }))
            }
            TakeOutcome::Declined => Err(loan.refusal()),
            TakeOutcome::Skipped => Ok(ReceiveOutcome::Skipped),
            TakeOutcome::Empty => Ok(ReceiveOutcome::Empty),
        }
    }

    /// Takes the payload straight into the loan and the header into the
    /// scratch, then decodes the header into the loan.
    fn receive_with_header_transcoded<L: LoanableSample>(
        endpoints: &mut E,
        scratch: &mut SampleBytes,
        header_transcoder: &<X::Transcoders as TranscodesSamples>::HeaderTranscoder,
        loanable: L,
    ) -> Result<ReceiveOutcome<L::InitializedSample>, ReceiveError> {
        let origin = origin!("Relay::receive_with_header_transcoded");

        let mut loan = PendingLoan::new(loanable);
        let outcome = fail!(
            from origin,
            when endpoints.take(&mut SplitDestination::new(&mut loan, &mut scratch.header)),
            with ReceiveError::Endpoints,
            "Failed to take a message"
        );
        match outcome {
            TakeOutcome::Taken => {}
            TakeOutcome::Declined => return Err(loan.refusal()),
            TakeOutcome::Skipped => return Ok(ReceiveOutcome::Skipped),
            TakeOutcome::Empty => return Ok(ReceiveOutcome::Empty),
        }

        let mut writable_sample = loan.into_writable_sample();
        Self::decode_header(header_transcoder, scratch.as_ref(), &mut writable_sample)?;

        // SAFETY: the header and the payload were both written into the
        // loaned sample above.
        Ok(ReceiveOutcome::Sample(unsafe {
            writable_sample.assume_init()
        }))
    }

    /// Takes a message into the scratch, then decodes the payload into the
    /// loan and copies the header.
    fn receive_with_payload_transcoded<L: LoanableSample>(
        endpoints: &mut E,
        scratch: &mut SampleBytes,
        header_size: usize,
        payload_transcoder: &<X::Transcoders as TranscodesSamples>::PayloadTranscoder,
        loanable: L,
    ) -> Result<ReceiveOutcome<L::InitializedSample>, ReceiveError> {
        let origin = origin!("Relay::receive_with_payload_transcoded");

        let outcome = fail!(
            from origin,
            when endpoints.take(&mut *scratch),
            with ReceiveError::Endpoints,
            "Failed to take a message"
        );
        match outcome {
            TakeOutcome::Taken => {}
            TakeOutcome::Declined => {
                fatal_panic!(
                    from origin,
                    "The endpoints reported a declined take into the relay's scratch buffers, which accept any length. This cannot occur with correctly implemented endpoints."
                );
            }
            TakeOutcome::Skipped => return Ok(ReceiveOutcome::Skipped),
            TakeOutcome::Empty => return Ok(ReceiveOutcome::Empty),
        }
        let wire = scratch.as_ref();

        let mut writable_sample = Self::decode_payload(payload_transcoder, wire, loanable)?;
        Self::copy_header(wire, &mut writable_sample, header_size)?;

        // SAFETY: the header and the payload were both written into the
        // loaned sample above.
        Ok(ReceiveOutcome::Sample(unsafe {
            writable_sample.assume_init()
        }))
    }

    /// Takes a message into the scratch, then decodes the payload and the
    /// header into the loan.
    fn receive_with_both_regions_transcoded<L: LoanableSample>(
        endpoints: &mut E,
        scratch: &mut SampleBytes,
        header_transcoder: &<X::Transcoders as TranscodesSamples>::HeaderTranscoder,
        payload_transcoder: &<X::Transcoders as TranscodesSamples>::PayloadTranscoder,
        loanable: L,
    ) -> Result<ReceiveOutcome<L::InitializedSample>, ReceiveError> {
        let origin = origin!("Relay::receive_with_both_regions_transcoded");

        let outcome = fail!(
            from origin,
            when endpoints.take(&mut *scratch),
            with ReceiveError::Endpoints,
            "Failed to take a message"
        );
        match outcome {
            TakeOutcome::Taken => {}
            TakeOutcome::Declined => {
                fatal_panic!(
                    from origin,
                    "The endpoints reported a declined take into the relay's scratch buffers, which accept any length. This cannot occur with correctly implemented endpoints."
                );
            }
            TakeOutcome::Skipped => return Ok(ReceiveOutcome::Skipped),
            TakeOutcome::Empty => return Ok(ReceiveOutcome::Empty),
        }
        let wire = scratch.as_ref();

        let mut writable_sample = Self::decode_payload(payload_transcoder, wire, loanable)?;
        Self::decode_header(header_transcoder, wire, &mut writable_sample)?;

        // SAFETY: the header and the payload were both written into the
        // loaned sample above.
        Ok(ReceiveOutcome::Sample(unsafe {
            writable_sample.assume_init()
        }))
    }

    /// Encodes the `local` header into `scratch`.
    fn encode_header(
        header_transcoder: &<X::Transcoders as TranscodesSamples>::HeaderTranscoder,
        local: SampleBytesRef<'_>,
        scratch: &mut Vec<u8>,
    ) -> Result<(), SendError> {
        let origin = origin!("Relay::encode_header");

        fail!(
            from origin,
            when header_transcoder.encode(local, scratch),
            with SendError::Transcode,
            "Failed to encode the header of a sample"
        );

        Ok(())
    }

    /// Encodes the `local` payload into `scratch`.
    fn encode_payload(
        payload_transcoder: &<X::Transcoders as TranscodesSamples>::PayloadTranscoder,
        local: SampleBytesRef<'_>,
        scratch: &mut Vec<u8>,
    ) -> Result<(), SendError> {
        let origin = origin!("Relay::encode_payload");

        fail!(
            from origin,
            when payload_transcoder.encode(local, scratch),
            with SendError::Transcode,
            "Failed to encode the payload of a sample"
        );

        Ok(())
    }

    /// Decodes the header of `wire` into the loaned sample.
    fn decode_header<W: WritableSample>(
        transcoder: &<X::Transcoders as TranscodesSamples>::HeaderTranscoder,
        wire: SampleBytesRef<'_>,
        loaned: &mut W,
    ) -> Result<(), ReceiveError> {
        let origin = origin!("Relay::decode_header");

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

    /// Loans a sample for the decoded payload of `wire` and decodes it
    /// there.
    fn decode_payload<L: LoanableSample>(
        transcoder: &<X::Transcoders as TranscodesSamples>::PayloadTranscoder,
        wire: SampleBytesRef<'_>,
        loanable: L,
    ) -> Result<LoanedWritableSample<L>, ReceiveError> {
        let origin = origin!("Relay::decode_payload");

        let mut loan = PendingLoan::new(loanable);
        match transcoder.decode(wire, &mut loan) {
            Ok(()) => Ok(loan.into_writable_sample()),
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
        let origin = origin!("Relay::copy_header");

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
}

/// A loan that is deferred until its size is known.
///
/// The size and direction in which the bytes shall be filled must be specified
/// when acquiring the loan.
enum PendingLoan<L: LoanableSample> {
    Unloaned(L),
    Loaned(LoanedWritableSample<L>),
    Refused(ReceiveError),
}

impl<L: LoanableSample> PendingLoan<L> {
    fn new(loanable: L) -> Self {
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
    fn refusal(&self) -> ReceiveError {
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
struct LoanBackedRegion<'a, L: LoanableSample>(&'a mut PendingLoan<L>);

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
enum LoanedWritableSample<L: LoanableSample> {
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
struct LoanBackedDestination<'l, L: LoanableSample> {
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
struct SplitDestination<'l, 's, L: LoanableSample> {
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

#[cfg(test)]
mod tests {
    use super::*;

    use alloc::vec::Vec;
    use core::convert::Infallible;

    use iceoryx2::node::{Node, NodeBuilder};
    use iceoryx2::port::LoanError;
    use iceoryx2::service::local;
    use iceoryx2::service::static_config::message_type_details::{TypeDetail, TypeVariant};
    use iceoryx2::testing::{generate_isolated_config, generate_service_name};
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_link_adapter::LocalTypes;
    use iceoryx2_link_backend::service_description::TypeDescription;
    use iceoryx2_link_backend::wire::publish_subscribe::{
        LoanFn, Publisher, SampleMut, SampleMutUninit, Subscriber, UnloanedSample,
        payload_bytes_mut, user_header_bytes_mut,
    };
    use iceoryx2_link_backend::wire::sample::{Header, Payload};

    /// The header type of the service under test, behind the untyped marker.
    type HeaderType = u64;
    /// The payload type of the service under test, behind the untyped marker.
    type PayloadType = u64;
    /// The size of a header and of a payload alike.
    const SIZE: usize = core::mem::size_of::<PayloadType>();
    const HEADER: [u8; SIZE] = HeaderType::to_ne_bytes(0x1111_2222_3333_4444);
    const PAYLOAD: [u8; SIZE] = PayloadType::to_ne_bytes(0x5555_6666_7777_8888);
    /// What the stub header transcoder decodes a header to, whatever the wire.
    const DECODED_HEADER: [u8; SIZE] = [0xAB; SIZE];

    /// A message in the middleware's forms.
    #[derive(Clone)]
    struct Message {
        header: Vec<u8>,
        payload: Vec<u8>,
    }

    fn message(header: &[u8], payload: &[u8]) -> Message {
        Message {
            header: header.to_vec(),
            payload: payload.to_vec(),
        }
    }

    /// Endpoints holding one pending message and every message published
    /// on them.
    #[derive(Default)]
    struct StubEndpoints {
        pending: Option<Message>,
        published: Vec<Message>,
    }

    impl StubEndpoints {
        fn pending(message: Message) -> Self {
            Self {
                pending: Some(message),
                ..Self::default()
            }
        }

        /// The one message published on the endpoints.
        fn published(&self) -> Message {
            assert_that!(self.published.len(), eq 1);
            self.published[0].clone()
        }
    }

    impl PublishSubscribeEndpoints for StubEndpoints {
        type Failure = Infallible;

        fn publish(&mut self, sample: SampleBytesRef<'_>) -> Result<(), Self::Failure> {
            self.published.push(message(sample.header, sample.payload));
            Ok(())
        }

        fn take<'a>(
            &mut self,
            destination: impl TakeDestination<'a>,
        ) -> Result<TakeOutcome, Self::Failure> {
            let Some(message) = self.pending.take() else {
                return Ok(TakeOutcome::Empty);
            };
            let Some(regions) = destination.for_lengths(SampleLengths {
                header: message.header.len(),
                payload: message.payload.len(),
            }) else {
                return Ok(TakeOutcome::Declined);
            };
            regions.header.copy_from_slice(&message.header);
            regions.payload.copy_from_slice(&message.payload);
            Ok(TakeOutcome::Taken)
        }
    }

    /// Drops the header on the way out and makes one up on the way in.
    struct StubHeader;

    /// Reverses the bytes of the payload in both directions.
    struct StubPayload;

    fn reversed<R: Region>(bytes: &[u8], into: &mut R) -> Result<(), TranscodeError<Infallible>> {
        let mut into = into.forward();
        into.resize(bytes.len())?;
        for (to, from) in into.iter_mut().zip(bytes.iter().rev()) {
            *to = *from;
        }
        Ok(())
    }

    fn reversed_bytes(bytes: [u8; SIZE]) -> [u8; SIZE] {
        let mut reversed = bytes;
        reversed.reverse();
        reversed
    }

    impl<'a> Transcoder<SampleBytesRef<'a>> for StubHeader {
        type Error = Infallible;

        fn encode<R: Region>(
            &self,
            _: SampleBytesRef<'a>,
            into: &mut R,
        ) -> Result<(), TranscodeError<Self::Error>> {
            into.forward().resize(0)?;
            Ok(())
        }

        fn decode<R: Region>(
            &self,
            _: SampleBytesRef<'a>,
            into: &mut R,
        ) -> Result<(), TranscodeError<Self::Error>> {
            let mut into = into.forward();
            into.resize(DECODED_HEADER.len())?;
            into.copy_from_slice(&DECODED_HEADER);
            Ok(())
        }
    }

    impl<'a> Transcoder<SampleBytesRef<'a>> for StubPayload {
        type Error = Infallible;

        fn encode<R: Region>(
            &self,
            local: SampleBytesRef<'a>,
            into: &mut R,
        ) -> Result<(), TranscodeError<Self::Error>> {
            reversed(local.payload, into)
        }

        fn decode<R: Region>(
            &self,
            wire: SampleBytesRef<'a>,
            into: &mut R,
        ) -> Result<(), TranscodeError<Self::Error>> {
            reversed(wire.payload, into)
        }
    }

    /// Names the stub transcoders as the relay's translator. The relay
    /// never calls it.
    struct StubTranslator;

    impl Translator<SampleShape> for StubTranslator {
        type RemoteTypes = SampleTypes;
        type Transcoders = SampleTranscoders<StubHeader, StubPayload>;
        type Error = Infallible;

        fn local(&self, remote: &SampleTypes) -> Result<LocalTypes<SampleShape>, Infallible> {
            Ok(remote.clone())
        }

        fn remote(&self, local: &LocalTypes<SampleShape>) -> Result<SampleTypes, Infallible> {
            Ok(local.clone())
        }

        fn transcoders(
            &self,
            _: &LocalTypes<SampleShape>,
            _: &SampleTypes,
        ) -> Result<Self::Transcoders, Infallible> {
            Ok(SampleTranscoders::TranscodeNone)
        }
    }

    fn types() -> SampleTypes {
        SampleTypes {
            payload: TypeDescription::from(&TypeDetail::new::<PayloadType>(TypeVariant::FixedSize)),
            user_header: TypeDescription::from(&TypeDetail::new::<HeaderType>(
                TypeVariant::FixedSize,
            )),
        }
    }

    fn relay(
        endpoints: StubEndpoints,
        transcoders: SampleTranscoders<StubHeader, StubPayload>,
    ) -> Relay<local::Service, StubEndpoints, StubTranslator> {
        Relay::<local::Service, StubEndpoints, StubTranslator>::new(endpoints, transcoders, types())
    }

    /// The local ports of a service with a `HeaderType` header and a
    /// `PayloadType` payload, opened untyped as the link opens them.
    struct Ports {
        publisher: Publisher<local::Service>,
        subscriber: Subscriber<local::Service>,
        _node: Node<local::Service>,
    }

    impl Ports {
        fn open() -> Self {
            let node = NodeBuilder::new()
                .config(&generate_isolated_config())
                .create::<local::Service>()
                .expect("node is created");
            let types = types();
            let payload = TypeDetail::try_from(&types.payload).expect("valid payload type");
            let user_header =
                TypeDetail::try_from(&types.user_header).expect("valid user header type");
            // SAFETY: the type details describe the untyped markers.
            let service = unsafe {
                node.service_builder(&generate_service_name())
                    .publish_subscribe::<Payload>()
                    .user_header::<Header>()
                    .__internal_set_user_header_type_details(&user_header)
                    .__internal_set_payload_type_details(&payload)
            }
            .create()
            .expect("service is created");
            let publisher = service
                .publisher_builder()
                .create()
                .expect("publisher is created");
            let subscriber = service
                .subscriber_builder()
                .create()
                .expect("subscriber is created");
            Self {
                publisher,
                subscriber,
                _node: node,
            }
        }

        fn loan(&self, len: usize) -> Result<SampleMutUninit<local::Service>, LoanError> {
            // SAFETY: the publisher was created for the untyped markers.
            unsafe { self.publisher.loan_custom_payload(len / SIZE) }
        }

        /// Receives through `relay` into samples loaned from the publisher.
        fn receive(
            &self,
            relay: &mut Relay<local::Service, StubEndpoints, StubTranslator>,
        ) -> Result<Option<SampleMut<local::Service>>, ReceiveError> {
            receive_into(relay, &mut |len| self.loan(len))
        }

        /// A received sample holding `header` and `payload`.
        fn sample(&self, header: &[u8; SIZE], payload: &[u8; SIZE]) -> Sample<local::Service> {
            let mut sample = self.loan(SIZE).expect("sample is loaned");
            // SAFETY: the header size is the service's.
            unsafe { user_header_bytes_mut(&mut sample, SIZE) }.copy_from_slice(header);
            payload_bytes_mut(&mut sample).copy_from_slice(payload);
            // SAFETY: the header and the payload were both populated above.
            unsafe { sample.assume_init() }
                .send()
                .expect("sample is sent");
            self.subscriber
                .receive()
                .expect("receive succeeds")
                .expect("a sample is received")
        }
    }

    /// Receives through `relay` into a sample loaned with `loan`.
    fn receive_into<LoanError>(
        relay: &mut Relay<local::Service, StubEndpoints, StubTranslator>,
        loan: &mut LoanFn<'_, local::Service, LoanError>,
    ) -> Result<Option<SampleMut<local::Service>>, ReceiveError> {
        let types = types();
        let ReceiveOutcome::Sample(sample) = relay.receive(UnloanedSample::new(&types, loan))?
        else {
            return Ok(None);
        };
        Ok(Some(sample))
    }

    fn header_of(sample: &SampleMut<local::Service>) -> [u8; SIZE] {
        // SAFETY: the header size is the service's.
        unsafe { user_header_bytes(sample.user_header(), SIZE) }
            .try_into()
            .expect("a header")
    }

    fn payload_of(sample: &SampleMut<local::Service>) -> [u8; SIZE] {
        let payload = sample.payload();
        // SAFETY: the payload marker is one byte, the slice is its bytes.
        unsafe { core::slice::from_raw_parts(payload.as_ptr() as *const u8, payload.len()) }
            .try_into()
            .expect("a payload")
    }

    #[test]
    fn passthrough_takes_a_message_into_the_sample() {
        let ports = Ports::open();
        let mut relay = relay(
            StubEndpoints::pending(message(&HEADER, &PAYLOAD)),
            SampleTranscoders::TranscodeNone,
        );

        let received = ports
            .receive(&mut relay)
            .expect("receive succeeds")
            .expect("a message is received");

        assert_that!(header_of(&received), eq HEADER);
        assert_that!(payload_of(&received), eq PAYLOAD);
    }

    #[test]
    fn passthrough_publishes_the_header_and_payload_of_a_sample() {
        let ports = Ports::open();
        let mut relay = relay(StubEndpoints::default(), SampleTranscoders::TranscodeNone);

        relay
            .send(&ports.sample(&HEADER, &PAYLOAD))
            .expect("send succeeds");

        let published = relay.endpoints.published();
        assert_that!(published.header, eq HEADER.to_vec());
        assert_that!(published.payload, eq PAYLOAD.to_vec());
    }

    #[test]
    fn a_transcoded_payload_is_decoded_into_the_sample() {
        let ports = Ports::open();
        let mut relay = relay(
            StubEndpoints::pending(message(&HEADER, &reversed_bytes(PAYLOAD))),
            SampleTranscoders::TranscodePayload(StubPayload),
        );

        let received = ports
            .receive(&mut relay)
            .expect("receive succeeds")
            .expect("a message is received");

        assert_that!(header_of(&received), eq HEADER);
        assert_that!(payload_of(&received), eq PAYLOAD);
    }

    #[test]
    fn a_transcoded_payload_is_encoded_into_the_message() {
        let ports = Ports::open();
        let mut relay = relay(
            StubEndpoints::default(),
            SampleTranscoders::TranscodePayload(StubPayload),
        );

        relay
            .send(&ports.sample(&HEADER, &PAYLOAD))
            .expect("send succeeds");

        let published = relay.endpoints.published();
        assert_that!(published.header, eq HEADER.to_vec());
        assert_that!(published.payload, eq reversed_bytes(PAYLOAD).to_vec());
    }

    #[test]
    fn a_transcoded_header_is_decoded_from_the_header_form() {
        let ports = Ports::open();
        // The middleware has no header form, the endpoints write none.
        let mut relay = relay(
            StubEndpoints::pending(message(&[], &PAYLOAD)),
            SampleTranscoders::TranscodeHeader(StubHeader),
        );

        let received = ports
            .receive(&mut relay)
            .expect("receive succeeds")
            .expect("a message is received");

        assert_that!(header_of(&received), eq DECODED_HEADER);
        assert_that!(payload_of(&received), eq PAYLOAD);
    }

    #[test]
    fn a_transcoded_header_is_encoded_into_the_header_form() {
        let ports = Ports::open();
        let mut relay = relay(
            StubEndpoints::default(),
            SampleTranscoders::TranscodeHeader(StubHeader),
        );

        relay
            .send(&ports.sample(&HEADER, &PAYLOAD))
            .expect("send succeeds");

        let published = relay.endpoints.published();
        assert_that!(published.header, is_empty);
        assert_that!(published.payload, eq PAYLOAD.to_vec());
    }

    #[test]
    fn a_transcoded_sample_has_both_regions_decoded() {
        let ports = Ports::open();
        let mut relay = relay(
            StubEndpoints::pending(message(&[], &reversed_bytes(PAYLOAD))),
            SampleTranscoders::TranscodeBoth(StubHeader, StubPayload),
        );

        let received = ports
            .receive(&mut relay)
            .expect("receive succeeds")
            .expect("a message is received");

        assert_that!(header_of(&received), eq DECODED_HEADER);
        assert_that!(payload_of(&received), eq PAYLOAD);
    }

    #[test]
    fn a_transcoded_sample_has_both_regions_encoded() {
        let ports = Ports::open();
        let mut relay = relay(
            StubEndpoints::default(),
            SampleTranscoders::TranscodeBoth(StubHeader, StubPayload),
        );

        relay
            .send(&ports.sample(&HEADER, &PAYLOAD))
            .expect("send succeeds");

        let published = relay.endpoints.published();
        assert_that!(published.header, is_empty);
        assert_that!(published.payload, eq reversed_bytes(PAYLOAD).to_vec());
    }

    #[test]
    fn nothing_pending_is_nothing_received() {
        let ports = Ports::open();
        let mut relay = relay(StubEndpoints::default(), SampleTranscoders::TranscodeNone);

        let received = ports.receive(&mut relay).expect("receive succeeds");

        assert_that!(received, is_none);
    }

    #[test]
    fn a_header_of_another_length_is_malformed() {
        const SHORT: usize = SIZE / 2;
        let ports = Ports::open();
        let mut relay = relay(
            StubEndpoints::pending(message(&[0; SHORT], &PAYLOAD)),
            SampleTranscoders::TranscodeNone,
        );

        let received = ports.receive(&mut relay).err();

        assert_that!(received, eq Some(ReceiveError::Malformed));
    }

    #[test]
    fn a_passthrough_header_of_another_length_is_malformed_when_transcoding() {
        const SHORT: usize = SIZE / 2;
        let ports = Ports::open();
        let mut relay = relay(
            StubEndpoints::pending(message(&[0; SHORT], &reversed_bytes(PAYLOAD))),
            SampleTranscoders::TranscodePayload(StubPayload),
        );

        let received = ports.receive(&mut relay).err();

        assert_that!(received, eq Some(ReceiveError::Malformed));
    }

    #[test]
    fn a_payload_of_another_length_is_malformed() {
        const SHORT: usize = SIZE / 2;
        let ports = Ports::open();
        let mut relay = relay(
            StubEndpoints::pending(message(&HEADER, &[0; SHORT])),
            SampleTranscoders::TranscodeNone,
        );

        let received = ports.receive(&mut relay).err();

        assert_that!(received, eq Some(ReceiveError::Malformed));
    }

    #[test]
    fn a_publisher_without_a_sample_is_a_loan_error() {
        let mut relay = relay(
            StubEndpoints::pending(message(&HEADER, &PAYLOAD)),
            SampleTranscoders::TranscodeNone,
        );

        let received = receive_into(&mut relay, &mut |_| -> Result<_, ()> { Err(()) }).err();

        assert_that!(received, eq Some(ReceiveError::Loan));
    }

    #[test]
    fn a_publisher_without_a_sample_is_a_loan_error_when_transcoding() {
        let mut relay = relay(
            StubEndpoints::pending(message(&HEADER, &reversed_bytes(PAYLOAD))),
            SampleTranscoders::TranscodePayload(StubPayload),
        );

        let received = receive_into(&mut relay, &mut |_| -> Result<_, ()> { Err(()) }).err();

        assert_that!(received, eq Some(ReceiveError::Loan));
    }

    #[test]
    fn a_plain_payload_written_from_its_end_is_a_transcode_error() {
        let ports = Ports::open();
        let types = types();
        let mut loan = |len| ports.loan(len);
        let mut sut: PendingLoan<UnloanedSample<'_, '_, local::Service, LoanError>> =
            PendingLoan::new(UnloanedSample::new(&types, &mut loan));

        let resized = BackwardRegion::resize(&mut sut.backward(), SIZE);

        assert_that!(resized, eq Err(ResizeError::DirectionUnsupported));
        assert_that!(sut.refusal(), eq ReceiveError::Transcode);
    }
}
