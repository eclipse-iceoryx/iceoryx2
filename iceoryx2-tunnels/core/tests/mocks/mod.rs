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

use iceoryx2_tunnels_core::{Relay, RelayBuilder, RelayFactory, Transport};

#[derive(Debug, Default)]
pub struct MockTransportConfig {}
pub struct MockTransport {}
pub struct MockRelay {}

#[derive(Debug)]
pub struct MockRelayBuilder {}

impl Transport for MockTransport {
    type Config = MockTransportConfig;
    type Error = ();

    fn create(_config: &Self::Config) -> Result<Self, Self::Error> {
        Ok(MockTransport {})
    }
}

impl Relay for MockRelay {
    fn propagate(&self, _bytes: *const u8, _len: usize) {}

    fn ingest(&self, _loan_fn: &mut dyn FnMut(usize) -> (*mut u8, usize)) -> bool {
        true
    }
}

impl RelayBuilder for MockRelayBuilder {
    type Error = ();

    fn create(self) -> Result<Box<dyn Relay>, Self::Error> {
        Ok(Box::from(MockRelay {}))
    }
}

impl<T: Transport> RelayFactory<T> for MockTransport {
    type PublishSubscribeBuilder = MockRelayBuilder;
    type EventBuilder = MockRelayBuilder;

    fn publish_subscribe(&self, _service: &str) -> Self::PublishSubscribeBuilder {
        MockRelayBuilder {}
    }

    fn event(&self, _service: &str) -> Self::EventBuilder {
        MockRelayBuilder {}
    }
}
