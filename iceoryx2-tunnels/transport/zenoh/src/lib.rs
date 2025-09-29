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

#[derive(Debug)]
pub struct PublishSubscribeRelayBuilder {}

impl RelayBuilder for PublishSubscribeRelayBuilder {
    type Error = Error;

    fn create(self) -> Result<Box<dyn Relay>, Self::Error> {
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

#[derive(Debug)]
pub struct EventRelayBuilder {}

impl RelayBuilder for EventRelayBuilder {
    type Error = Error;

    fn create(self) -> Result<Box<dyn Relay>, Self::Error> {
        todo!()
    }
}

pub struct Zenoh {
    session: Session,
}

impl Transport for Zenoh {
    type Config = zenoh::Config;
    type Error = Error;

    fn create(config: &Self::Config) -> Result<Self, Self::Error> {
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
    type PublishSubscribeBuilder = PublishSubscribeRelayBuilder;
    type EventBuilder = EventRelayBuilder;

    fn publish_subscribe(&self, service: &str) -> Self::PublishSubscribeBuilder {
        todo!()
    }

    fn event(&self, service: &str) -> Self::EventBuilder {
        todo!()
    }
}

impl<S: Service> Discovery<S> for Zenoh {
    type Handle = zenoh::Session;
    type Error = Error;

    fn discover<
        F: FnMut(&iceoryx2::service::static_config::StaticConfig) -> Result<(), Self::Error>,
    >(
        handle: &mut Self::Handle,
        process_discovery: &mut F,
    ) -> Result<(), Self::Error> {
        todo!()
    }
}
