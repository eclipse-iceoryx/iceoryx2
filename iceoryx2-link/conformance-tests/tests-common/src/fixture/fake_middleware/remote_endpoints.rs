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

use iceoryx2_link_adapter::{EndpointDescription, EndpointTypes};
use iceoryx2_link_backend::service_description::{ServiceDescription, ServiceTypes};
use iceoryx2_link_conformance_tests::fixture::{
    Discoverable, MessageEndpoints, PropagatesPayloads, PropagatesPayloadsWithHeaders,
};
use iceoryx2_link_testing::{
    FakeEndpointDescription, FakeEndpointSettings, FakeEndpoints, FakeMiddleware, FakeSampleTypes,
    endpoint_of,
};

use super::wire_form::{Header, Payload, WireForm};

/// The endpoint `service` maps to.
pub(crate) fn endpoint_for<T: WireForm>(service: &ServiceDescription) -> FakeEndpointDescription {
    let types = match service.types() {
        ServiceTypes::PublishSubscribe(types) => EndpointTypes::PublishSubscribe(
            T::default()
                .remote(types)
                .expect("the fake translators never fail"),
        ),
        ServiceTypes::Event => EndpointTypes::Event,
        ServiceTypes::RequestResponse(_) => {
            unimplemented!("the fake middleware has no request-response endpoints")
        }
    };
    EndpointDescription {
        types,
        ..endpoint_of(service)
    }
}

/// Remote endpoints on the fake middleware's endpoint of a service. Their
/// messages are in the wire form the translator `T` expects.
pub(crate) struct FakeRemoteEndpoints<T> {
    service: ServiceDescription,
    description: FakeEndpointDescription,
    remote: FakeEndpoints,
    header_size: usize,
    _translator: PhantomData<T>,
}

impl<T: WireForm> FakeRemoteEndpoints<T> {
    /// Creates remote endpoints on `middleware`'s endpoint of `service`.
    pub(crate) fn on(middleware: &FakeMiddleware, service: ServiceDescription) -> Self {
        let description = endpoint_for::<T>(&service);
        let remote = middleware.remote_endpoints(&description);
        let header_size = match service.types() {
            ServiceTypes::PublishSubscribe(types) => types.user_header.size,
            ServiceTypes::Event => 0,
            ServiceTypes::RequestResponse(_) => {
                unimplemented!("the fake middleware has no request-response endpoints")
            }
        };
        Self {
            service,
            description,
            remote,
            header_size,
            _translator: PhantomData,
        }
    }
}

/// The value of a message is its bytes. The fake middleware imposes no
/// form of its own on them.
impl<T: WireForm> MessageEndpoints<FakeEndpointSettings, EndpointTypes<FakeSampleTypes>>
    for FakeRemoteEndpoints<T>
{
    type Value = Vec<u8>;

    fn description(&self) -> &FakeEndpointDescription {
        &self.description
    }

    fn value(&self, n: u64) -> Vec<u8> {
        n.to_le_bytes().to_vec()
    }

    fn encode(&self, value: &Vec<u8>) -> Vec<u8> {
        value.clone()
    }

    fn decode(&self, bytes: &[u8]) -> Vec<u8> {
        bytes.to_vec()
    }

    fn send(&self, value: Vec<u8>) {
        self.remote.send(&value);
    }

    fn receive(&self) -> Option<Vec<u8>> {
        self.remote.receive()
    }
}

impl<T: WireForm> Discoverable for FakeRemoteEndpoints<T> {
    fn service(&self) -> &ServiceDescription {
        &self.service
    }
}

impl<T: WireForm> PropagatesPayloads<Payload> for FakeRemoteEndpoints<T> {
    fn send_payload(&self, payload: Payload) {
        let mut message = alloc::vec![0; self.header_size];
        message.extend_from_slice(&T::encode_payload(payload));
        self.remote.send(&message);
    }

    fn receive_payload(&self) -> Option<Payload> {
        let message = self.remote.receive()?;
        let (_, payload) = message.split_at(self.header_size);
        Some(T::decode_payload(payload.try_into().expect("a payload")))
    }
}

impl<T: WireForm> PropagatesPayloadsWithHeaders<Header, Payload> for FakeRemoteEndpoints<T> {
    fn send_with_header(&self, header: Header, payload: Payload) {
        self.remote
            .send(&[T::encode_header(header), T::encode_payload(payload)].concat());
    }

    fn receive_with_header(&self) -> Option<(Header, Payload)> {
        let message = self.remote.receive()?;
        let (header, payload) = message.split_at(self.header_size);
        Some((
            T::decode_header(header.try_into().expect("a header")),
            T::decode_payload(payload.try_into().expect("a payload")),
        ))
    }
}
