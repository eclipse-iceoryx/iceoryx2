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
use core::time::Duration;

use iceoryx2_link_adapter::{Adapter, EndpointDescription};

/// One middleware, the adapters on it, and the remote endpoints on it.
pub trait AdapterFixture {
    type Adapter: Adapter;
    /// Remote endpoints of the middleware, speaking its messages, bytes
    /// in its form.
    type RemoteEndpoints: MessageEndpoints<
            <Self::Adapter as Adapter>::EndpointSettings,
            <Self::Adapter as Adapter>::EndpointTypes,
        >;

    /// A fresh, isolated middleware.
    fn new() -> Self;

    /// An adapter on the middleware.
    fn adapter(&mut self) -> Self::Adapter;

    /// Remote endpoints under a new description. Gone once dropped.
    fn remote_endpoints(&mut self) -> Self::RemoteEndpoints;
}

/// Remote endpoints speaking the middleware's messages, bytes in its
/// form.
pub trait MessageEndpoints<S, T> {
    /// The description of the endpoints.
    fn description(&self) -> &EndpointDescription<S, T>;

    /// Sends a message.
    fn send_message(&self, message: &[u8]);

    /// The next message pending, if any.
    fn receive_message(&self) -> Option<Vec<u8>>;

    /// Waits until the gateway's endpoints opened on the description are
    /// connected to these, or `timeout` passes. Immediate on a middleware
    /// that delivers without matching first.
    fn sync(&self, _timeout: Duration) -> bool {
        true
    }
}
