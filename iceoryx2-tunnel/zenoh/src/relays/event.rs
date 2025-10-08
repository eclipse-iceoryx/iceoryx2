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

use iceoryx2::prelude::EventId;
use iceoryx2::service::static_config::StaticConfig;
use iceoryx2::service::Service;
use iceoryx2_bb_log::debug;
use iceoryx2_bb_log::fail;
use iceoryx2_tunnel_backend::traits::{EventRelay, RelayBuilder};
use zenoh::handlers::FifoChannel;
use zenoh::handlers::FifoChannelHandler;
use zenoh::pubsub::Publisher;
use zenoh::pubsub::Subscriber;
use zenoh::qos::Reliability;
use zenoh::sample::Locality;
use zenoh::sample::Sample;
use zenoh::Session;
use zenoh::Wait;

use crate::keys;
use crate::relays::announce_service;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum CreationError {
    PublisherDeclaration,
    SubscriberDeclaration,
    ServiceAnouncement,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum SendError {
    EventPut,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum ReceiveError {
    EventReceive,
    EventIngestion,
    InvalidEvent,
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
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<'a, S: Service> RelayBuilder for Builder<'a, S> {
    type CreationError = CreationError;
    type Relay = Relay<S>;

    fn create(self) -> Result<Self::Relay, Self::CreationError> {
        debug!(
            from "event::RelayBuilder::create",
            "{}",
            format!("Creating event relay for service {}", self.static_config.name())
        );

        let key = keys::event(self.static_config.service_id());

        let notifier = fail!(
            from "event::RelayBuilder::create",
            when self.session
                .declare_publisher(key.clone())
                .allowed_destination(Locality::Remote)
                .reliability(Reliability::Reliable)
                .wait(),
            with CreationError::PublisherDeclaration,
            "Failed to create zenoh publisher for notifications"
        );

        // TODO(correctness): Make handler type and properties configurable
        let listener = fail!(
        from "event::RelayBuilder::create",
        when self.session
            .declare_subscriber(key.clone())
            .with(FifoChannel::new(10))
            .allowed_origin(Locality::Remote)
            .wait(),
        with CreationError::SubscriberDeclaration,
        "Failed to create zenoh subscriber for notifications");

        fail!(
            from "event::RelayBuilder::create",
            when announce_service(self.session, self.static_config),
            with CreationError::ServiceAnouncement,
            "Failed to annnounce service on Zenoh"
        );

        Ok(Relay {
            notifier,
            listener,
            _phantom: core::marker::PhantomData,
        })
    }
}

#[derive(Debug)]
pub struct Relay<S: Service> {
    notifier: Publisher<'static>,
    listener: Subscriber<FifoChannelHandler<Sample>>,
    _phantom: core::marker::PhantomData<S>,
}

impl<S: Service> EventRelay<S> for Relay<S> {
    type SendError = SendError;
    type ReceiveError = ReceiveError;

    fn send(&self, event_id: EventId) -> Result<(), Self::SendError> {
        fail!(
            from "event::Relay::propagate",
            when self.notifier.put(event_id.as_value().to_ne_bytes()).wait(),
            with SendError::EventPut,
            "Failed to propagate notification to zenoh"
        );

        Ok(())
    }

    fn receive(&self) -> Result<Option<EventId>, Self::ReceiveError> {
        let sample = fail!(
            from "event::Relay::receive",
            when self.listener.try_recv(),
            with ReceiveError::EventReceive,
            "Failed to receive event from zenoh"
        );

        match sample {
            Some(sample) => {
                let payload = sample.payload();
                if payload.len() == std::mem::size_of::<usize>() {
                    let id: usize =
                        unsafe { payload.to_bytes().as_ptr().cast::<usize>().read_unaligned() };

                    Ok(Some(EventId::new(id)))
                } else {
                    Err(ReceiveError::InvalidEvent)
                }
            }
            None => Ok(None),
        }
    }
}
