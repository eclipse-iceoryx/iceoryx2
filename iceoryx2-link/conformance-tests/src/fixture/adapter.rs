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
use iceoryx2::service::messaging_pattern::MessagingPattern;
use iceoryx2::testing::generate_isolated_config;
use iceoryx2_link_adapter::{Adapter, EndpointDescription, EndpointTypes};

/// One middleware, the adapters on it, and the remote endpoints on it.
pub trait AdapterFixture {
    /// The middleware's types of a sample.
    type SampleTypes: Clone + PartialEq + 'static;
    type Adapter: Adapter<EndpointTypes = EndpointTypes<Self::SampleTypes>>;
    /// Remote endpoints of a pattern. A suite bounds them by what it needs
    /// of them.
    type RemoteEndpoints: Describable<
            Settings = <Self::Adapter as Adapter>::EndpointSettings,
            Types = EndpointTypes<Self::SampleTypes>,
        >;

    /// A fresh, isolated middleware.
    fn new() -> Self;

    /// The configuration to use in the fixture.
    fn config(&self) -> Config {
        generate_isolated_config()
    }

    /// An adapter on the middleware.
    fn adapter(&mut self) -> Self::Adapter;

    /// Creates remote endpoints of `pattern` with a new description.
    /// Dropping them removes them from the middleware.
    fn remote_endpoints(&mut self, pattern: MessagingPattern) -> Self::RemoteEndpoints;
}

/// Remote endpoints with a description the gateway's endpoints open on.
pub trait Describable {
    /// The middleware's settings of an endpoint.
    type Settings;
    /// The middleware's types of an endpoint.
    type Types;

    /// The description of the endpoints.
    fn description(&self) -> &EndpointDescription<Self::Settings, Self::Types>;

    /// Waits until the gateway's endpoints opened on the description are
    /// connected to these, or `timeout` passes. Immediate on a middleware
    /// that delivers without matching first.
    fn sync(&self, _timeout: Duration) -> bool {
        true
    }
}

/// A message of a response stream as the remote endpoints see it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResponseMessage<V> {
    Response(V),
    End,
}

/// Remote endpoints that send and receive the middleware's requests and
/// responses as values. They encode values into the wire form the
/// gateway's endpoints use.
pub trait PropagatesRequestResponseMessages {
    /// The value of a request.
    type Request: Clone + PartialEq + Debug;
    /// The value of a response.
    type Response: Clone + PartialEq + Debug;
    /// The middleware's identity of a request.
    type RequestId: Copy + PartialEq + Debug;

    /// The request the suites send for `n`. Different `n` give different
    /// requests.
    fn request(&self, n: u64) -> Self::Request;

    /// The response the suites send for `n`. Different `n` give different
    /// responses.
    fn response(&self, n: u64) -> Self::Response;

    /// The bytes of a message carrying `request`.
    fn encode_request(&self, request: &Self::Request) -> Vec<u8>;

    /// The request the message `bytes` carries.
    fn decode_request(&self, bytes: &[u8]) -> Self::Request;

    /// The bytes of a message carrying `response`.
    fn encode_response(&self, response: &Self::Response) -> Vec<u8>;

    /// The response the message `bytes` carries.
    fn decode_response(&self, bytes: &[u8]) -> Self::Response;

    /// Sends `request`.
    fn send_request(&self, request: Self::Request) -> Self::RequestId;

    /// The next request pending, if any, its id and its value.
    fn receive_request(&self) -> Option<(Self::RequestId, Self::Request)>;

    /// Sends `response` to the request `id`.
    fn send_response(&self, id: Self::RequestId, response: Self::Response);

    /// Ends the response stream of the request `id`.
    fn end_response_stream(&self, id: Self::RequestId);

    /// The next message of a response stream pending, if any, with the id
    /// of its request.
    fn receive_response(&self) -> Option<(Self::RequestId, ResponseMessage<Self::Response>)>;
}

/// Remote endpoints that send and receive the middleware's messages as
/// values. They encode values into the wire form the gateway's endpoints
/// use.
pub trait PropagatesMessages {
    /// The value of a message.
    type Value: Clone + PartialEq + Debug;

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
}
