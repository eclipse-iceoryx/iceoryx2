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

use iceoryx2_tunnel_traits::{Relay, RelayBuilder, RelayFactory, Transport};

#[derive(Debug, Default)]
pub struct MockTransportConfig {}
pub struct MockTransport {}
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

impl Transport for MockTransport {
    type Config = MockTransportConfig;
    type CreationError = ();
    type RelayFactory = MockRelayFactory;

    fn create(_config: &Self::Config) -> Result<Self, Self::CreationError> {
        Ok(MockTransport {})
    }

    fn relay_builder(&self) -> Self::RelayFactory {
        MockRelayFactory {}
    }
}
