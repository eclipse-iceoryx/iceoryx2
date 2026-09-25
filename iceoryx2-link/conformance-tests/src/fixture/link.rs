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

use iceoryx2::config::Config;
use iceoryx2::service::Service;
use iceoryx2_link_backend::Backend;
use iceoryx2_link_gateway::Gateway;
use iceoryx2_link_tunnel::Tunnel;

use crate::fixture::{GatewayFixture, TunnelFixture};

/// What the link suites build their backends from. Sealed, implemented
/// by `TunnelLinkFixture` and `GatewayLinkFixture` alone.
pub trait LinkFixture<S: Service>: sealed::Sealed {
    type Backend: Backend<S>;

    /// A fresh, isolated mechanism.
    fn new() -> Self;

    /// The configuration to use in the fixture.
    fn config(&self) -> Config;

    /// A backend on the mechanism, for the local system configured by
    /// `config`.
    fn backend(&mut self, config: &Config) -> Self::Backend;
}

/// A link fixture over a tunnel fixture, its tunnels the link's backends.
pub struct TunnelLinkFixture<F>(F);

impl<S: Service, F: TunnelFixture> LinkFixture<S> for TunnelLinkFixture<F> {
    type Backend = Tunnel<F::Carrier>;

    fn new() -> Self {
        Self(F::new())
    }

    fn config(&self) -> Config {
        self.0.config()
    }

    fn backend(&mut self, config: &Config) -> Self::Backend {
        self.0.tunnel(config)
    }
}

/// A link fixture over a gateway fixture, its gateways the link's backends.
pub struct GatewayLinkFixture<F>(F);

impl<S: Service, F: GatewayFixture<S>> LinkFixture<S> for GatewayLinkFixture<F> {
    type Backend = Gateway<F::Adapter, F::Mapping, F::Translator>;

    fn new() -> Self {
        Self(F::new())
    }

    fn config(&self) -> Config {
        self.0.config()
    }

    fn backend(&mut self, _: &Config) -> Self::Backend {
        self.0.gateway()
    }
}

mod sealed {
    pub trait Sealed {}
    impl<F> Sealed for super::TunnelLinkFixture<F> {}
    impl<F> Sealed for super::GatewayLinkFixture<F> {}
}
