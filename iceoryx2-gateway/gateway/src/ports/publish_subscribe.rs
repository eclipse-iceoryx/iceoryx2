// Copyright (c) 2025 Contributors to the Eclipse Foundation
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

use alloc::format;

use iceoryx2::identifiers::UniqueNodeId;
use iceoryx2::node::Node;
use iceoryx2::port::LoanError;
use iceoryx2::prelude::AllocationStrategy;
use iceoryx2::service::Service;
use iceoryx2::service::builder::publish_subscribe;
use iceoryx2::service::header::payload_header::PayloadHeader;
use iceoryx2::service::service_name::ServiceName;
use iceoryx2::service::static_config::message_type_details::{TypeDetail, TypeVariant};
use iceoryx2_gateway_backend::types::publish_subscribe::{
    Header, LoanFn, Payload, Publisher, Sample, SampleMut, Subscriber,
};
use iceoryx2_gateway_backend::types::service_description::{
    PortSettings, PublishSubscribeDescription, PublishSubscribeSettings, TypeDescription,
};
use iceoryx2_log::{fail, trace};

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
    SampleDelivery,
    PayloadIngestion,
}

impl core::fmt::Display for SendError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "SendError::{self:?}")
    }
}

impl core::error::Error for SendError {}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum ReceiveError {
    CustomPayloadReceive,
    SamplePropagation,
}

impl core::fmt::Display for ReceiveError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ReceiveError::{self:?}")
    }
}

impl core::error::Error for ReceiveError {}

#[derive(Debug)]
pub(crate) struct PublishSubscribePorts<S: Service> {
    pub(crate) name: ServiceName,
    pub(crate) description: PublishSubscribeDescription,
    pub(crate) publisher: Publisher<S>,
    pub(crate) subscriber: Subscriber<S>,
}

impl<S: Service> PublishSubscribePorts<S> {
    pub(crate) fn new(
        name: &ServiceName,
        description: &PublishSubscribeDescription,
        node: &Node<S>,
    ) -> Result<Self, CreationError> {
        let origin = format!(
            "PublishSubscribePorts<{}>::new",
            core::any::type_name::<S>()
        );

        let payload_details = fail!(
            from origin,
            when TypeDetail::try_from(&description.payload),
            with CreationError::TypeDetails,
            "Payload type of PublishSubscribe({}) cannot be represented as a TypeDetail", name
        );
        let user_header_details = fail!(
            from origin,
            when TypeDetail::try_from(&description.user_header),
            with CreationError::TypeDetails,
            "User header type of PublishSubscribe({}) cannot be represented as a TypeDetail", name
        );

        let builder = unsafe {
            node.service_builder(name)
                .publish_subscribe::<Payload>()
                .user_header::<Header>()
                .__internal_set_user_header_type_details(&user_header_details)
                .__internal_set_payload_type_details(&payload_details)
        };
        let builder = match &description.settings {
            PortSettings::Value(settings) => apply_settings(builder, settings),
            PortSettings::LocalDefaults => builder,
        };

        let service = fail!(
            from origin,
            when builder.open_or_create(),
            with CreationError::Service,
            "Failed to open or create service PublishSubscribe({})", name
        );
        let publisher = fail!(
            from origin,
            when service
                .publisher_builder()
                .allocation_strategy(AllocationStrategy::PowerOfTwo)
                .create(),
            with CreationError::Publisher,
            "Failed to create Publisher for PublishSubscribe({})", name
        );
        let subscriber = fail!(
            from origin,
            when service.subscriber_builder().create(),
            with CreationError::Subscriber,
            "Failed to create Subscriber for PublishSubscribe({})", name
        );

        Ok(PublishSubscribePorts {
            name: *name,
            description: description.clone(),
            publisher,
            subscriber,
        })
    }

    pub(crate) fn send<IngestFn, IngestError>(
        &self,
        mut ingest: IngestFn,
    ) -> Result<bool, SendError>
    where
        IngestFn: for<'a> FnMut(
            &'a mut LoanFn<'a, S, LoanError>,
        ) -> Result<Option<SampleMut<S>>, IngestError>,
    {
        let mut ingested = false;

        loop {
            let sample = ingest(&mut |number_of_bytes| {
                let Some(number_of_elements) =
                    number_of_elements(&self.description.payload, number_of_bytes)
                else {
                    fail!(
                        from self,
                        with LoanError::InternalFailure,
                        "Backend requested a loan of {} bytes which does not hold whole payload elements",
                        number_of_bytes
                    );
                };

                let sample = unsafe { self.publisher.loan_custom_payload(number_of_elements) };
                let sample = fail!(
                    from self,
                    when sample,
                    "Failed to loan custom payload for ingestion from backend"
                );

                Ok(sample)
            });

            let sample = fail!(
                from self,
                when sample,
                with SendError::PayloadIngestion,
                "Failed to ingest payload from backend"
            );

            match sample {
                Some(sample) => {
                    trace!(from self, "Sending PublishSubscribe({})", self.name);

                    fail!(
                        from self,
                        when sample.send(),
                        with SendError::SampleDelivery,
                        "Failed to send ingested payload"
                    );

                    ingested = true;
                }
                None => break,
            }
        }

        Ok(ingested)
    }

    pub(crate) fn receive<PropagateFn, E>(
        &self,
        node_id: &UniqueNodeId,
        mut propagate: PropagateFn,
    ) -> Result<bool, ReceiveError>
    where
        PropagateFn: FnMut(Sample<S>) -> Result<(), E>,
    {
        let mut propagated = false;

        loop {
            let sample = self.subscriber.receive();
            let sample = fail!(
                from self,
                when sample,
                with ReceiveError::CustomPayloadReceive,
                "Failed to receive custom payload to propagate to backend"
            );

            match sample {
                Some(sample) => {
                    trace!(from self, "Received PublishSubscribe({})", self.name);

                    if sample.header().node_id() == *node_id {
                        // Ignore samples published by the gateway itself to avoid loopback.
                        continue;
                    }

                    fail!(
                        from self,
                        when propagate(sample),
                        with ReceiveError::SamplePropagation,
                        "Failed to propagate sample"
                    );

                    propagated = true;
                }
                None => break,
            }
        }

        Ok(propagated)
    }
}

/// Number of payload elements that exactly fill the given number of bytes.
fn number_of_elements(payload: &TypeDescription, number_of_bytes: usize) -> Option<usize> {
    match payload.variant {
        TypeVariant::FixedSize => (number_of_bytes == payload.size).then_some(1),
        TypeVariant::Dynamic => {
            let fills_whole_elements =
                payload.size != 0 && number_of_bytes.is_multiple_of(payload.size);
            fills_whole_elements.then(|| number_of_bytes / payload.size)
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

    use iceoryx2_bb_testing::assert_that;

    fn payload(variant: TypeVariant, size: usize) -> TypeDescription {
        TypeDescription {
            variant,
            type_name: "test_type".into(),
            size,
            alignment: 1,
        }
    }

    #[test]
    fn fixed_size_payload_holds_one_element() {
        let payload = payload(TypeVariant::FixedSize, 8);

        assert_that!(number_of_elements(&payload, 8), eq Some(1));
    }

    #[test]
    fn zero_sized_fixed_size_payload_holds_one_element() {
        let payload = payload(TypeVariant::FixedSize, 0);

        assert_that!(number_of_elements(&payload, 0), eq Some(1));
    }

    #[test]
    fn fixed_size_payload_rejects_other_byte_counts() {
        let payload = payload(TypeVariant::FixedSize, 8);

        assert_that!(number_of_elements(&payload, 0), eq None);
        assert_that!(number_of_elements(&payload, 12), eq None);
        assert_that!(number_of_elements(&payload, 16), eq None);
    }

    #[test]
    fn dynamic_payload_holds_whole_elements() {
        let payload = payload(TypeVariant::Dynamic, 4);

        assert_that!(number_of_elements(&payload, 0), eq Some(0));
        assert_that!(number_of_elements(&payload, 4), eq Some(1));
        assert_that!(number_of_elements(&payload, 12), eq Some(3));
    }

    #[test]
    fn dynamic_payload_rejects_partial_elements() {
        let payload = payload(TypeVariant::Dynamic, 4);

        assert_that!(number_of_elements(&payload, 6), eq None);
    }

    #[test]
    fn zero_sized_dynamic_payload_rejects_every_byte_count() {
        let payload = payload(TypeVariant::Dynamic, 0);

        assert_that!(number_of_elements(&payload, 0), eq None);
        assert_that!(number_of_elements(&payload, 4), eq None);
    }
}
