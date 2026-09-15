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
use core::marker::PhantomData;

use iceoryx2_link_adapter::EndpointDescription;
use iceoryx2_link_backend::service_description::{ServiceDescription, ServiceTypes};
use iceoryx2_link_conformance_tests::fixture::{
    DiscoverableEndpoints, MessageEndpoints, PayloadEndpoints, SampleEndpoints,
};
use iceoryx2_link_testing::{
    FakeEndpointDescription, FakeEndpointSettings, FakeEndpointTypes, FakeEndpoints,
    FakeMiddleware, endpoint_of,
};

use super::wire_form::{Header, Payload, WireForm};

/// The endpoint `service` maps to, its types as `T` has the middleware
/// name them.
pub(crate) fn endpoint_for<T: WireForm>(service: &ServiceDescription) -> FakeEndpointDescription {
    EndpointDescription {
        types: T::default()
            .remote(service.types())
            .expect("the fake translators never fail"),
        ..endpoint_of(service)
    }
}

/// Remote endpoints on the endpoint `service` maps to on `middleware`,
/// speaking payloads as `T` has the middleware encode them.
pub(crate) fn remote_endpoints_on<T: WireForm>(
    middleware: &FakeMiddleware,
    service: ServiceDescription,
) -> RemotePayloadEndpoints<T> {
    let remote = middleware.remote_endpoints(&endpoint_for::<T>(&service));
    let header = match service.types() {
        ServiceTypes::PublishSubscribe(types) => types.user_header.size,
        ServiceTypes::Event => 0,
    };
    RemotePayloadEndpoints {
        service,
        remote,
        header,
        _translator: PhantomData,
    }
}

/// Remote endpoints on a fake middleware, on one endpoint.
pub(crate) struct RemoteMessageEndpoints {
    pub(crate) description: FakeEndpointDescription,
    pub(crate) remote: FakeEndpoints,
}

impl MessageEndpoints<FakeEndpointSettings, FakeEndpointTypes> for RemoteMessageEndpoints {
    fn description(&self) -> &FakeEndpointDescription {
        &self.description
    }

    fn send_message(&self, message: &[u8]) {
        self.remote.send(message);
    }

    fn receive_message(&self) -> Option<Vec<u8>> {
        self.remote.receive()
    }
}

/// Remote endpoints on a fake middleware, on the endpoint of
/// one service, speaking payloads as `T` has the middleware encode them.
pub(crate) struct RemotePayloadEndpoints<T> {
    pub(crate) service: ServiceDescription,
    pub(crate) remote: FakeEndpoints,
    /// The size of the user header leading each message, sent as zero.
    header: usize,
    _translator: PhantomData<T>,
}

impl<W: WireForm> DiscoverableEndpoints for RemotePayloadEndpoints<W> {
    fn service(&self) -> &ServiceDescription {
        &self.service
    }
}

impl<W: WireForm> PayloadEndpoints<Payload> for RemotePayloadEndpoints<W> {
    fn send_payload(&self, payload: Payload) {
        let mut message = alloc::vec![0; self.header];
        message.extend_from_slice(&W::encode_payload(payload));
        self.remote.send(&message);
    }

    fn receive_payload(&self) -> Option<Payload> {
        let message = self.remote.receive()?;
        let (_, payload) = message.split_at(self.header);
        Some(W::decode_payload(payload.try_into().expect("a payload")))
    }
}

impl<W: WireForm> SampleEndpoints<Header, Payload> for RemotePayloadEndpoints<W> {
    fn send_sample(&self, header: Header, payload: Payload) {
        self.remote
            .send(&[W::encode_header(header), W::encode_payload(payload)].concat());
    }

    fn receive_sample(&self) -> Option<(Header, Payload)> {
        let message = self.remote.receive()?;
        let (header, payload) = message.split_at(self.header);
        Some((
            W::decode_header(header.try_into().expect("a header")),
            W::decode_payload(payload.try_into().expect("a payload")),
        ))
    }
}
