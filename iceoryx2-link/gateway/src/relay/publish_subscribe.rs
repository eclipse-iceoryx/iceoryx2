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
use core::marker::PhantomData;

use iceoryx2::service::Service;
use iceoryx2_link_adapter::Mapping;
use iceoryx2_link_adapter::{
    Adapter, EndpointDescription, LoanError, LoanableSample, PublishSubscribeEndpoints, Region,
    TakeError, Transcoding, UnsupportedLength,
};
use iceoryx2_link_adapter::{
    HeaderTranscoder, PayloadTranscoder, PublishSubscribeTranslation, SampleTranscoder, Translator,
};
use iceoryx2_link_backend::relay::{PublishSubscribeRelay, ReceiveOutcome, RelayBuilder};
use iceoryx2_link_backend::service_description::{
    PublishSubscribeDescription, SampleTypes, ServiceDescription,
};
use iceoryx2_link_backend::wire::publish_subscribe::Sample;
use iceoryx2_link_backend::wire::sample::{WritableSample, payload_bytes, user_header_bytes};
use iceoryx2_log::{fail, fatal_panic, origin};

use crate::relay::{CreationError, ReceiveError, SendError};

/// Creates relays over an adapter's endpoints.
pub struct Builder<'a, S, A, M: Mapping, T: Translator> {
    pub(super) adapter: &'a mut A,
    pub(super) translator: &'a T,
    pub(super) publish_subscribe_description: PublishSubscribeDescription<'a>,
    /// The endpoint description of the service opened on the middleware.
    pub(super) endpoint_description: &'a EndpointDescription<M::EndpointSettings, T::EndpointTypes>,
    pub(super) _service: PhantomData<S>,
}

impl<S, A, M, T> RelayBuilder for Builder<'_, S, A, M, T>
where
    S: Service,
    M: Mapping,
    T: Translator,
    A: Adapter<EndpointSettings = M::EndpointSettings, EndpointTypes = T::EndpointTypes>,
{
    type CreationError = CreationError;
    type Relay = Relay<S, A::PublishSubscribeEndpoints, T::Transcoder>;

    fn create(self) -> Result<Self::Relay, Self::CreationError> {
        let origin = origin!("Builder::create");
        let publish_subscribe_description = self.publish_subscribe_description;
        let translation = fail!(
            from origin,
            when self
                .translator
                .publish_subscribe(ServiceDescription::types(&publish_subscribe_description), &self.endpoint_description.types),
            with CreationError::PublishSubscribeTranslation,
            "No translation for service {}", publish_subscribe_description.name()
        );
        let endpoints = fail!(
            from origin,
            when self.adapter.publish_subscribe(self.endpoint_description),
            with CreationError::Endpoints,
            "Failed to open the endpoints of service {}", publish_subscribe_description.name()
        );
        Ok(Relay::new(
            endpoints,
            translation,
            publish_subscribe_description.types().clone(),
        ))
    }
}

/// Moves publish-subscribe samples over the gateway's endpoints,
/// translating them on the way.
pub struct Relay<S, E, X> {
    endpoints: E,
    translation: PublishSubscribeTranslation<X>,
    types: SampleTypes,
    scratch: Scratch,
    _service: PhantomData<S>,
}

impl<S: Service, E: PublishSubscribeEndpoints, X: SampleTranscoder> Relay<S, E, X> {
    pub(crate) fn new(
        endpoints: E,
        translation: PublishSubscribeTranslation<X>,
        types: SampleTypes,
    ) -> Self {
        Self {
            endpoints,
            translation,
            types,
            scratch: Scratch::default(),
            _service: PhantomData,
        }
    }
}

impl<S: Service, E: PublishSubscribeEndpoints, X: SampleTranscoder> PublishSubscribeRelay<S>
    for Relay<S, E, X>
{
    type SendError = SendError;
    type ReceiveError = ReceiveError;

    fn send(&mut self, sample: &Sample<S>) -> Result<(), Self::SendError> {
        let origin = origin!("Relay::send");

        let pending =
            PendingMessage::new(&self.translation, &self.types, sample, &mut self.scratch);
        let message = fail!(
            from origin,
            when pending.into_message(),
            "Failed to encode a sample into a message"
        );
        fail!(
            from origin,
            when self.endpoints.publish(message.header, message.payload),
            with SendError::Endpoints,
            "Failed to publish a message"
        );
        Ok(())
    }

    fn receive<L: LoanableSample>(
        &mut self,
        loanable: L,
    ) -> Result<ReceiveOutcome<L::Sample>, Self::ReceiveError> {
        let origin = origin!("Relay::receive");

        let pending = UnloanedPendingSample::new(&self.translation, loanable, &mut self.scratch);
        let taken = match self.endpoints.take(pending) {
            Ok(taken) => taken,
            Err(TakeError::Malformed) => {
                fail!(
                    from origin,
                    with ReceiveError::Malformed,
                    "Took a message that does not fit the service"
                );
            }
            Err(TakeError::Exhausted) => {
                fail!(from origin, with ReceiveError::Loan, "No sample to take into");
            }
            Err(TakeError::Endpoints(_)) => {
                fail!(from origin, with ReceiveError::Endpoints, "Failed to take a message");
            }
        };
        let pending = match taken {
            ReceiveOutcome::Sample(pending) => pending,
            ReceiveOutcome::Skipped => return Ok(ReceiveOutcome::Skipped),
            ReceiveOutcome::Empty => return Ok(ReceiveOutcome::Empty),
        };
        let writable = fail!(
            from origin,
            when pending.into_sample(),
            "Failed to decode a message into a sample"
        );
        Ok(ReceiveOutcome::Sample(writable))
    }
}

/// The bytes of a middleware's message.
struct Message<'a> {
    header: &'a [u8],
    payload: &'a [u8],
}

/// A message being assembled from a sample.
///
/// Each region of the sample is either handed over as is, or encoded into
/// a scratch that is handed over instead.
struct PendingMessage<'a, X> {
    translation: &'a PublishSubscribeTranslation<X>,
    header: &'a [u8],
    payload: &'a [u8],
    scratch: &'a mut Scratch,
}

impl<'a, X> PendingMessage<'a, X> {
    fn new<S: Service>(
        translation: &'a PublishSubscribeTranslation<X>,
        types: &'a SampleTypes,
        sample: &'a Sample<S>,
        scratch: &'a mut Scratch,
    ) -> Self {
        // SAFETY: the sample belongs to the service this relay was built
        // for, whose description states the user header size.
        let header = unsafe { user_header_bytes(sample.user_header(), types.user_header.size) };
        let payload = payload_bytes(sample.payload());
        Self {
            translation,
            header,
            payload,
            scratch,
        }
    }
}

impl<'a, X: SampleTranscoder> PendingMessage<'a, X> {
    /// The message with every region in the middleware's form.
    fn into_message(self) -> Result<Message<'a>, SendError> {
        let origin = origin!("PendingMessage::into_message");

        // Nothing to do for Passthrough translator.
        let PublishSubscribeTranslation::Transcode {
            outbound,
            transcoder,
            ..
        } = self.translation
        else {
            return Ok(Message {
                header: self.header,
                payload: self.payload,
            });
        };

        // Transcode the payload.
        let payload = match outbound.payload {
            Transcoding::Passthrough => self.payload,
            Transcoding::Transcode => {
                fail!(
                    from origin,
                    when transcoder.payloads().encode(self.payload, &mut self.scratch.payload),
                    with SendError::Transcode,
                    "Failed to encode the payload of a sample"
                );
                &self.scratch.payload[..]
            }
        };

        // Transcode the header.
        let header = match outbound.header {
            Transcoding::Passthrough => self.header,
            Transcoding::Transcode => {
                fail!(
                    from origin,
                    when transcoder.headers().encode(self.header, &mut self.scratch.header),
                    with SendError::Transcode,
                    "Failed to encode the header of a sample"
                );
                &self.scratch.header[..]
            }
        };

        Ok(Message { header, payload })
    }
}

/// An [`UnloanedSample`][unloaned] with the translation applied to a taken
/// message when populating the sample.
///
/// [`LoanableSample::loan`] consumes it and yields a
/// [`LoanedPendingSample`]. A passthrough payload is loaned for right
/// away, a transcoded one is written to a scratch buffer to be decoded.
///
/// [unloaned]: iceoryx2_link_backend::wire::publish_subscribe::UnloanedSample
struct UnloanedPendingSample<'a, L: LoanableSample, X> {
    translation: &'a PublishSubscribeTranslation<X>,
    loanable: L,
    scratch: &'a mut Scratch,
}

impl<'a, L: LoanableSample, X> UnloanedPendingSample<'a, L, X> {
    fn new(
        translation: &'a PublishSubscribeTranslation<X>,
        loanable: L,
        scratch: &'a mut Scratch,
    ) -> Self {
        scratch.reset();
        Self {
            translation,
            loanable,
            scratch,
        }
    }
}

impl<'a, L: LoanableSample, X> LoanableSample for UnloanedPendingSample<'a, L, X> {
    type Sample = LoanedPendingSample<'a, L, X>;

    fn header_size(&self) -> usize {
        self.loanable.header_size()
    }

    fn loan(self, payload_len: usize) -> Result<Self::Sample, LoanError> {
        let Self {
            translation,
            loanable,
            scratch,
        } = self;

        let loan = match translation.inbound().payload {
            // Loan the sample and write the payload directly.
            Transcoding::Passthrough => LoanState::Loaned(loanable.loan(payload_len)?),
            // Write the payload to the scratch, the sample is loaned when
            // the decode knows the local length.
            Transcoding::Transcode => {
                scratch.payload.resize(payload_len, 0);
                LoanState::Deferred(loanable)
            }
        };

        Ok(LoanedPendingSample {
            translation,
            loan,
            scratch,
        })
    }
}

/// The state of the loan behind a taken payload.
enum LoanState<L: LoanableSample> {
    /// Taken for the payload as written.
    Loaned(L::Sample),
    /// Deferred until the decode knows the local payload length.
    Deferred(L),
}

/// A [`LoanedSample`][loaned] with the translation applied to a taken message
/// when populating the sample.
///
/// Produced by [`LoanableSample::loan`] on an [`UnloanedPendingSample`].
///
/// Once the header and the payload are written, can be converted to a
/// populated sample with [`LoanedPendingSample::into_sample`].
///
/// [loaned]: iceoryx2_link_backend::wire::publish_subscribe::LoanedSample
struct LoanedPendingSample<'a, L: LoanableSample, X> {
    translation: &'a PublishSubscribeTranslation<X>,
    loan: LoanState<L>,
    scratch: &'a mut Scratch,
}

impl<L: LoanableSample, X> WritableSample for LoanedPendingSample<'_, L, X> {
    fn payload(&mut self) -> &mut [u8] {
        match self.loan {
            LoanState::Loaned(ref mut writable) => writable.payload(),
            LoanState::Deferred(_) => &mut self.scratch.payload,
        }
    }

    fn header(&mut self, len: usize) -> Result<&mut [u8], UnsupportedLength> {
        match (self.translation.inbound().header, &mut self.loan) {
            // Provide the sample's header directly.
            (Transcoding::Passthrough, LoanState::Loaned(writable)) => writable.header(len),
            // Provide the scratch buffer to decode into or store a header
            // provided before the destination sample.
            _ => self.scratch.header.for_length(len),
        }
    }
}

impl<L: LoanableSample, X: SampleTranscoder> LoanedPendingSample<'_, L, X> {
    /// The loaned sample, decoding what the take parked in the scratch.
    fn into_sample(self) -> Result<L::Sample, ReceiveError> {
        let origin = origin!("LoanedPendingSample::into_sample");

        let LoanedPendingSample {
            translation,
            loan,
            scratch,
        } = self;

        // Acquire a writable loan for the sample.
        let mut writable = match (loan, translation) {
            (LoanState::Loaned(writable), _) => writable,
            (
                LoanState::Deferred(loanable),
                PublishSubscribeTranslation::Transcode { transcoder, .. },
            ) => {
                fail!(
                    from origin,
                    when transcoder.payloads().decode(&scratch.payload, loanable),
                    to ReceiveError,
                    "Failed to decode the payload of a message"
                )
            }
            (LoanState::Deferred(_), PublishSubscribeTranslation::Passthrough) => {
                fatal_panic!(
                    from origin,
                    "A payload was deferred without a transcoder to decode it"
                );
            }
        };

        // Decode the header into the sample or populate the sample with
        // a header prepared in the scratch.
        match translation {
            PublishSubscribeTranslation::Transcode {
                inbound,
                transcoder,
                ..
            } if inbound.header == Transcoding::Transcode => {
                fail!(
                    from origin,
                    when transcoder
                        .headers()
                        .decode(&scratch.header, &mut writable),
                    to ReceiveError,
                    "Failed to decode the header of a message"
                );
            }
            _ if !scratch.header.is_empty() => {
                let header = fail!(
                    from origin,
                    when writable.header(scratch.header.len()),
                    to ReceiveError,
                    "A header of {} bytes does not fit the service", scratch.header.len()
                );
                header.copy_from_slice(&scratch.header);
            }
            _ => {}
        }

        Ok(writable)
    }
}

/// Reusable buffers for a message's regions in the middleware's forms, in
/// either direction.
#[derive(Default)]
struct Scratch {
    header: Vec<u8>,
    payload: Vec<u8>,
}

impl Scratch {
    /// Reset the scratch for a new message.
    fn reset(&mut self) {
        // The payload is always sized by a take before it is read.
        // The header buffer is reused.
        self.header.clear();
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
    use iceoryx2_link_adapter::{SampleTranscodings, TranscodeError};
    use iceoryx2_link_backend::service_description::TypeDescription;
    use iceoryx2_link_backend::wire::publish_subscribe::{
        LoanFn, Publisher, SampleMut, SampleMutUninit, Subscriber, UnloanedSample,
        payload_bytes_mut, user_header_bytes_mut,
    };
    use iceoryx2_link_backend::wire::sample::{Header, Payload, WritableSample};

    /// The header type of the service under test, behind the untyped marker.
    type HeaderType = u64;
    /// The payload type of the service under test, behind the untyped marker.
    type PayloadType = u64;
    /// The size of a header and of a payload alike.
    const SIZE: usize = core::mem::size_of::<PayloadType>();
    const HEADER: [u8; SIZE] = HeaderType::to_ne_bytes(0x1111_2222_3333_4444);
    const PAYLOAD: [u8; SIZE] = PayloadType::to_ne_bytes(0x5555_6666_7777_8888);
    /// What the fake transcoder decodes a header to, whatever the wire.
    const DECODED_HEADER: [u8; SIZE] = [0xAB; SIZE];

    const TRANSCODED_HEADER: SampleTranscodings = SampleTranscodings {
        header: Transcoding::Transcode,
        payload: Transcoding::Passthrough,
    };
    const TRANSCODED_PAYLOAD: SampleTranscodings = SampleTranscodings {
        header: Transcoding::Passthrough,
        payload: Transcoding::Transcode,
    };

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

    fn write(
        region: Result<&mut [u8], UnsupportedLength>,
        bytes: &[u8],
    ) -> Result<(), TakeError<Infallible>> {
        match region {
            Ok(into) => {
                into.copy_from_slice(bytes);
                Ok(())
            }
            Err(refusal) => Err(refusal.into()),
        }
    }

    impl PublishSubscribeEndpoints for StubEndpoints {
        type Failure = Infallible;

        fn publish(&mut self, header: &[u8], payload: &[u8]) -> Result<(), Self::Failure> {
            self.published.push(message(header, payload));
            Ok(())
        }

        fn take<L: LoanableSample>(
            &mut self,
            loanable: L,
        ) -> Result<ReceiveOutcome<L::Sample>, TakeError<Self::Failure>> {
            let Some(message) = self.pending.take() else {
                return Ok(ReceiveOutcome::Empty);
            };
            let mut writable = loanable.loan(message.payload.len())?;
            writable.payload().copy_from_slice(&message.payload);
            write(writable.header(message.header.len()), &message.header)?;
            Ok(ReceiveOutcome::Sample(writable))
        }
    }

    /// Reverses the bytes of a payload, drops a header on the way out and
    /// makes one up on the way in.
    struct StubTranscoder;

    fn reversed<R: Region>(bytes: &[u8], into: &mut R) -> Result<(), TranscodeError<Infallible>> {
        let into = into.for_length(bytes.len())?;
        reverse_into(bytes, into);
        Ok(())
    }

    fn reverse_into(bytes: &[u8], into: &mut [u8]) {
        for (to, from) in into.iter_mut().zip(bytes.iter().rev()) {
            *to = *from;
        }
    }

    fn reversed_bytes(bytes: [u8; SIZE]) -> [u8; SIZE] {
        let mut reversed = bytes;
        reversed.reverse();
        reversed
    }

    impl HeaderTranscoder for StubTranscoder {
        type Failure = Infallible;

        fn encode<R: Region>(
            &self,
            _: &[u8],
            into: &mut R,
        ) -> Result<(), TranscodeError<Self::Failure>> {
            into.for_length(0)?;
            Ok(())
        }

        fn decode<W: WritableSample>(
            &self,
            _: &[u8],
            writable: &mut W,
        ) -> Result<(), TranscodeError<Self::Failure>> {
            writable
                .header(DECODED_HEADER.len())?
                .copy_from_slice(&DECODED_HEADER);
            Ok(())
        }
    }

    impl PayloadTranscoder for StubTranscoder {
        type Failure = Infallible;

        fn encode<R: Region>(
            &self,
            payload: &[u8],
            into: &mut R,
        ) -> Result<(), TranscodeError<Self::Failure>> {
            reversed(payload, into)
        }

        fn decode<L: LoanableSample>(
            &self,
            wire: &[u8],
            loanable: L,
        ) -> Result<L::Sample, TranscodeError<Self::Failure>> {
            let mut writable = loanable.loan(wire.len())?;
            reverse_into(wire, writable.payload());
            Ok(writable)
        }
    }

    impl SampleTranscoder for StubTranscoder {
        type HeaderTranscoder = StubTranscoder;
        type PayloadTranscoder = StubTranscoder;

        fn headers(&self) -> &StubTranscoder {
            self
        }

        fn payloads(&self) -> &StubTranscoder {
            self
        }
    }

    /// The fake transcoder applied as `transcodings` says, in both
    /// directions.
    fn transcoded(transcodings: SampleTranscodings) -> PublishSubscribeTranslation<StubTranscoder> {
        PublishSubscribeTranslation::Transcode {
            outbound: transcodings,
            inbound: transcodings,
            transcoder: StubTranscoder,
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
        translation: PublishSubscribeTranslation<StubTranscoder>,
    ) -> Relay<local::Service, StubEndpoints, StubTranscoder> {
        Relay::<local::Service, StubEndpoints, StubTranscoder>::new(endpoints, translation, types())
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
            relay: &mut Relay<local::Service, StubEndpoints, StubTranscoder>,
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
        relay: &mut Relay<local::Service, StubEndpoints, StubTranscoder>,
        loan: &mut LoanFn<'_, local::Service, LoanError>,
    ) -> Result<Option<SampleMut<local::Service>>, ReceiveError> {
        let types = types();
        let ReceiveOutcome::Sample(loaned) = relay.receive(UnloanedSample::new(&types, loan))?
        else {
            return Ok(None);
        };
        // SAFETY: the stub endpoints populated both regions of the loaned
        // sample, the header and the payload.
        Ok(Some(unsafe { loaned.into_sample().assume_init() }))
    }

    fn header_of(sample: &SampleMut<local::Service>) -> [u8; SIZE] {
        // SAFETY: the header size is the service's.
        unsafe { user_header_bytes(sample.user_header(), SIZE) }
            .try_into()
            .expect("a header")
    }

    fn payload_of(sample: &SampleMut<local::Service>) -> [u8; SIZE] {
        payload_bytes(sample.payload())
            .try_into()
            .expect("a payload")
    }

    #[test]
    fn passthrough_takes_a_message_into_the_sample() {
        let ports = Ports::open();
        let mut relay = relay(
            StubEndpoints::pending(message(&HEADER, &PAYLOAD)),
            PublishSubscribeTranslation::Passthrough,
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
        let mut relay = relay(
            StubEndpoints::default(),
            PublishSubscribeTranslation::Passthrough,
        );

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
            transcoded(TRANSCODED_PAYLOAD),
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
        let mut relay = relay(StubEndpoints::default(), transcoded(TRANSCODED_PAYLOAD));

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
            transcoded(TRANSCODED_HEADER),
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
        let mut relay = relay(StubEndpoints::default(), transcoded(TRANSCODED_HEADER));

        relay
            .send(&ports.sample(&HEADER, &PAYLOAD))
            .expect("send succeeds");

        let published = relay.endpoints.published();
        assert_that!(published.header, is_empty);
        assert_that!(published.payload, eq PAYLOAD.to_vec());
    }

    #[test]
    fn nothing_pending_is_nothing_received() {
        let ports = Ports::open();
        let mut relay = relay(
            StubEndpoints::default(),
            PublishSubscribeTranslation::Passthrough,
        );

        let received = ports.receive(&mut relay).expect("receive succeeds");

        assert_that!(received, is_none);
    }

    #[test]
    fn a_header_of_another_length_is_malformed() {
        const SHORT: usize = SIZE / 2;
        let ports = Ports::open();
        let mut relay = relay(
            StubEndpoints::pending(message(&[0; SHORT], &PAYLOAD)),
            PublishSubscribeTranslation::Passthrough,
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
            PublishSubscribeTranslation::Passthrough,
        );

        let received = ports.receive(&mut relay).err();

        assert_that!(received, eq Some(ReceiveError::Malformed));
    }

    #[test]
    fn a_publisher_without_a_sample_is_a_loan_error() {
        let mut relay = relay(
            StubEndpoints::pending(message(&HEADER, &PAYLOAD)),
            PublishSubscribeTranslation::Passthrough,
        );

        let received = receive_into(&mut relay, &mut |_| -> Result<_, ()> { Err(()) }).err();

        assert_that!(received, eq Some(ReceiveError::Loan));
    }
}
