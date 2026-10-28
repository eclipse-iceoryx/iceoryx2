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

use alloc::collections::{BTreeMap, VecDeque};
use alloc::rc::Rc;
use alloc::vec::Vec;

use iceoryx2::service::service_name::ServiceName;
use iceoryx2_bb_concurrency::cell::{Ref, RefCell};
use iceoryx2_link_adapter::EndpointTypes;
use iceoryx2_link_backend::WakeHandle;

use crate::adapter::{
    FakeAdapter, FakeEndpointDescription, FakePublishSubscribeEndpoints,
    FakeRequestResponseEndpoints,
};

/// The bytes of one message as the middleware carries them.
pub type Message = Vec<u8>;

/// A borrowed [`Message`].
pub type MessageRef<'a> = &'a [u8];

/// The middleware's identity of an endpoint.
pub(super) type EndpointId = u64;

/// The middleware's identity of a request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct FakeRequestId(u64);

/// A request as the middleware carries it.
pub type FakeRequestMessage = Message;

/// A message of a response stream as the middleware carries it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FakeResponseMessage {
    Response(Message),
    End,
}

/// Who created an endpoint. Only remote endpoints make a description one
/// the adapter lists.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Origin {
    Remote,
    Gateway,
}

/// What an endpoint receives. The pattern of its description decides it.
enum Inbox {
    Samples(VecDeque<Message>),
    RequestResponse {
        requests: VecDeque<(FakeRequestId, FakeRequestMessage)>,
        responses: VecDeque<(FakeRequestId, FakeResponseMessage)>,
    },
}

impl Inbox {
    /// The samples queue. The endpoint must be publish-subscribe.
    fn samples(&mut self) -> &mut VecDeque<Message> {
        match self {
            Self::Samples(samples) => samples,
            Self::RequestResponse { .. } => panic!("a request-response endpoint has no samples"),
        }
    }

    /// The requests queue. The endpoint must be request-response.
    fn requests(&mut self) -> &mut VecDeque<(FakeRequestId, FakeRequestMessage)> {
        match self {
            Self::RequestResponse { requests, .. } => requests,
            Self::Samples(_) => panic!("a publish-subscribe endpoint has no requests"),
        }
    }

    /// The responses queue. The endpoint must be request-response.
    fn responses(&mut self) -> &mut VecDeque<(FakeRequestId, FakeResponseMessage)> {
        match self {
            Self::RequestResponse { responses, .. } => responses,
            Self::Samples(_) => panic!("a publish-subscribe endpoint has no responses"),
        }
    }

    fn for_description(description: &FakeEndpointDescription) -> Self {
        match description.types {
            EndpointTypes::RequestResponse { .. } => Self::RequestResponse {
                requests: VecDeque::new(),
                responses: VecDeque::new(),
            },
            EndpointTypes::PublishSubscribe(_) | EndpointTypes::Event => {
                Self::Samples(VecDeque::new())
            }
        }
    }
}

pub(super) struct Endpoint {
    pub(super) origin: Origin,
    inbox: Inbox,
}

/// The endpoints associated with a single description.
pub(super) struct Topic {
    pub(super) description: FakeEndpointDescription,
    pub(super) endpoints: BTreeMap<EndpointId, Endpoint>,
    /// The endpoint each open request was sent from.
    open_requests: BTreeMap<FakeRequestId, EndpointId>,
}

#[derive(Default)]
pub(super) struct State {
    pub(super) topics: BTreeMap<ServiceName, Topic>,
    /// Moved whenever endpoints join or leave.
    pub(super) generation: u64,
    next_endpoint_id: u64,
    next_request_id: u64,
    /// The wakes of the adapters that attached one.
    pub(super) wakes: Vec<WakeHandle>,
}

impl State {
    fn wake_adapters(&self) {
        for wake in &self.wakes {
            wake.signal();
        }
    }
}

/// A fake substitute for a middleware. Endpoints with one description
/// receive each other's messages.
#[derive(Clone, Default)]
pub struct FakeMiddleware {
    state: Rc<RefCell<State>>,
}

impl FakeMiddleware {
    /// Creates an empty middleware.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates an adapter connected to the middleware.
    pub fn adapter(&self) -> FakeAdapter {
        FakeAdapter::new(self.clone())
    }

    /// Creates remote publish-subscribe endpoints with `description`, as
    /// an application of the middleware would.
    pub fn remote_publish_subscribe_endpoints(
        &self,
        description: &FakeEndpointDescription,
    ) -> FakePublishSubscribeEndpoints {
        let id = self.join(description, Origin::Remote);
        FakePublishSubscribeEndpoints::new(self.clone(), description, id)
    }

    /// Creates remote request-response endpoints with `description`, as
    /// an application of the middleware would.
    pub fn remote_request_response_endpoints(
        &self,
        description: &FakeEndpointDescription,
    ) -> FakeRequestResponseEndpoints {
        let id = self.join(description, Origin::Remote);
        FakeRequestResponseEndpoints::new(self.clone(), description, id)
    }

    pub(super) fn state(&self) -> Ref<'_, State> {
        self.state.borrow()
    }

    pub(super) fn attach(&self, wake: WakeHandle) {
        self.state.borrow_mut().wakes.push(wake);
    }

    pub(super) fn join(&self, description: &FakeEndpointDescription, origin: Origin) -> EndpointId {
        let mut state = self.state.borrow_mut();
        let id = state.next_endpoint_id;
        state.next_endpoint_id += 1;
        state.generation = state.generation.wrapping_add(1);
        state
            .topics
            .entry(description.settings.name)
            .or_insert_with(|| Topic {
                description: description.clone(),
                endpoints: BTreeMap::new(),
                open_requests: BTreeMap::new(),
            })
            .endpoints
            .insert(
                id,
                Endpoint {
                    origin,
                    inbox: Inbox::for_description(description),
                },
            );
        state.wake_adapters();
        id
    }

    pub(super) fn leave(&self, name: &ServiceName, id: EndpointId) {
        let mut state = self.state.borrow_mut();
        state.generation = state.generation.wrapping_add(1);
        if let Some(topic) = state.topics.get_mut(name) {
            topic.endpoints.remove(&id);
            if topic.endpoints.is_empty() {
                state.topics.remove(name);
            }
        }
        state.wake_adapters();
    }

    /// Sends a sample from `from` to every other endpoint with the
    /// description.
    pub(super) fn send_sample(
        &self,
        name: &ServiceName,
        from: EndpointId,
        message: MessageRef<'_>,
    ) {
        let mut state = self.state.borrow_mut();
        let Some(topic) = state.topics.get_mut(name) else {
            return;
        };
        for (_, remote) in topic.endpoints.iter_mut().filter(|(id, _)| **id != from) {
            remote.inbox.samples().push_back(message.to_vec());
        }
        state.wake_adapters();
    }

    /// The next sample pending at the endpoint `id`, if any.
    pub(super) fn receive_sample(&self, name: &ServiceName, id: EndpointId) -> Option<Message> {
        let mut state = self.state.borrow_mut();
        let endpoint = state.topics.get_mut(name)?.endpoints.get_mut(&id)?;
        endpoint.inbox.samples().pop_front()
    }

    /// Sends a request from `from` to every other endpoint with the
    /// description.
    ///
    /// Returns the id the middleware gave the request.
    pub(super) fn send_request(
        &self,
        name: &ServiceName,
        from: EndpointId,
        message: MessageRef<'_>,
    ) -> FakeRequestId {
        let mut state = self.state.borrow_mut();
        let id = FakeRequestId(state.next_request_id);
        state.next_request_id += 1;
        let Some(topic) = state.topics.get_mut(name) else {
            return id;
        };
        topic.open_requests.insert(id, from);
        for (_, remote) in topic.endpoints.iter_mut().filter(|(id, _)| **id != from) {
            remote.inbox.requests().push_back((id, message.to_vec()));
        }
        state.wake_adapters();
        id
    }

    /// The next request pending at the endpoint `id`, if any.
    pub(super) fn receive_request(
        &self,
        name: &ServiceName,
        id: EndpointId,
    ) -> Option<(FakeRequestId, FakeRequestMessage)> {
        let mut state = self.state.borrow_mut();
        let endpoint = state.topics.get_mut(name)?.endpoints.get_mut(&id)?;
        endpoint.inbox.requests().pop_front()
    }

    /// Sends a response to the endpoint the request `request` was sent
    /// from. Dropped if that request is not open.
    pub(super) fn send_response(
        &self,
        name: &ServiceName,
        request: FakeRequestId,
        message: MessageRef<'_>,
    ) {
        self.deliver_response(
            name,
            request,
            FakeResponseMessage::Response(message.to_vec()),
        );
    }

    /// The next response or end pending at the endpoint `id`, if any.
    pub(super) fn receive_response(
        &self,
        name: &ServiceName,
        id: EndpointId,
    ) -> Option<(FakeRequestId, FakeResponseMessage)> {
        let mut state = self.state.borrow_mut();
        let endpoint = state.topics.get_mut(name)?.endpoints.get_mut(&id)?;
        endpoint.inbox.responses().pop_front()
    }

    /// Ends the responses to the request `request` for the endpoint it was
    /// sent from. The request is closed.
    pub(super) fn end_response_stream(&self, name: &ServiceName, request: FakeRequestId) {
        self.deliver_response(name, request, FakeResponseMessage::End);
        if let Some(topic) = self.state.borrow_mut().topics.get_mut(name) {
            topic.open_requests.remove(&request);
        }
    }

    fn deliver_response(
        &self,
        name: &ServiceName,
        request: FakeRequestId,
        response: FakeResponseMessage,
    ) {
        let mut state = self.state.borrow_mut();
        let Some(topic) = state.topics.get_mut(name) else {
            return;
        };
        let Some(origin) = topic.open_requests.get(&request).copied() else {
            return;
        };
        let Some(endpoint) = topic.endpoints.get_mut(&origin) else {
            return;
        };
        endpoint.inbox.responses().push_back((request, response));
        state.wake_adapters();
    }
}
