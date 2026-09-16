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
    Adapter, Destination, EndpointDescription, PublishSubscribeEndpoints, Region, ResizeError,
    TakeError, Transcoding,
};
use iceoryx2_link_adapter::{
    HeaderTranscoder, PayloadTranscoder, PublishSubscribeTranslation, SampleTranscoder, Translator,
};
use iceoryx2_link_backend::relay::{PublishSubscribeRelay, RelayBuilder};
use iceoryx2_link_backend::service_description::{
    PublishSubscribeDescription, PublishSubscribeTypes, ServiceDescription,
};
use iceoryx2_link_backend::wire::publish_subscribe::{
    LoanFn, Sample, SampleMut, SampleMutUninit, fits, payload_bytes, regions_mut, user_header_bytes,
};
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
    types: PublishSubscribeTypes,
    scratch: Scratch,
    _service: PhantomData<S>,
}

impl<S: Service, E: PublishSubscribeEndpoints, X: SampleTranscoder> Relay<S, E, X> {
    pub(crate) fn new(
        endpoints: E,
        translation: PublishSubscribeTranslation<X>,
        types: PublishSubscribeTypes,
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

    fn receive<LoanError>(
        &mut self,
        loan: &mut LoanFn<'_, S, LoanError>,
    ) -> Result<Option<SampleMut<S>>, Self::ReceiveError> {
        let origin = origin!("Relay::receive");

        let mut pending =
            PendingSample::new(&self.translation, &self.types, loan, &mut self.scratch);
        let taken = match self.endpoints.take(&mut pending) {
            Ok(taken) => taken,
            Err(TakeError::Rejected(refusal)) => {
                fail!(from origin, with ReceiveError::from(refusal), "Rejected a message");
            }
            Err(TakeError::Failed(_)) => {
                fail!(from origin, with ReceiveError::Endpoints, "Failed to take a message");
            }
        };
        if !taken {
            return Ok(None);
        }
        let sample = fail!(
            from origin,
            when pending.into_sample(),
            "Failed to decode a message into a sample"
        );
        // SAFETY: every region was written through the destination, by the
        // endpoints or the transcoder.
        Ok(Some(unsafe { sample.assume_init() }))
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
        types: &'a PublishSubscribeTypes,
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

/// A sample being assembled from a taken message.
///
/// The taken message is either moved directly into the sample, or into a
/// scratch and then decoded into the sample.
struct PendingSample<'a, 'b, S: Service, X, LoanError> {
    translation: &'a PublishSubscribeTranslation<X>,
    loan: Loan<'a, 'b, S, LoanError>,
    scratch: &'a mut Scratch,
}

impl<'a, 'b, S: Service, X, LoanError> PendingSample<'a, 'b, S, X, LoanError> {
    fn new(
        translation: &'a PublishSubscribeTranslation<X>,
        types: &'a PublishSubscribeTypes,
        loan: &'a mut LoanFn<'b, S, LoanError>,
        scratch: &'a mut Scratch,
    ) -> Self {
        scratch.reset();
        Self {
            translation,
            loan: Loan::new(types, loan),
            scratch,
        }
    }
}

impl<S: Service, X, LoanError> Destination for PendingSample<'_, '_, S, X, LoanError> {
    fn payload(&mut self, payload_size: usize) -> Result<&mut [u8], ResizeError> {
        match self.translation.inbound().payload {
            // Provide sample buffer directly.
            Transcoding::Passthrough => self.loan.for_length(payload_size),
            // Provide the scratch buffer.
            Transcoding::Transcode => self.scratch.payload.for_length(payload_size),
        }
    }

    fn header(&mut self, header_size: usize) -> Result<&mut [u8], ResizeError> {
        match (self.translation.inbound().header, self.loan.header()) {
            // Provide the header buffer of the sample.
            (Transcoding::Passthrough, Some(header)) => header.for_length(header_size),
            // Provide the scratch buffer, transcoded later or waiting for the
            // sample.
            _ => self.scratch.header.for_length(header_size),
        }
    }
}

impl<S: Service, X: SampleTranscoder, LoanError> PendingSample<'_, '_, S, X, LoanError> {
    /// The sample with every region written, decoding what the take parked
    /// in the scratch.
    fn into_sample(self) -> Result<SampleMutUninit<S>, ReceiveError> {
        let origin = origin!("PendingSample::into_sample");

        let PendingSample {
            translation,
            mut loan,
            scratch,
        } = self;

        // Transcode the payload.
        if let PublishSubscribeTranslation::Transcode {
            inbound,
            transcoder,
            ..
        } = translation
            && inbound.payload == Transcoding::Transcode
        {
            fail!(
                from origin,
                when transcoder.payloads().decode(&scratch.payload, &mut loan),
                to ReceiveError,
                "Failed to decode the payload of a message"
            );
        }

        // Transcode the header or copy it out from the scratch.
        let header_size = loan.header_size();
        let mut sample = loan.into_sample();
        // SAFETY: the header size is the service's, as its description
        // states.
        let (mut header, _) = unsafe { regions_mut(&mut sample, header_size) };
        match translation {
            PublishSubscribeTranslation::Transcode {
                inbound,
                transcoder,
                ..
            } if inbound.header == Transcoding::Transcode => {
                fail!(
                    from origin,
                    when transcoder.headers().decode(&scratch.header, &mut header),
                    to ReceiveError,
                    "Failed to decode the header of a message"
                );
            }
            // Copy any header already prepared in the scratch buffer.
            _ if !scratch.header.is_empty() => {
                let header = fail!(
                    from origin,
                    when header.for_length(scratch.header.len()),
                    to ReceiveError,
                    "A header of {} bytes does not fit the service", scratch.header.len()
                );
                header.copy_from_slice(&scratch.header);
            }
            _ => {}
        }

        Ok(sample)
    }
}

/// A sample loaned once its payload has a length.
struct Loan<'a, 'b, S: Service, LoanError> {
    types: &'a PublishSubscribeTypes,
    loan: &'a mut LoanFn<'b, S, LoanError>,
    sample: Option<SampleMutUninit<S>>,
}

impl<'a, 'b, S: Service, LoanError> Loan<'a, 'b, S, LoanError> {
    fn new(types: &'a PublishSubscribeTypes, loan: &'a mut LoanFn<'b, S, LoanError>) -> Self {
        Self {
            types,
            loan,
            sample: None,
        }
    }

    fn header_size(&self) -> usize {
        self.types.user_header.size
    }

    /// The header of the loaned sample, once loaned.
    fn header(&mut self) -> Option<&mut [u8]> {
        let header_size = self.header_size();
        let sample = self.sample.as_mut()?;
        // SAFETY: the header size is the service's, as its description
        // states.
        let (header, _) = unsafe { regions_mut(sample, header_size) };
        Some(header)
    }

    /// The loaned sample. Panics unless a payload was written, a taken
    /// message has one by contract.
    fn into_sample(self) -> SampleMutUninit<S> {
        let origin = origin!("Loan::into_sample");

        let Some(sample) = self.sample else {
            fatal_panic!(
                from origin,
                "No payload was written for a taken message, the endpoints or the transcoder broke its contract"
            );
        };
        sample
    }
}

impl<S: Service, LoanError> Region for Loan<'_, '_, S, LoanError> {
    /// Loans the sample for `payload_size` bytes and hands out the payload.
    fn for_length(&mut self, payload_size: usize) -> Result<&mut [u8], ResizeError> {
        let origin = origin!("Loan::for_length");

        let header_size = self.header_size();
        let sample = match self.sample {
            Some(ref mut sample) => sample,
            None => {
                if !fits(self.types, header_size, payload_size) {
                    fail!(
                        from origin,
                        with ResizeError::Malformed,
                        "A payload of {} bytes does not fit the service description", payload_size
                    );
                }
                let sample = fail!(
                    from origin,
                    when (self.loan)(payload_size),
                    with ResizeError::Exhausted,
                    "Failed to loan a sample for a payload of {} bytes", payload_size
                );
                self.sample.insert(sample)
            }
        };

        // SAFETY: the header size is the service's, as its description
        // states.
        let (_, payload) = unsafe { regions_mut(sample, header_size) };

        // TODO: Resize the loaned sample instead, once slice samples can be
        // resized through the publisher's allocation strategy.
        if payload.len() != payload_size {
            fail!(
                from origin,
                with ResizeError::NotResizable,
                "The sample holds a payload of {} bytes, not {}", payload.len(), payload_size
            );
        }

        Ok(payload)
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
    use iceoryx2_link_backend::wire::publish_subscribe::{Header, Payload, Publisher, Subscriber};

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

    /// What a take writes, and in which order.
    #[derive(Default, Clone, Copy)]
    enum Writes {
        #[default]
        PayloadThenHeader,
        HeaderThenPayload,
        HeaderOnly,
    }

    /// Endpoints holding one pending message and every message published
    /// on them.
    #[derive(Default)]
    struct StubEndpoints {
        pending: Option<Message>,
        published: Vec<Message>,
        writes: Writes,
    }

    impl StubEndpoints {
        fn pending(message: Message) -> Self {
            Self {
                pending: Some(message),
                ..Self::default()
            }
        }

        fn pending_written(message: Message, writes: Writes) -> Self {
            Self {
                writes,
                ..Self::pending(message)
            }
        }

        /// The one message published on the endpoints.
        fn published(&self) -> Message {
            assert_that!(self.published.len(), eq 1);
            self.published[0].clone()
        }
    }

    fn write(
        region: Result<&mut [u8], ResizeError>,
        bytes: &[u8],
    ) -> Result<(), TakeError<Infallible>> {
        match region {
            Ok(into) => {
                into.copy_from_slice(bytes);
                Ok(())
            }
            Err(refusal) => Err(TakeError::Rejected(refusal)),
        }
    }

    impl PublishSubscribeEndpoints for StubEndpoints {
        type Failure = Infallible;

        fn publish(&mut self, header: &[u8], payload: &[u8]) -> Result<(), Self::Failure> {
            self.published.push(message(header, payload));
            Ok(())
        }

        fn take<D: Destination>(&mut self, into: &mut D) -> Result<bool, TakeError<Self::Failure>> {
            let Some(message) = self.pending.take() else {
                return Ok(false);
            };
            match self.writes {
                Writes::PayloadThenHeader => {
                    write(into.payload(message.payload.len()), &message.payload)?;
                    write(into.header(message.header.len()), &message.header)?;
                }
                Writes::HeaderThenPayload => {
                    write(into.header(message.header.len()), &message.header)?;
                    write(into.payload(message.payload.len()), &message.payload)?;
                }
                Writes::HeaderOnly => {
                    write(into.header(message.header.len()), &message.header)?;
                }
            }
            Ok(true)
        }
    }

    /// Reverses the bytes of a payload, drops a header on the way out and
    /// makes one up on the way in.
    struct StubTranscoder;

    fn reversed<R: Region>(bytes: &[u8], into: &mut R) -> Result<(), TranscodeError<Infallible>> {
        let into = into
            .for_length(bytes.len())
            .map_err(TranscodeError::Rejected)?;
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

    impl HeaderTranscoder for StubTranscoder {
        type Failure = Infallible;

        fn encode<R: Region>(
            &self,
            _: &[u8],
            into: &mut R,
        ) -> Result<(), TranscodeError<Self::Failure>> {
            into.for_length(0).map_err(TranscodeError::Rejected)?;
            Ok(())
        }

        fn decode<R: Region>(
            &self,
            _: &[u8],
            into: &mut R,
        ) -> Result<(), TranscodeError<Self::Failure>> {
            let into = into
                .for_length(DECODED_HEADER.len())
                .map_err(TranscodeError::Rejected)?;
            into.copy_from_slice(&DECODED_HEADER);
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

        fn decode<R: Region>(
            &self,
            wire: &[u8],
            into: &mut R,
        ) -> Result<(), TranscodeError<Self::Failure>> {
            reversed(wire, into)
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

    fn types() -> PublishSubscribeTypes {
        PublishSubscribeTypes {
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
            relay.receive(&mut |len| self.loan(len))
        }

        /// A received sample holding `header` and `payload`.
        fn sample(&self, header: &[u8; SIZE], payload: &[u8; SIZE]) -> Sample<local::Service> {
            let mut sample = self.loan(SIZE).expect("sample is loaned");
            // SAFETY: the header size is the service's.
            let (into_header, into_payload) = unsafe { regions_mut(&mut sample, SIZE) };
            into_header.copy_from_slice(header);
            into_payload.copy_from_slice(payload);
            // SAFETY: both regions were written.
            unsafe { sample.assume_init() }
                .send()
                .expect("sample is sent");
            self.subscriber
                .receive()
                .expect("receive succeeds")
                .expect("a sample is received")
        }
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
    fn a_header_written_before_the_payload_reaches_the_sample() {
        let ports = Ports::open();
        let mut relay = relay(
            StubEndpoints::pending_written(message(&HEADER, &PAYLOAD), Writes::HeaderThenPayload),
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

        let received = relay.receive(&mut |_| -> Result<_, ()> { Err(()) }).err();

        assert_that!(received, eq Some(ReceiveError::Loan));
    }

    #[test]
    #[should_panic(expected = "No payload was written for a taken message")]
    fn a_message_without_a_payload_is_a_contract_breach() {
        let ports = Ports::open();
        let mut relay = relay(
            StubEndpoints::pending_written(message(&HEADER, &PAYLOAD), Writes::HeaderOnly),
            PublishSubscribeTranslation::Passthrough,
        );

        let _ = ports.receive(&mut relay);
    }
}
