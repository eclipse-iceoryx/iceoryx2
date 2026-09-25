// Copyright (c) 2026 Contributors to the Eclipse Foundation
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

use alloc::vec::Vec;
use core::fmt::Debug;
use core::time::Duration;

use iceoryx2::config::Config;
use iceoryx2::testing::generate_isolated_config;
use iceoryx2_link_adapter::{Adapter, EndpointDescription, EndpointTypes};

/// One middleware, the adapters on it, and the remote endpoints on it.
pub trait AdapterFixture {
    /// The middleware's types of a sample.
    type SampleTypes: Clone + PartialEq + 'static;
    type Adapter: Adapter<EndpointTypes = EndpointTypes<Self::SampleTypes>>;
    /// Remote endpoints of the middleware, speaking its messages, bytes
    /// in its form.
    type RemoteEndpoints: MessageEndpoints<
            <Self::Adapter as Adapter>::EndpointSettings,
            EndpointTypes<Self::SampleTypes>,
        >;

    /// A fresh, isolated middleware.
    fn new() -> Self;

    /// The configuration to use in the fixture.
    fn config(&self) -> Config {
        generate_isolated_config()
    }

    /// An adapter on the middleware.
    fn adapter(&mut self) -> Self::Adapter;

    /// Remote endpoints under a new description. Gone once dropped.
    fn remote_endpoints(&mut self) -> Self::RemoteEndpoints;
}

/// Remote endpoints speaking the middleware's messages as values, and
/// encoding them as the bytes the gateway's endpoints handle.
pub trait MessageEndpoints<S, T> {
    /// The value of a message.
    type Value: Clone + PartialEq + Debug;

    /// The description of the endpoints.
    fn description(&self) -> &EndpointDescription<S, T>;

    /// The value the suites send for `n`. Different `n` give different
    /// values.
    fn value(&self, n: u64) -> Self::Value;

    /// The bytes of a message carrying `value`.
    fn encode(&self, value: &Self::Value) -> Vec<u8>;

    /// The value the message `bytes` carries.
    fn decode(&self, bytes: &[u8]) -> Self::Value;

    /// Sends a message carrying `value`.
    fn send(&self, value: Self::Value);

    /// The value of the next message pending, if any.
    fn receive(&self) -> Option<Self::Value>;

    /// Waits until the gateway's endpoints opened on the description are
    /// connected to these, or `timeout` passes. Immediate on a middleware
    /// that delivers without matching first.
    fn sync(&self, _timeout: Duration) -> bool {
        true
    }
}
