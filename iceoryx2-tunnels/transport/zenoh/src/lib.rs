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

use iceoryx2::service::Service;
use iceoryx2_bb_log::fail;
use iceoryx2_tunnels_core::{Discovery, Relay, RelayBuilder, RelayFactory, Transport};
use zenoh::{Session, Wait};

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Error {
    Error,
}

pub struct PublishSubscribeRelay {}

impl Relay for PublishSubscribeRelay {
    fn propagate(&mut self, data: *const u8, len: usize, align: usize) {
        todo!()
    }

    fn ingest(&mut self, data: *mut u8, len: usize, align: usize) {
        todo!()
    }
}

pub struct PublishSubscribeRelayBuilder {}

impl<T: Transport> RelayBuilder<T> for PublishSubscribeRelayBuilder {
    type CreationError = Error;
    fn create(self) -> Result<Box<dyn Relay>, Error> {
        todo!()
    }
}

pub struct EventRelay {}

impl Relay for EventRelay {
    fn propagate(&mut self, data: *const u8, len: usize, align: usize) {
        todo!()
    }

    fn ingest(&mut self, data: *mut u8, len: usize, align: usize) {
        todo!()
    }
}

pub struct EventRelayBuilder {}

impl<T: Transport> RelayBuilder<T> for EventRelayBuilder {
    type CreationError = Error;
    fn create(self) -> Result<Box<dyn Relay>, Error> {
        todo!()
    }
}

pub struct Zenoh {
    session: Session,
}

impl Transport for Zenoh {
    type TransportConfig = zenoh::Config;
    type PublishSubscribeConfig = zenoh::Config;
    type EventConfig = zenoh::Config;

    type CreationError = Error;

    fn create(config: &Self::TransportConfig) -> Result<Self, Self::CreationError> {
        let session = zenoh::open(config.clone()).wait();
        let session = fail!(
            from "ZenohTransport::create()",
            when session,
            with Error::Error,
            "failed to create zenoh session"
        );

        Ok(Self { session })
    }
}

impl<T: Transport> RelayFactory<T> for Zenoh {
    fn publish_subscribe<'a>(
        &'a self,
        service: &'a str,
        config: &'a T::PublishSubscribeConfig,
    ) -> impl RelayBuilder<T> + 'a
    where
        Self: 'a,
    {
        PublishSubscribeRelayBuilder {}
    }

    fn event<'a>(
        &'a self,
        service: &'a str,
        config: &'a T::EventConfig,
    ) -> impl RelayBuilder<T> + 'a
    where
        Self: 'a,
    {
        EventRelayBuilder {}
    }
}

impl<S: Service> Discovery<S> for Zenoh {
    type DiscoveryError = Error;

    fn discover<
        OnDiscovered: FnMut(&iceoryx2::service::static_config::StaticConfig) -> Result<(), Self::DiscoveryError>,
    >(
        &mut self,
        on_discovered: &mut OnDiscovered,
    ) -> Result<(), Self::DiscoveryError> {
        todo!()
    }
}
