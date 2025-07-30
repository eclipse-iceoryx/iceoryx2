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

use crate::middleware;
use crate::Channel;
use crate::PropagationError;

use iceoryx2::port::publisher::Publisher as IceoryxPublisher;
use iceoryx2::service::builder::CustomHeaderMarker;
use iceoryx2::service::builder::CustomPayloadMarker;
use iceoryx2::service::port_factory::publish_subscribe::PortFactory as IceoryxPublishSubscribeService;
use iceoryx2::service::static_config::StaticConfig as IceoryxServiceConfig;
use iceoryx2_bb_log::fail;
use iceoryx2_bb_log::fatal_panic;
use iceoryx2_bb_log::info;

use zenoh::handlers::FifoChannelHandler;
use zenoh::pubsub::Subscriber as ZenohSubscriber;
use zenoh::sample::Sample;
use zenoh::Session as ZenohSession;

// TODO: More granularity in errors
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum CreationError {
    Error,
}

impl core::fmt::Display for CreationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> std::fmt::Result {
        core::write!(f, "CreationError::{self:?}")
    }
}

impl core::error::Error for CreationError {}

/// A channel for propagating remote `iceoryx2` publish-subscribe payloads to local subscribers.
#[derive(Debug)]
pub(crate) struct SubscriberChannel<ServiceType: iceoryx2::service::Service> {
    iox_service_config: IceoryxServiceConfig,
    iox_publisher: IceoryxPublisher<ServiceType, [CustomPayloadMarker], CustomHeaderMarker>,
    z_subscriber: ZenohSubscriber<FifoChannelHandler<Sample>>,
}

impl<ServiceType: iceoryx2::service::Service> SubscriberChannel<ServiceType> {
    // Creates an inbound channel from remote hosts for publish-subscribe payloads for a
    // particular service.
    pub fn create(
        iox_service_config: &IceoryxServiceConfig,
        iox_publish_subscribe_service: &IceoryxPublishSubscribeService<
            ServiceType,
            [CustomPayloadMarker],
            CustomHeaderMarker,
        >,
        z_session: &ZenohSession,
    ) -> Result<Self, CreationError> {
        info!(
            "CREATE SubscriberChannel {} [{}]",
            iox_service_config.service_id().as_str(),
            iox_service_config.name()
        );

        let iox_publisher = fail!(
            from "SubscriberChannel::create()",
            when middleware::iceoryx::create_publisher::<ServiceType>(iox_publish_subscribe_service),
            with CreationError::Error,
            "failed to create iceoryx publisher to propagate remote payloads to local subscribers"
        );

        let z_subscriber = fail!(
            from "SubscriberChannel::create()",
            when middleware::zenoh::create_subscriber(z_session, iox_service_config),
            with CreationError::Error,
            "failed to create Zenoh subscriber to receive remote payloads"
        );

        Ok(Self {
            iox_service_config: iox_service_config.clone(),
            iox_publisher,
            z_subscriber,
        })
    }
}

impl<ServiceType: iceoryx2::service::Service> Channel for SubscriberChannel<ServiceType> {
    /// Propagate remote publish-subscribe payloads received on the service to local subscribers.
    fn propagate(&self) -> Result<(), PropagationError> {
        for z_sample in self.z_subscriber.drain() {
            let iox_message_type_details = self
                .iox_service_config
                .publish_subscribe()
                .message_type_details();
            let iox_payload_size = iox_message_type_details.payload.size();
            let _iox_payload_alignment = iox_message_type_details.payload.alignment();

            // TODO(correctness): verify size and alignment
            let z_payload = z_sample.payload();

            let number_of_elements = z_payload.len() / iox_payload_size;
            unsafe {
                match self.iox_publisher.loan_custom_payload(number_of_elements) {
                    Ok(mut iox_sample) => {
                        core::ptr::copy_nonoverlapping(
                            z_payload.to_bytes().as_ptr(),
                            iox_sample.payload_mut().as_mut_ptr() as *mut u8,
                            z_payload.len(),
                        );
                        let iox_sample = iox_sample.assume_init();
                        fail!(
                            from &self,
                            when iox_sample.send(),
                            with PropagationError::IceoryxPort,
                            "failed to publish remote payload to local subscribers"
                        );

                        info!(
                            "PROPAGATE SubscriberChannel {} [{}]",
                            self.iox_service_config.service_id().as_str(),
                            self.iox_service_config.name()
                        );
                    }
                    Err(e) => {
                        fatal_panic!("failed to loan custom payload: {e}");
                    }
                }
            }
        }

        Ok(())
    }
}
