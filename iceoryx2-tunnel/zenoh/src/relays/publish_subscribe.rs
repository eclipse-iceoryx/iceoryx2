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

use iceoryx2::service::builder::CustomHeaderMarker;
use iceoryx2::service::builder::CustomPayloadMarker;
use iceoryx2::service::{static_config::StaticConfig, Service};
use iceoryx2_bb_log::{debug, fail};
use iceoryx2_tunnel_backend::traits::{PublishSubscribeRelay, RelayBuilder};
use iceoryx2_tunnel_backend::types::publish_subscribe::LoanFn;
use iceoryx2_tunnel_backend::types::publish_subscribe::SampleMut;
use zenoh::{
    bytes::ZBytes,
    handlers::{FifoChannel, FifoChannelHandler},
    pubsub::{Publisher, Subscriber},
    qos::Reliability,
    sample::{Locality, Sample},
    Session, Wait,
};

use crate::keys;
use crate::relays::announce_service;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum CreationError {
    FailedToDeclarePublisher,
    FailedToDeclareSubscriber,
    FailedToAnnounceService,
}
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum SendError {
    FailedToPutPayload,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum ReceiveError {
    FailedToLoanSample,
}

#[derive(Debug)]
pub struct Builder<'a, S: Service> {
    session: &'a Session,
    static_config: &'a StaticConfig,
    _phantom: core::marker::PhantomData<S>,
}

impl<'a, S: Service> Builder<'a, S> {
    pub fn new(session: &'a Session, static_config: &'a StaticConfig) -> Builder<'a, S> {
        Builder {
            session,
            static_config,
            _phantom: core::marker::PhantomData::default(),
        }
    }
}

impl<'a, S: Service> RelayBuilder for Builder<'a, S> {
    type CreationError = CreationError;
    type Relay = Relay<S>;

    fn create(self) -> Result<Self::Relay, Self::CreationError> {
        debug!(
            from "publish_subscribe::RelayBuilder::create",
            "{}",
            format!("Creating publish-subscribe relay for service {}", self.static_config.name())
        );

        let key = keys::publish_subscribe(self.static_config.service_id());

        let publisher = fail!(
            from "publish_subscribe::RelayBuilder::create",
            when self.session
                .declare_publisher(key.clone())
                .allowed_destination(Locality::Remote)
                .reliability(Reliability::Reliable)
                .wait(),
            with CreationError::FailedToDeclarePublisher,
            "Failed to create zenoh publisher for publish-subscribe payloads"
        );

        // TODO(correctness): Make handler type and properties configurable
        let subscriber = fail!(
            from "publish_subscribe::RelayBuilder::create",
            when self.session
                .declare_subscriber(key.clone())
                .with(FifoChannel::new(10))
                .allowed_origin(Locality::Remote)
                .wait(),
            with CreationError::FailedToDeclareSubscriber,
            "Failed to create zenoh subscriber for publish-subscribe payloads"
        );

        fail!(
            from "publish_subscribe::RelayBuilder::create",
            when announce_service(self.session, self.static_config),
            with CreationError::FailedToAnnounceService,
            "Failed to annnounce service on Zenoh"
        );

        Ok(Relay {
            static_config: self.static_config.clone(),
            publisher,
            subscriber,
            _phantom: core::marker::PhantomData::default(),
        })
    }
}

#[derive(Debug)]
pub struct Relay<S: Service> {
    static_config: StaticConfig,
    publisher: Publisher<'static>,
    subscriber: Subscriber<FifoChannelHandler<Sample>>,
    _phantom: core::marker::PhantomData<S>,
}

impl<S: Service> PublishSubscribeRelay<S> for Relay<S> {
    type SendError = SendError;
    type ReceiveError = ReceiveError;

    fn send(
        &self,
        sample: iceoryx2::sample::Sample<S, [CustomPayloadMarker], CustomHeaderMarker>,
    ) -> Result<(), Self::SendError> {
        debug!(
            from "publish_subscribe::Relay::propagate",
            "Propagating publish-subscribe payload from service '{}'",
            self.static_config.name()
        );

        let payload = sample.payload();
        let bytes = payload.as_ptr() as *mut u8;
        let len = payload.len();

        let payload = unsafe { ZBytes::from(core::slice::from_raw_parts(bytes, len)) };
        fail!(
            from "publish_subscribe::Relay::propagate",
            when self.publisher.put(payload).wait(),
            with SendError::FailedToPutPayload,
            "Failed to propagate propagate publish-subscribe payload to zenoh"
        );

        Ok(())
    }

    fn receive<LoanError>(
        &self,
        loan: &mut LoanFn<'_, S, LoanError>,
    ) -> Result<Option<SampleMut<S>>, Self::ReceiveError> {
        for zenoh_sample in self.subscriber.drain() {
            debug!(
                from "publish_subscribe::Relay::ingest",
                "Ingesting publish-subscribe payload from service '{}'",
                self.static_config.name()
            );

            let zenoh_payload = zenoh_sample.payload();
            let mut iceoryx_sample = fail!(
                from "publish_subscribe::Relay::ingest",
                when loan(zenoh_payload.len()),
                with ReceiveError::FailedToLoanSample,
                "Failed to loan sample from iceoryx"
            );
            let iceoryx_payload = iceoryx_sample.payload_mut();

            assert!(
                iceoryx_payload.len() >= zenoh_payload.len(),
                "loan_size ({}) is too small for received payload ({})",
                iceoryx_payload.len(),
                zenoh_payload.len()
            );

            // TODO: Is there a safe iceoryx2 API to copy these bytes?
            unsafe {
                core::ptr::copy_nonoverlapping(
                    zenoh_payload.to_bytes().as_ptr(),
                    iceoryx_payload.as_mut_ptr().cast(),
                    zenoh_payload.len(),
                );
            }
            let initialized_sample = unsafe { iceoryx_sample.assume_init() };

            return Ok(Some(initialized_sample));
        }

        Ok(None)
    }
}
