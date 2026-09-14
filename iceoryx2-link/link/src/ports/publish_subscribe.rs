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

use iceoryx2::identifiers::UniqueNodeId;
use iceoryx2::node::Node;
use iceoryx2::port::LoanError;
use iceoryx2::prelude::AllocationStrategy;
use iceoryx2::service::Service;
use iceoryx2::service::builder::publish_subscribe;
use iceoryx2::service::header::payload_header::PayloadHeader;
use iceoryx2::service::service_name::ServiceName;
use iceoryx2::service::static_config::message_type_details::{TypeDetail, TypeVariant};
use iceoryx2_link_backend::description::{
    PublishSubscribeSettings, PublishSubscribeTypes, TypeDescription,
};
use iceoryx2_link_backend::origin;
use iceoryx2_link_backend::wire::publish_subscribe::{
    Header, LoanFn, Payload, Publisher, Sample, SampleMut, Subscriber,
};
use iceoryx2_log::fail;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum CreationError {
    TypeDetails,
    Service,
    Publisher,
    Subscriber,
}

impl core::fmt::Display for CreationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "CreationError::{self:?}")
    }
}

impl core::error::Error for CreationError {}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum SendError {
    Ingestion,
    Delivery,
}

impl core::fmt::Display for SendError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "SendError::{self:?}")
    }
}

impl core::error::Error for SendError {}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum ReceiveError {
    Receive,
    Propagation,
}

impl core::fmt::Display for ReceiveError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ReceiveError::{self:?}")
    }
}

impl core::error::Error for ReceiveError {}

/// The link's own ports on a publish-subscribe service, opened from its
/// description with the exact types and settings.
#[derive(Debug)]
pub(crate) struct PublishSubscribePorts<S: Service> {
    name: ServiceName,
    payload: TypeDescription,
    publisher: Publisher<S>,
    subscriber: Subscriber<S>,
}

impl<S: Service> PublishSubscribePorts<S> {
    pub(crate) fn open(
        node: &Node<S>,
        name: &ServiceName,
        settings: &PublishSubscribeSettings,
        types: &PublishSubscribeTypes,
    ) -> Result<Self, CreationError> {
        let origin = origin!("PublishSubscribePorts::open");
        let payload = fail!(
            from origin,
            when TypeDetail::try_from(&types.payload),
            with CreationError::TypeDetails,
            "Payload type of {} cannot be represented as a type detail", name
        );
        let user_header = fail!(
            from origin,
            when TypeDetail::try_from(&types.user_header),
            with CreationError::TypeDetails,
            "User header type of {} cannot be represented as a type detail", name
        );

        // SAFETY: the type details come from a description of this exact
        // service, so the untyped markers stand in for the real types.
        let builder = unsafe {
            node.service_builder(name)
                .publish_subscribe::<Payload>()
                .user_header::<Header>()
                .__internal_set_user_header_type_details(&user_header)
                .__internal_set_payload_type_details(&payload)
        };
        let service = fail!(
            from origin,
            when apply_settings(builder, settings).open_or_create(),
            with CreationError::Service,
            "Failed to open or create {}", name
        );
        let publisher = fail!(
            from origin,
            when service
                .publisher_builder()
                .allocation_strategy(AllocationStrategy::PowerOfTwo)
                .create(),
            with CreationError::Publisher,
            "Failed to create the publisher of {}", name
        );
        let subscriber = fail!(
            from origin,
            when service.subscriber_builder().create(),
            with CreationError::Subscriber,
            "Failed to create the subscriber of {}", name
        );

        Ok(Self {
            name: *name,
            payload: types.payload.clone(),
            publisher,
            subscriber,
        })
    }

    /// Hands every pending sample not published by `own_node` to
    /// `propagate`.
    ///
    /// Returns true when payloads are propagated.
    pub(crate) fn receive<E>(
        &self,
        own_node: &UniqueNodeId,
        mut propagate: impl FnMut(Sample<S>) -> Result<(), E>,
    ) -> Result<bool, ReceiveError> {
        let origin = origin!("PublishSubscribePorts::receive");
        let mut received = false;
        loop {
            let sample = fail!(
                from origin,
                when self.subscriber.receive(),
                with ReceiveError::Receive,
                "Failed to receive from {}", self.name
            );
            let Some(sample) = sample else {
                break;
            };
            // The link's own publications are what it ingested from the
            // opposing side. Propagating them back would loop.
            if sample.header().node_id() == *own_node {
                continue;
            }
            fail!(
                from origin,
                when propagate(sample),
                with ReceiveError::Propagation,
                "Failed to propagate a sample of {}", self.name
            );
            received = true;
        }
        Ok(received)
    }

    /// Publishes every sample `ingest` fills into a loan, until it has
    /// nothing more.
    ///
    /// Returns true when payloads are ingested.
    pub(crate) fn send<E>(
        &self,
        mut ingest: impl for<'a> FnMut(
            &'a mut LoanFn<'a, S, LoanError>,
        ) -> Result<Option<SampleMut<S>>, E>,
    ) -> Result<bool, SendError> {
        let origin = origin!("PublishSubscribePorts::send");
        let mut sent = false;
        loop {
            let sample = fail!(
                from origin,
                when ingest(&mut |number_of_bytes| self.loan(number_of_bytes)),
                with SendError::Ingestion,
                "Failed to ingest a sample for {}", self.name
            );
            let Some(sample) = sample else {
                break;
            };
            fail!(
                from origin,
                when sample.send(),
                with SendError::Delivery,
                "Failed to send a sample of {}", self.name
            );
            sent = true;
        }
        Ok(sent)
    }

    /// Loans a sample holding `number_of_bytes` of payload, which must be
    /// whole elements of the payload type.
    fn loan(
        &self,
        number_of_bytes: usize,
    ) -> Result<iceoryx2_link_backend::wire::publish_subscribe::SampleMutUninit<S>, LoanError> {
        let origin = origin!("PublishSubscribePorts::loan");
        let Some(number_of_elements) = number_of_elements(&self.payload, number_of_bytes) else {
            fail!(
                from origin,
                with LoanError::InternalFailure,
                "A loan of {} bytes does not hold whole elements of {}", number_of_bytes, self.name
            );
        };
        // SAFETY: the publisher was created for the untyped payload marker
        // with this service's real type details.
        let sample = fail!(
            from origin,
            when unsafe { self.publisher.loan_custom_payload(number_of_elements) },
            "Failed to loan a sample of {}", self.name
        );
        Ok(sample)
    }
}

/// The number of payload elements `number_of_bytes` holds, if whole.
fn number_of_elements(payload: &TypeDescription, number_of_bytes: usize) -> Option<usize> {
    match payload.variant {
        TypeVariant::FixedSize => (number_of_bytes == payload.size).then_some(1),
        TypeVariant::Dynamic => {
            let whole = payload.size != 0 && number_of_bytes.is_multiple_of(payload.size);
            whole.then(|| number_of_bytes / payload.size)
        }
    }
}

fn apply_settings<S: Service>(
    builder: publish_subscribe::Builder<Payload, Header, S>,
    settings: &PublishSubscribeSettings,
) -> publish_subscribe::Builder<Payload, Header, S> {
    builder
        .max_subscribers(settings.max_subscribers)
        .max_publishers(settings.max_publishers)
        .max_nodes(settings.max_nodes)
        .history_size(settings.history_size)
        .subscriber_max_buffer_size(settings.subscriber_max_buffer_size)
        .subscriber_max_borrowed_samples(settings.subscriber_max_borrowed_samples)
        .enable_safe_overflow(settings.safe_overflow)
}

#[cfg(test)]
mod tests {
    use super::*;

    use iceoryx2::node::NodeBuilder;
    use iceoryx2::service::local;
    use iceoryx2::service::messaging_pattern::MessagingPattern;
    use iceoryx2::testing::{generate_isolated_config, generate_service_name};
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_link_backend::description::{PatternSettings, ServiceDescription, ServiceTypes};

    const ALIGNMENT: usize = 1;

    fn payload(variant: TypeVariant, size: usize) -> TypeDescription {
        TypeDescription {
            variant,
            type_name: "test_type".into(),
            size,
            alignment: ALIGNMENT,
        }
    }

    #[test]
    fn fixed_size_payload_holds_exactly_one_element() {
        const ELEMENT_SIZE: usize = 8;

        let payload = payload(TypeVariant::FixedSize, ELEMENT_SIZE);
        assert_that!(number_of_elements(&payload, ELEMENT_SIZE), eq Some(1));
        assert_that!(number_of_elements(&payload, 0), eq None);
        assert_that!(number_of_elements(&payload, 2 * ELEMENT_SIZE), eq None);
    }

    #[test]
    fn dynamic_payload_holds_whole_elements() {
        const ELEMENT_SIZE: usize = 4;

        let payload = payload(TypeVariant::Dynamic, ELEMENT_SIZE);
        assert_that!(number_of_elements(&payload, 0), eq Some(0));
        assert_that!(number_of_elements(&payload, 3 * ELEMENT_SIZE), eq Some(3));
        assert_that!(number_of_elements(&payload, ELEMENT_SIZE + ELEMENT_SIZE / 2), eq None);
    }

    #[test]
    fn samples_round_trip_through_the_ports() {
        const PUBLISHED: u64 = 42;
        const INGESTED: u64 = 7;

        let config = generate_isolated_config();
        let app_node = NodeBuilder::new()
            .config(&config)
            .create::<local::Service>()
            .expect("node is created");
        let link_node = NodeBuilder::new()
            .config(&config)
            .create::<local::Service>()
            .expect("node is created");
        let service_name = generate_service_name();
        let app_service = app_node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .create()
            .expect("service is created");
        let app_publisher = app_service
            .publisher_builder()
            .create()
            .expect("publisher is created");
        let app_subscriber = app_service
            .subscriber_builder()
            .create()
            .expect("subscriber is created");

        let static_config =
            local::Service::details(&service_name, &config, MessagingPattern::PublishSubscribe)
                .expect("details are readable")
                .expect("service exists")
                .static_details;
        let description = ServiceDescription::try_from(&static_config).expect("carried pattern");
        let PatternSettings::PublishSubscribe(settings) = &description.settings().pattern else {
            panic!("a publish-subscribe service");
        };
        let ServiceTypes::PublishSubscribe(types) = &description.types() else {
            panic!("a publish-subscribe service");
        };
        let sut = PublishSubscribePorts::open(&link_node, &service_name, settings, types)
            .expect("ports open on the existing service");

        // Out of the local system: the app publishes, the ports receive.
        app_publisher.send_copy(PUBLISHED).expect("sample is sent");
        let mut received = alloc::vec::Vec::new();
        let any = sut
            .receive(link_node.id(), |sample| {
                received.push(u64::from_ne_bytes(
                    iceoryx2_link_backend::wire::publish_subscribe::payload_bytes(sample.payload())
                        .try_into()
                        .expect("payload is a u64"),
                ));
                Ok::<(), ()>(())
            })
            .expect("receive succeeds");
        assert_that!(any, eq true);
        assert_that!(received, eq alloc::vec![PUBLISHED]);

        // Into the local system: the ports ingest, the app receives.
        let mut ingested_once = false;
        let any = sut
            .send(|loan| {
                if ingested_once {
                    return Ok::<_, ()>(None);
                }
                ingested_once = true;
                let sample = loan(core::mem::size_of::<u64>()).expect("loan succeeds");
                // SAFETY: the loan holds exactly one u64 and the header is
                // zero sized.
                let sample = unsafe {
                    iceoryx2_link_backend::wire::publish_subscribe::initialize_sample(
                        sample,
                        &[],
                        &INGESTED.to_ne_bytes(),
                    )
                };
                Ok(Some(sample))
            })
            .expect("send succeeds");
        assert_that!(any, eq true);
        // The app's subscriber also saw the app's own publication first.
        let mut at_app = alloc::vec::Vec::new();
        while let Some(sample) = app_subscriber.receive().expect("receive succeeds") {
            at_app.push(*sample.payload());
        }
        assert_that!(at_app, eq alloc::vec![PUBLISHED, INGESTED]);

        // The ports do not receive back what they sent themselves.
        let any = sut
            .receive(link_node.id(), |_| Ok::<(), ()>(()))
            .expect("receive succeeds");
        assert_that!(any, eq false);
    }
}
