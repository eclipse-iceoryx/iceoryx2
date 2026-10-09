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

use core::marker::PhantomData;

use iceoryx2::service::Service;
use iceoryx2_link_adapter::Mapping;
use iceoryx2_link_adapter::{
    Adapter, EndpointDescription, EndpointTypes, LoanableSample, PublishSubscribeEndpoints,
    SampleBytesRef, TakeOutcome,
};
use iceoryx2_link_adapter::{SampleShape, TranscodesSamples, Translator};
use iceoryx2_link_backend::relay::{PublishSubscribeRelay, ReceiveOutcome, RelayBuilder};
use iceoryx2_link_backend::service_description::{PublishSubscribeDescription, SampleTypes};
use iceoryx2_link_backend::wire::publish_subscribe::Sample;
use iceoryx2_link_backend::wire::publish_subscribe::payload_bytes;
use iceoryx2_link_backend::wire::sample::user_header_bytes;
use iceoryx2_log::{fail, fatal_panic, origin};

use crate::relay::transcoding::Transcoding;
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
pub struct Relay<S, E, X: Translator<SampleShape, Transcoders: TranscodesSamples>> {
    endpoints: E,
    transcoding: Transcoding<X::Transcoders>,
    header_size: usize,
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
            transcoding: Transcoding::new(transcoders, &types),
            header_size: types.user_header.size,
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
            header: unsafe { user_header_bytes(sample.user_header(), self.header_size) },
            payload: payload_bytes(sample),
        };
        let wire = fail!(
            from origin,
            when self.transcoding.encode(local),
            "Failed to encode a sample"
        );

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
        let origin = origin!("Relay::receive");

        let mut decode = self.transcoding.decode(loanable);
        let outcome = fail!(
            from origin,
            when self.endpoints.take(&mut decode.input()),
            with ReceiveError::Endpoints,
            "Failed to take a message"
        );
        match outcome {
            TakeOutcome::Taken => {
                let sample = fail!(
                    from origin,
                    when decode.output(),
                    "Failed to decode a message"
                );
                Ok(ReceiveOutcome::Sample(sample))
            }
            TakeOutcome::Declined => Err(decode.refusal()),
            TakeOutcome::Skipped => Ok(ReceiveOutcome::Skipped),
            TakeOutcome::Empty => Ok(ReceiveOutcome::Empty),
        }
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
    use iceoryx2_link_adapter::{
        SampleLengths, SampleTranscoders, TakeDestination, TranscodeError, Transcoder,
    };
    use iceoryx2_link_backend::service_description::TypeDescription;
    use iceoryx2_link_backend::wire::publish_subscribe::{
        Publisher, SampleMut, SampleMutUninit, Subscriber,
    };
    use iceoryx2_link_backend::wire::region::{BackwardRegion, ForwardRegion, Region, ResizeError};
    use iceoryx2_link_backend::wire::sample::{Header, Payload};
    use iceoryx2_link_backend::wire::sample::{
        LoanFn, UnloanedSample, payload_bytes_mut, user_header_bytes_mut,
    };

    use crate::relay::transcoding::PendingLoan;

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
        loan: &mut LoanFn<'_, SampleMutUninit<local::Service>, LoanError>,
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
        let mut sut: PendingLoan<UnloanedSample<'_, SampleMutUninit<local::Service>, LoanError>> =
            PendingLoan::new(UnloanedSample::new(&types, &mut loan));

        let resized = BackwardRegion::resize(&mut sut.backward(), SIZE);

        assert_that!(resized, eq Err(ResizeError::DirectionUnsupported));
        assert_that!(sut.refusal(), eq ReceiveError::Transcode);
    }
}
