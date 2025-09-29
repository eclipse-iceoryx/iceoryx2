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
use iceoryx2::sample_mut_uninit::SampleMutUninit;
use iceoryx2::service::builder::publish_subscribe::*;
use iceoryx2::service::builder::CustomHeaderMarker;
use iceoryx2::service::builder::CustomPayloadMarker;
use iceoryx2::service::port_factory::*;
use iceoryx2::service::Service;
use iceoryx2_bb_log::fail;
use iceoryx2_bb_log::fatal_panic;

use crate::Relay;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Error {
    Creation,
    Propagation,
}

impl From<PublishSubscribeOpenOrCreateError> for Error {
    fn from(_: PublishSubscribeOpenOrCreateError) -> Self {
        Error::Creation
    }
}

impl From<PublisherCreateError> for Error {
    fn from(_: PublisherCreateError) -> Self {
        Error::Creation
    }
}

impl From<SubscriberCreateError> for Error {
    fn from(_: SubscriberCreateError) -> Self {
        Error::Creation
    }
}

pub(crate) struct Ports<S: Service> {
    pub(crate) publisher: Publisher<S, [CustomPayloadMarker], CustomHeaderMarker>,
    pub(crate) subscriber: Subscriber<S, [CustomPayloadMarker], CustomHeaderMarker>,
}

impl<S: Service> Ports<S> {
    pub(crate) fn new(
        service: &publish_subscribe::PortFactory<S, [CustomPayloadMarker], CustomHeaderMarker>,
    ) -> Result<Self, Error> {
        let publisher = fail!(
            from "PublishSubscribePorts<S>::new",
            when service
                .publisher_builder()
                .allocation_strategy(AllocationStrategy::PowerOfTwo)
                .create(),
            "{}", &format!("Failed to create Publisher for '{}'", service.name())
        );

        let subscriber = fail!(
            from "PublishSubscribePorts<S>::new",
            when service.subscriber_builder().create(),
            "{}", &format!("Failed to create Subscriber for '{}'", service.name())
        );

        Ok(Ports {
            publisher,
            subscriber,
        })
    }

    pub(crate) fn propagate(
        &self,
        node_id: &iceoryx2::node::NodeId,
        relay: &Box<dyn Relay>,
    ) -> Result<(), Error> {
        loop {
            match unsafe { self.subscriber.receive_custom_payload() } {
                Ok(Some(sample)) => {
                    if sample.header().node_id() == *node_id {
                        // Ignore samples published by the gateway itself to prevent loopback.
                        continue;
                    }
                    let ptr = sample.payload().as_ptr() as *const u8;
                    let len = sample.len();

                    relay.propagate(ptr, len);
                }
                Ok(None) => break,
                Err(e) => fatal_panic!("Failed to receive custom payload: {}", e),
            }
        }

        Ok(())
    }

    pub(crate) fn ingest(&self, relay: &Box<dyn Relay>) -> Result<(), Error>
    where
        S: Service,
    {
        let payload_size = 1; // TODO: Get from PortFactory

        loop {
            let mut loaned_sample: Option<
                SampleMutUninit<S, [MaybeUninit<CustomPayloadMarker>], CustomHeaderMarker>,
            > = None;

            let ingested = relay.ingest(&mut |number_of_bytes| {
                let number_of_elements = number_of_bytes / payload_size;

                let (ptr, len) =
                    match unsafe { self.publisher.loan_custom_payload(number_of_elements) } {
                        Ok(mut sample) => {
                            let payload = sample.payload_mut();
                            let ptr = payload.as_mut_ptr() as *mut u8;
                            let len = payload.len();

                            loaned_sample = Some(sample);

                            (ptr, len)
                        }
                        Err(e) => fatal_panic!("Failed to loan custom payload: {e}"),
                    };

                (ptr, len)
            });

            if ingested {
                if let Some(sample) = loaned_sample {
                    let sample = unsafe { sample.assume_init() };
                    fail!(
                        from "PublishSubscribePorts<S>::ingest",
                        when sample.send(),
                        with Error::Propagation,
                        "Failed to send ingested payload"
                    );
                }
            } else {
                break;
            }
        }

        Ok(())
    }
}
