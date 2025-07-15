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

use iceoryx2::node::NodeId as IceoryxNodeId;
use iceoryx2::port::subscriber::Subscriber as IceoryxSubscriber;
use iceoryx2::service::builder::CustomHeaderMarker;
use iceoryx2::service::builder::CustomPayloadMarker;
use iceoryx2::service::port_factory::publish_subscribe::PortFactory as IceoryxPublishSubscribeService;
use iceoryx2::service::static_config::StaticConfig as IceoryxServiceConfig;
use iceoryx2_bb_log::fail;
use iceoryx2_bb_log::fatal_panic;
use iceoryx2_bb_log::info;

use zenoh::bytes::ZBytes;
use zenoh::pubsub::Publisher as ZenohPublisher;
use zenoh::Session as ZenohSession;
use zenoh::Wait;

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

/// A channel for propagating `iceoryx2` publish-subscribe payloads to remote hosts.
#[derive(Debug)]
pub(crate) struct PublisherChannel<'a, ServiceType: iceoryx2::service::Service> {
    iox_node_id: IceoryxNodeId,
    iox_service_config: IceoryxServiceConfig,
    iox_subscriber: IceoryxSubscriber<ServiceType, [CustomPayloadMarker], CustomHeaderMarker>,
    z_publisher: ZenohPublisher<'a>,
}

impl<ServiceType: iceoryx2::service::Service> PublisherChannel<'_, ServiceType> {
    // Creates an outbound channel to remote hosts for publish-subscribe payloads for a
    // particular service.
    pub fn create(
        iox_node_id: &IceoryxNodeId,
        iox_service_config: &IceoryxServiceConfig,
        iox_service: &IceoryxPublishSubscribeService<
            ServiceType,
            [CustomPayloadMarker],
            CustomHeaderMarker,
        >,
        z_session: &ZenohSession,
    ) -> Result<Self, CreationError> {
        info!(
            "CREATE PublisherChannel  {} [{}]",
            iox_service_config.service_id().as_str(),
            iox_service_config.name()
        );

        let iox_subscriber = fail!(
            from "PublisherChannel::create()",
            when middleware::iceoryx::create_subscriber::<ServiceType>(iox_service),
            with CreationError::Error,
            "failed to create iceoryx subscriber to propagate local payloads"
        );
        let z_publisher = fail!(
            from "PublisherChannel::create()",
            when middleware::zenoh::create_publisher(z_session, iox_service_config),
            with CreationError::Error,
            "failed to create zenoh publisher for local payloads"
        );

        Ok(Self {
            iox_node_id: *iox_node_id,
            iox_service_config: iox_service_config.clone(),
            iox_subscriber,
            z_publisher,
        })
    }
}

impl<ServiceType: iceoryx2::service::Service> Channel for PublisherChannel<'_, ServiceType> {
    /// Propagate local publish-subscribe payloads to remote hosts.
    fn propagate(&self) -> Result<(), PropagationError> {
        loop {
            match unsafe { self.iox_subscriber.receive_custom_payload() } {
                Ok(Some(sample)) => {
                    if sample.header().node_id() == self.iox_node_id {
                        // Ignore samples published by the gateway itself to prevent loopback.
                        continue;
                    }

                    let ptr = sample.payload().as_ptr() as *const u8;
                    let len = sample.len();
                    let bytes = unsafe { core::slice::from_raw_parts(ptr, len) };

                    // TODO(optimization): Is it possible to create the ZBytes struct without copy?
                    let z_payload = ZBytes::from(bytes);
                    fail!(
                        from &self,
                        when self.z_publisher.put(z_payload).wait(),
                        with PropagationError::OtherPort,
                        "failed to propagate local payload to remote hosts"
                    );

                    info!(
                        "PROPAGATE PublisherChannel {} [{}]",
                        self.iox_service_config.service_id().as_str(),
                        self.iox_service_config.name()
                    );
                }
                Ok(None) => break, // No more samples available
                Err(e) => {
                    fatal_panic!("failed to receive custom payload: {e}");
                }
            }
        }

        Ok(())
    }
}
