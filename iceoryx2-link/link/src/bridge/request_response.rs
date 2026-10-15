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

use iceoryx2::node::Node;
use iceoryx2::service::Service;
use iceoryx2_link_backend::relay::{RelayBuilder, RelayFactory, RequestResponseRelay};
use iceoryx2_link_backend::service_description::RequestResponseDescription;
use iceoryx2_link_backend::wire::request_response::{OutboundRequest, OutboundResponse};
use iceoryx2_link_backend::{Backend, RemoteDescription, RemoteId};
use iceoryx2_log::{fail, origin};

use crate::bridge::{Bridge, BridgeError, Monitoring, OpenError};
use crate::ports::RequestResponsePorts;

/// A request-response service's local ports paired with the backend's
/// relay for them.
pub(super) struct RequestResponseBridge<S: Service, B: Backend<S>> {
    ports: RequestResponsePorts<S, RemoteId<S, B>>,
    relay: B::RequestResponseRelay,
    monitoring: Monitoring,
}

impl<S: Service, B: Backend<S>> Bridge for RequestResponseBridge<S, B> {
    type Service = S;
    type Backend = B;
    type LocalDescription<'a> = RequestResponseDescription<'a>;

    fn open(
        node: &Node<S>,
        backend: &mut B,
        local_description: RequestResponseDescription<'_>,
        remote_description: &RemoteDescription<S, B>,
    ) -> Result<Self, OpenError> {
        let origin = origin!("RequestResponseBridge::open");

        let ports = fail!(
            from origin,
            when RequestResponsePorts::open(node, &local_description.name(), local_description.settings(), local_description.types()),
            with OpenError::Ports,
            "Failed to open the local ports of {}", local_description.name()
        );

        let mut factory = backend.relay_factory();
        let relay = fail!(
            from origin,
            when factory.request_response(local_description, remote_description).create(),
            with OpenError::Relay,
            "Failed to create the relay of {}", local_description.name()
        );

        Ok(Self {
            ports,
            relay,
            monitoring: Monitoring::default(),
        })
    }

    fn update_remotes<'a>(&mut self, remotes: impl Iterator<Item = &'a RemoteId<S, B>>)
    where
        RemoteId<S, B>: 'a,
    {
        self.ports.update_remotes(remotes.cloned().collect());
    }

    fn propagate(&mut self, _: &Node<S>, _: bool) -> Result<(), BridgeError> {
        let origin = origin!("RequestResponseBridge::propagate");

        let propagated = fail!(
            from origin,
            when self.ports.receive_requests(|id, outbound| self.relay.send_request(id, outbound)),
            with BridgeError::Propagation,
            "Failed to propagate requests"
        );
        self.monitoring.outbound += propagated;

        let ingested = fail!(
            from origin,
            when self.ports.send_requests(|loanable| self.relay.receive_request(loanable)),
            with BridgeError::Ingestion,
            "Failed to ingest requests from the opposing side"
        );
        self.monitoring.inbound += ingested;

        let propagated = fail!(
            from origin,
            when self.ports.receive_responses(|id, outbound| self.relay.send_response(id, outbound)),
            with BridgeError::Propagation,
            "Failed to propagate responses"
        );
        self.monitoring.outbound += propagated;

        let ingested = fail!(
            from origin,
            when self.ports.send_responses(|ports| self.relay.receive_response(|id| ports.response_loan(id))),
            with BridgeError::Ingestion,
            "Failed to ingest responses from the opposing side"
        );
        self.monitoring.inbound += ingested;

        Ok(())
    }

    fn monitoring(&mut self) -> &mut Monitoring {
        &mut self.monitoring
    }
}

/// Closing the bridge abandons every local request and ends the stream of
/// every remote request through the relay.
impl<S: Service, B: Backend<S>> Drop for RequestResponseBridge<S, B> {
    fn drop(&mut self) {
        self.ports
            .drain_local_requests(|id| self.relay.send_request(id, OutboundRequest::Abandoned));
        self.ports
            .drain_remote_requests(|id| self.relay.send_response(id, OutboundResponse::End));
    }
}
