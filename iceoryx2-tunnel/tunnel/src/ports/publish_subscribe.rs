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
use iceoryx2::port::publisher::PublisherCreateError;
use iceoryx2::port::subscriber::SubscriberCreateError;
use iceoryx2::prelude::AllocationStrategy;
use iceoryx2::service::builder::publish_subscribe::*;
use iceoryx2::service::port_factory::*;
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
    FailedToCreateService,
    FailedToCreatePublisher,
    FailedToCreateSubscriber,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum PropagationError {}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum IngestionError {
    FailedToSendSample,
}

impl From<PublishSubscribeOpenOrCreateError> for CreationError {
    fn from(_: PublishSubscribeOpenOrCreateError) -> Self {
        CreationError::FailedToCreateService
    }
}

impl From<PublisherCreateError> for CreationError {
    fn from(_: PublisherCreateError) -> Self {
        CreationError::FailedToCreatePublisher
    }
}

impl From<SubscriberCreateError> for CreationError {
    fn from(_: SubscriberCreateError) -> Self {
        CreationError::FailedToCreateSubscriber
    }
}

#[derive(Debug)]
pub(crate) struct Ports<S: Service> {
    pub(crate) static_config: StaticConfig,
    pub(crate) publisher: Publisher<S>,
    pub(crate) subscriber: Subscriber<S>,
}

impl<S: Service> Ports<S> {
    pub(crate) fn new(static_config: &StaticConfig, node: &Node<S>) -> Result<Self, CreationError> {
        let port_config = static_config.publish_subscribe();
        let service = unsafe {
            fail!(
                from "Tunnel::setup_publish_subscribe()",
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
                "{}", format!("Failed to open or create publish-subscribe service '{}'", static_config.name())
            )
        };

        let publisher = fail!(
            from "Ports::new",
            when service
                .publisher_builder()
                .allocation_strategy(AllocationStrategy::PowerOfTwo)
                .create(),
            "{}", &format!("Failed to create Publisher for '{}'", service.name())
        );

        let subscriber = fail!(
            from "Ports::new",
            when service.subscriber_builder().create(),
            "{}", &format!("Failed to create Subscriber for '{}'", service.name())
        );

        Ok(Ports {
            static_config: static_config.clone(),
            publisher,
            subscriber,
        })
    }

    /// Receive data from the ports
    pub(crate) fn receive<PropagateFn>(
        &self,
        node_id: &iceoryx2::node::NodeId,
        mut propagate: PropagateFn,
    ) -> Result<(), PropagationError>
    where
        // Propagation function provided by the caller.
        PropagateFn: FnMut(Sample<S>),
    {
        loop {
            match unsafe { self.subscriber.receive_custom_payload() } {
                Ok(Some(sample)) => {
                    debug!(
                        from "Ports::receive",
                        "Received PublishSubscribe({})",
                        self.static_config.name()
                    );

                    if sample.header().node_id() == *node_id {
                        // Ignore samples published by the gateway itself to prevent loopback.
                        continue;
                    }

                    propagate(sample);
                }
                Ok(None) => break,
                Err(e) => {
                    // TODO: Use fail!
                    fatal_panic!("Failed to receive custom payload: {}", e)
                }
            }
        }

        Ok(())
    }

    pub(crate) fn send<IngestFn>(&self, mut ingest: IngestFn) -> Result<(), IngestionError>
    where
        IngestFn: for<'a> FnMut(&'a mut LoanFn<'a, S>) -> Option<SampleMut<S>>,
    {
        let type_details = self
            .static_config
            .publish_subscribe()
            .message_type_details();

        loop {
            let initialized_sample = ingest(&mut |number_of_bytes| {
                let number_of_elements = number_of_bytes / type_details.payload.size();

                match unsafe { self.publisher.loan_custom_payload(number_of_elements) } {
                    Ok(sample_to_initialize) => {
                        return sample_to_initialize;
                    }
                    Err(e) => {
                        fatal_panic!(from "Ports::send", "Failed to loan custom payload: {e}")
                    }
                }
            });

            if let Some(sample) = initialized_sample {
                debug!(from "Ports::send", "Sending: PublishSubscribe({})", self.static_config.name());
                fail!(
                    from "Ports::send",
                    when sample.send(),
                    with IngestionError::FailedToSendSample,
                    "Failed to send ingested payload"
                );
            } else {
                break;
            }
        }

        Ok(())
    }
}
