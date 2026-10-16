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

use core::time::Duration;

use iceoryx2::service::Service;
use iceoryx2_link_adapter::Adapter;
use iceoryx2_link_adapter::Mapping;
use iceoryx2_link_adapter::{SampleShape, TranscodesSamples, Translator};
use iceoryx2_link_backend::service_description::ServiceDescription;
use iceoryx2_link_gateway::Gateway;

use crate::fixture::AdapterFixture;

/// One middleware, the gateways on it, each over an adapter of its own
/// with the middleware's mapping and translator, and the remote endpoints
/// its services map to.
pub trait GatewayFixture<S: Service>: AdapterFixture {
    type Mapping: Mapping<EndpointSettings = <Self::Adapter as Adapter>::EndpointSettings>;
    type Translator: Translator<SampleShape, RemoteTypes = Self::SampleTypes, Transcoders: TranscodesSamples>;
    /// Remote endpoints where a service maps to. A suite bounds them by
    /// what it needs of them.
    type RemoteEndpoints;

    /// The mapping between services and the middleware's endpoints.
    fn mapping(&self) -> Self::Mapping;

    /// The translator between local types and the middleware's.
    fn translator(&self) -> Self::Translator;

    /// A gateway on the middleware, over a new adapter with the mapping
    /// and translator.
    fn gateway(&mut self) -> Gateway<Self::Adapter, Self::Mapping, Self::Translator> {
        Gateway::new(self.adapter(), self.mapping(), self.translator())
    }

    /// Remote endpoints where `service` maps to, one the mapping covers.
    /// Gone once dropped.
    fn remote_endpoints_on(
        &mut self,
        service: &ServiceDescription,
    ) -> <Self as GatewayFixture<S>>::RemoteEndpoints;
}

/// Remote endpoints the link can discover, where a service maps to.
pub trait Discoverable {
    /// The service the endpoints map to.
    fn service(&self) -> &ServiceDescription;

    /// Waits until the gateway's endpoints where the service maps to are
    /// connected to these, or `timeout` passes. Immediate on a middleware
    /// that delivers without matching first.
    fn sync(&self, _timeout: Duration) -> bool {
        true
    }
}

/// Remote endpoints that propagate publish-subscribe payloads of type `P`.
pub trait PropagatesPayloads<P>: Discoverable {
    /// Sends a payload.
    fn send_payload(&self, payload: P);

    /// The next payload pending, if any.
    fn receive_payload(&self) -> Option<P>;
}

/// Remote endpoints that propagate publish-subscribe payloads and user headers
/// of types `H` and `P`.
pub trait PropagatesPayloadsWithHeaders<H, P>: PropagatesPayloads<P> {
    /// Sends a payload with its header.
    fn send_with_header(&self, header: H, payload: P);

    /// The next payload pending, if any, with its header.
    fn receive_with_header(&self) -> Option<(H, P)>;
}
