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

use core::mem::MaybeUninit;

use iceoryx2::port::publisher::Publisher;
use iceoryx2::port::publisher::PublisherCreateError;
use iceoryx2::port::subscriber::Subscriber;
use iceoryx2::port::subscriber::SubscriberCreateError;
use iceoryx2::prelude::AllocationStrategy;
use iceoryx2::sample::Sample;
use iceoryx2::sample_mut::SampleMut;
use iceoryx2::sample_mut_uninit::SampleMutUninit;
use iceoryx2::service::builder::publish_subscribe::*;
use iceoryx2::service::builder::CustomHeaderMarker;
use iceoryx2::service::builder::CustomPayloadMarker;
use iceoryx2::service::port_factory::*;
use iceoryx2::service::static_config::StaticConfig;
use iceoryx2::service::Service;
use iceoryx2_bb_log::debug;
use iceoryx2_bb_log::fail;
use iceoryx2_bb_log::fatal_panic;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum CreationError {
    Service,
    Publisher,
    Subscriber,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum PropagationError {}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum IngestionError {
    FailedToSendSample,
}

impl From<PublishSubscribeOpenOrCreateError> for CreationError {
    fn from(_: PublishSubscribeOpenOrCreateError) -> Self {
        CreationError::Service
    }
}

impl From<PublisherCreateError> for CreationError {
    fn from(_: PublisherCreateError) -> Self {
        CreationError::Publisher
    }
}

impl From<SubscriberCreateError> for CreationError {
    fn from(_: SubscriberCreateError) -> Self {
        CreationError::Subscriber
    }
}

#[derive(Debug)]
pub(crate) struct Ports<S: Service> {
    pub(crate) static_config: StaticConfig,
    pub(crate) publisher: Publisher<S, [CustomPayloadMarker], CustomHeaderMarker>,
    pub(crate) subscriber: Subscriber<S, [CustomPayloadMarker], CustomHeaderMarker>,
}

impl<S: Service> Ports<S> {
    pub(crate) fn new(
        static_config: &StaticConfig,
        service: &publish_subscribe::PortFactory<S, [CustomPayloadMarker], CustomHeaderMarker>,
    ) -> Result<Self, CreationError> {
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
        PropagateFn: FnMut(Sample<S, [CustomPayloadMarker], CustomHeaderMarker>),
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

    // TODO: Pass ownership to ingest, then reclaim it..?
    pub(crate) fn send<IngestFn>(&self, mut ingest: IngestFn) -> Result<(), IngestionError>
    where
        IngestFn: for<'a> FnMut(
            &'a mut LoanFn<'a, S>,
        )
            -> Option<SampleMut<S, [CustomPayloadMarker], CustomHeaderMarker>>,
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

pub type LoanFn<'a, S> = dyn FnMut(
        usize,
    )
        -> SampleMutUninit<S, [core::mem::MaybeUninit<CustomPayloadMarker>], CustomHeaderMarker>
    + 'a;
