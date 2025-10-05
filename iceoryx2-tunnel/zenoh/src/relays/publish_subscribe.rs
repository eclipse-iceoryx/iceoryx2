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

use iceoryx2::service::static_config::StaticConfig;
use iceoryx2_bb_log::{debug, error, fail};
use iceoryx2_tunnel_traits::{PublishSubscribeRelay, RelayBuilder};
use zenoh::{
    bytes::ZBytes,
    handlers::{FifoChannel, FifoChannelHandler},
    pubsub::{Publisher, Subscriber},
    qos::Reliability,
    sample::{Locality, Sample},
    Session, Wait,
};

use crate::keys;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum CreationError {
    FailedToCreateEndpoint,
}
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum PropagationError {
    FailedToPutPayload,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum IngestionError {
    Error,
}

impl From<Box<dyn std::error::Error + Send + Sync>> for CreationError {
    fn from(_: Box<dyn std::error::Error + Send + Sync>) -> Self {
        CreationError::FailedToCreateEndpoint
    }
}

#[derive(Debug)]
pub struct Builder<'a> {
    session: &'a Session,
    static_config: &'a StaticConfig,
}

impl<'a> Builder<'a> {
    pub fn new(session: &'a Session, static_config: &'a StaticConfig) -> Builder<'a> {
        Builder {
            session,
            static_config,
        }
    }
}

impl<'a> RelayBuilder for Builder<'a> {
    type CreationError = CreationError;
    type Relay = Relay;

    fn create(self) -> Result<Self::Relay, Self::CreationError> {
        let key = keys::publish_subscribe(self.static_config.service_id());

        let publisher = fail!(
            from "publish_subscribe::RelayBuilder::create()",
            when self.session
                .declare_publisher(key.clone())
                .allowed_destination(Locality::Remote)
                .reliability(Reliability::Reliable)
                .wait(),
            "Failed to create zenoh publisher for payloads"
        );

        // TODO(correctness): Make handler type and properties configurable
        let subscriber = fail!(
            from "publish_subscribe::RelayBuilder::create()",
            when self.session
                .declare_subscriber(key.clone())
                .with(FifoChannel::new(10))
                .allowed_origin(Locality::Remote)
                .wait(),
            "Failed to create zenoh subscriber for payloads"
        );

        fail!(
            from "publish_subscribe::RelayBuilder::create()",
            when announce_service(self.session, self.static_config),
            "Failed to announce service"
        );

        Ok(Relay {
            static_config: self.static_config.clone(),
            publisher,
            subscriber,
        })
    }
}

#[derive(Debug)]
pub struct Relay {
    static_config: StaticConfig,
    publisher: Publisher<'static>,
    subscriber: Subscriber<FifoChannelHandler<Sample>>,
}

impl iceoryx2_tunnel_traits::PublishSubscribeRelay for Relay {
    type PropagationError = PropagationError;
    type IngestionError = PropagationError;

    fn propagate(&self, bytes: *const u8, len: usize) -> Result<(), Self::PropagationError> {
        debug!(from "Relay::propagate", "Propagating PublishSubscribe({})", self.static_config.name());

        let payload = unsafe { ZBytes::from(core::slice::from_raw_parts(bytes, len)) };
        fail!(
            from "publish_subscribe::Relay::propagate",
            when self.publisher.put(payload).wait(),
            with PropagationError::FailedToPutPayload,
            "Failed to propagate payload over relay"
        );
        Ok(())
    }

    fn ingest(
        &self,
        loan: &mut dyn FnMut(usize) -> (*mut u8, usize),
    ) -> Result<bool, Self::IngestionError> {
        for zenoh_sample in self.subscriber.drain() {
            debug!(from "Relay::ingest", "Ingesting PublishSubscribe({})", self.static_config.name());

            let zenoh_payload = zenoh_sample.payload();
            let (loaned_ptr, loan_size) = loan(zenoh_payload.len());

            assert!(
                loan_size >= zenoh_payload.len(),
                "loan_size ({}) is too small for received payload ({})",
                loan_size,
                zenoh_payload.len()
            );

            unsafe {
                core::ptr::copy_nonoverlapping(
                    zenoh_payload.to_bytes().as_ptr(),
                    loaned_ptr,
                    zenoh_payload.len(),
                );
            }

            return Ok(true);
        }

        Ok(false)
    }
}

pub fn announce_service(
    session: &Session,
    static_config: &StaticConfig,
) -> Result<(), zenoh::Error> {
    let key = keys::service_details(static_config.service_id());
    let service_config_serialized = fail!(
        from "announce_service()",
        when serde_json::to_string(&static_config),
        "failed to serialize service config"
    );

    // Notify all current hosts.
    fail!(
        from "announce_service()",
        when session
            .put(key.clone(), service_config_serialized.clone())
            .allowed_destination(Locality::Remote)
            .wait(),
        "failed to share service details with remote hosts"
    );

    // Set up a queryable to respond to future hosts.
    fail!(
        from "announce_service()",
        when session
            .declare_queryable(key.clone())
            .callback(move |query| {
                let _ = query
                    .reply(key.clone(), service_config_serialized.clone())
                    .wait()
                    .inspect_err(|e| {
                        error!("Failed to announce service {}: {}", key, e);
                    });
            })
            .allowed_origin(Locality::Remote)
            .background()
            .wait(),
        "failed to set up queryable to share service details with remote hosts"
    );

    Ok(())
}
