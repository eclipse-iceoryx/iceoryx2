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

use iceoryx2_tunnel_traits::{Discovery, Relay, RelayBuilder, RelayFactory, Transport};

#[derive(Debug, Default)]
pub struct MockRelay {}

#[derive(Debug)]
pub struct MockRelayBuilder {}

impl Relay for MockRelay {
    fn propagate(&self, _bytes: *const u8, _len: usize) {}

    fn ingest(&self, _loan_fn: &mut dyn FnMut(usize) -> (*mut u8, usize)) -> bool {
        true
    }
}

impl RelayBuilder for MockRelayBuilder {
    type CreationError = ();

    fn create(self) -> Result<Box<dyn Relay>, Self::CreationError> {
        Ok(Box::from(MockRelay {}))
    }
}

pub struct MockRelayFactory {}

impl RelayFactory for MockRelayFactory {
    type PublishSubscribeBuilder = MockRelayBuilder;
    type EventBuilder = MockRelayBuilder;

    fn publish_subscribe(&self, _service: &str) -> Self::PublishSubscribeBuilder {
        MockRelayBuilder {}
    }

    fn event(&self, _service: &str) -> Self::EventBuilder {
        MockRelayBuilder {}
    }
}

pub struct MockTransport {
    discovery: MockDiscovery,
}

#[derive(Debug, Default)]
pub struct MockTransportConfig {}

impl Transport for MockTransport {
    type Config = MockTransportConfig;
    type CreationError = ();
    type RelayFactory = MockRelayFactory;
    type Discovery = MockDiscovery;

    fn create(_config: &Self::Config) -> Result<Self, Self::CreationError> {
        Ok(MockTransport {
            discovery: MockDiscovery {},
        })
    }

    fn relay_builder(&self) -> Self::RelayFactory {
        MockRelayFactory {}
    }

    fn discovery(&mut self) -> &mut impl iceoryx2_tunnel_traits::Discovery {
        &mut self.discovery
    }
}

pub struct MockDiscovery {}

impl Discovery for MockDiscovery {
    type DiscoveryError = ();

    fn discover<
        F: FnMut(&iceoryx2::service::static_config::StaticConfig) -> Result<(), Self::DiscoveryError>,
    >(
        &mut self,
        process_discovery: &mut F,
    ) -> Result<(), Self::DiscoveryError> {
        todo!()
    }
}
