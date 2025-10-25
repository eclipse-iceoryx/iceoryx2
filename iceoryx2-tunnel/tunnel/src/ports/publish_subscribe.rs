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

use iceoryx2::node::{Node, NodeId};
use iceoryx2::port::LoanError;
use iceoryx2::prelude::AllocationStrategy;
use iceoryx2::service::{static_config::StaticConfig, Service};
use iceoryx2_bb_log::{fail, trace};
use iceoryx2_tunnel_backend::types::publish_subscribe::{
    Header, LoanFn, Payload, Publisher, Sample, SampleMut, Subscriber,
};

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum CreationError {
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
    pub(crate) static_config: StaticConfig,
    pub(crate) publisher: Publisher<S>,
    pub(crate) subscriber: Subscriber<S>,
}

impl<S: Service> PublishSubscribePorts<S> {
    pub(crate) fn new(static_config: &StaticConfig, node: &Node<S>) -> Result<Self, CreationError> {
        let origin = format!(
            "PublishSubscribePorts<{}>::new",
            core::any::type_name::<S>()
        );

        let port_config = static_config.publish_subscribe();
        let service = unsafe {
            fail!(
                from origin,
                when node.service_builder(static_config.name())
                        .publish_subscribe::<Payload>()
                        .user_header::<Header>()
                        .__internal_set_user_header_type_details(
                            &port_config.message_type_details().user_header,
                        )
                        .__internal_set_payload_type_details(
                            &port_config.message_type_details().payload,
                        )
                        .enable_safe_overflow(port_config.has_safe_overflow())
                        .history_size(port_config.history_size())
                        .max_nodes(port_config.max_nodes())
                        .max_publishers(port_config.max_publishers())
                        .max_subscribers(port_config.max_subscribers())
                        .subscriber_max_buffer_size(port_config.subscriber_max_buffer_size())
                        .subscriber_max_borrowed_samples(
                            port_config.subscriber_max_borrowed_samples(),
                        )
                        .open_or_create(),
                with CreationError::Service,
                "Failed to open or create service {}({})", static_config.messaging_pattern(), static_config.name()
            )
        };

        let publisher = fail!(
            from origin,
            when service
                .publisher_builder()
                .allocation_strategy(AllocationStrategy::PowerOfTwo)
                .create(),
            with CreationError::Publisher,
            "Failed to create Publisher for {}({})", static_config.messaging_pattern(), static_config.name()
        );

        let subscriber = fail!(
            from origin,
            when service.subscriber_builder().create(),
            with CreationError::Subscriber,
            "Failed to create Subscriber for {}({})", static_config.messaging_pattern(), static_config.name()
        );

        Ok(PublishSubscribePorts {
            static_config: static_config.clone(),
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

        let type_details = self
            .static_config
            .publish_subscribe()
            .message_type_details();

        loop {
            let sample = ingest(&mut |number_of_bytes| {
                let number_of_elements = number_of_bytes / type_details.payload.size();

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
                    trace!(
                        from self,
                        "Sending {}({})",
                        self.static_config.messaging_pattern(),
                        self.static_config.name()
                    );

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
        node_id: &NodeId,
        mut propagate: PropagateFn,
    ) -> Result<bool, ReceiveError>
    where
        PropagateFn: FnMut(Sample<S>) -> Result<(), E>,
    {
        let mut propagated = false;

        loop {
            let sample = unsafe { self.subscriber.receive_custom_payload() };
            let sample = fail!(
                from self,
                when sample,
                with ReceiveError::CustomPayloadReceive,
                "Failed to receive custom payload to propagate to backend"
            );

            match sample {
                Some(sample) => {
                    trace!(
                        from self,
                        "Received {}({})",
                        self.static_config.messaging_pattern(),
                        self.static_config.name()
                    );

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
