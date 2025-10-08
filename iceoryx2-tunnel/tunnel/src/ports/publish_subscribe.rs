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

use iceoryx2::node::Node;
use iceoryx2::node::NodeId;
use iceoryx2::port::LoanError;
use iceoryx2::prelude::AllocationStrategy;
use iceoryx2::service::static_config::StaticConfig;
use iceoryx2::service::Service;
use iceoryx2_bb_log::debug;
use iceoryx2_bb_log::fail;
use iceoryx2_bb_log::fatal_panic;
use iceoryx2_tunnel_backend::types::publish_subscribe::Header;
use iceoryx2_tunnel_backend::types::publish_subscribe::LoanFn;
use iceoryx2_tunnel_backend::types::publish_subscribe::Payload;
use iceoryx2_tunnel_backend::types::publish_subscribe::Publisher;
use iceoryx2_tunnel_backend::types::publish_subscribe::Sample;
use iceoryx2_tunnel_backend::types::publish_subscribe::SampleMut;
use iceoryx2_tunnel_backend::types::publish_subscribe::Subscriber;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum CreationError {
    Service,
    Publisher,
    Subscriber,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum SendError {
    SampleDelivery,
    PayloadIngestion,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum ReceiveError {
    SamplePropagation,
}

#[derive(Debug)]
pub(crate) struct PublishSubscribePorts<S: Service> {
    pub(crate) static_config: StaticConfig,
    pub(crate) publisher: Publisher<S>,
    pub(crate) subscriber: Subscriber<S>,
}

impl<S: Service> PublishSubscribePorts<S> {
    pub(crate) fn new(static_config: &StaticConfig, node: &Node<S>) -> Result<Self, CreationError> {
        let port_config = static_config.publish_subscribe();
        let service = unsafe {
            fail!(
                from "PublishSubscribePorts::new",
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
                "{}", format!("Failed to open or create service {}({})", static_config.messaging_pattern(), static_config.name())
            )
        };

        let publisher = fail!(
            from "PublishSubscribePorts::new",
            when service
                .publisher_builder()
                .allocation_strategy(AllocationStrategy::PowerOfTwo)
                .create(),
            with CreationError::Publisher,
            "{}", &format!("Failed to create Publisher for {}({})", static_config.messaging_pattern(), static_config.name())
        );

        let subscriber = fail!(
            from "Ports::new",
            when service.subscriber_builder().create(),
            with CreationError::Subscriber,
            "{}", &format!("Failed to create Subscriber for {}({})", static_config.messaging_pattern(), static_config.name())
        );

        Ok(PublishSubscribePorts {
            static_config: static_config.clone(),
            publisher,
            subscriber,
        })
    }

    pub(crate) fn send<IngestFn, IngestError>(&self, mut ingest: IngestFn) -> Result<(), SendError>
    where
        IngestFn: for<'a> FnMut(
            &'a mut LoanFn<'a, S, LoanError>,
        ) -> Result<Option<SampleMut<S>>, IngestError>,
    {
        let type_details = self
            .static_config
            .publish_subscribe()
            .message_type_details();

        loop {
            let sample = ingest(&mut |number_of_bytes| {
                let number_of_elements = number_of_bytes / type_details.payload.size();

                match unsafe { self.publisher.loan_custom_payload(number_of_elements) } {
                    Ok(sample_to_initialize) => Ok(sample_to_initialize),
                    Err(e) => {
                        // This should never happen?
                        fatal_panic!(from "PublishSubscribePorts::send", "Failed to loan custom payload: {e}")
                    }
                }
            });

            let sample = fail!(
                from "PublishSubscribePorts::send",
                when sample,
                with SendError::PayloadIngestion,
                "Failed to ingest payload from backend"
            );

            match sample {
                Some(sample) => {
                    debug!(from "PublishSubscribePorts::send", "Sending: PublishSubscribe({})", self.static_config.name());
                    fail!(
                        from "Ports::send",
                        when sample.send(),
                        with SendError::SampleDelivery,
                        "Failed to send ingested payload"
                    );
                }
                None => break,
            }
        }

        Ok(())
    }

    pub(crate) fn receive<PropagateFn, E>(
        &self,
        node_id: &NodeId,
        mut propagate: PropagateFn,
    ) -> Result<(), ReceiveError>
    where
        PropagateFn: FnMut(Sample<S>) -> Result<(), E>,
    {
        loop {
            match unsafe { self.subscriber.receive_custom_payload() } {
                Ok(Some(sample)) => {
                    debug!(
                        from "PublishSubscribePorts::receive",
                        "Received PublishSubscribe({})",
                        self.static_config.name()
                    );

                    if sample.header().node_id() == *node_id {
                        // Ignore samples published by the gateway itself to prevent loopback.
                        continue;
                    }

                    fail!(
                        from "PublishSubscribePorts::receive",
                        when propagate(sample),
                        with ReceiveError::SamplePropagation,
                        "Failed to propagate sample"
                    );
                }
                Ok(None) => break,
                Err(e) => {
                    // This should never happen?
                    fatal_panic!(from "PublishSubscribePorts::receive", "Failed to receive custom payload: {}", e)
                }
            }
        }

        Ok(())
    }
}
