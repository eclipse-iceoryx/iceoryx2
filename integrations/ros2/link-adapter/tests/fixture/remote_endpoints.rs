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

use core::marker::PhantomData;
use core::time::Duration;
use std::time::Instant;

use iceoryx2_integrations_ros2_link_adapter::testing::{
    PeerNode, RclPublisher, RclSubscription, take_serialized,
};
use iceoryx2_integrations_ros2_link_adapter::{
    QosProfile, TopicDescription, TopicSettings, TopicTypes,
};
use iceoryx2_link_adapter::EndpointDescription;
use iceoryx2_link_backend::service_description::ServiceDescription;
use iceoryx2_link_conformance_tests::fixture::{
    DiscoverableEndpoints, MessageEndpoints, PayloadEndpoints,
};
use iceoryx2_link_conformance_tests::parameters::PayloadShape;

use super::TranslatorUnderTest;
use super::serialization;

/// The pause between two looks at the matching counts.
pub const POLL_PERIOD: Duration = Duration::from_millis(10);

/// Remote endpoints on a topic, speaking the messages of the translator
/// under test `T`.
pub struct RemoteMessageEndpoints<T> {
    description: TopicDescription,
    publisher: RclPublisher,
    subscription: RclSubscription,
    _translator: PhantomData<T>,
}

impl<T> RemoteMessageEndpoints<T> {
    pub(super) fn new(peer: &PeerNode, description: TopicDescription, qos: QosProfile) -> Self {
        let EndpointDescription { settings, types } = &description;

        let publisher = peer.publisher(&settings.topic, &types.type_name, qos.clone());
        let subscription = peer.subscription(&settings.topic, &types.type_name, qos);

        Self {
            description,
            publisher,
            subscription,
            _translator: PhantomData,
        }
    }
}

impl<T: TranslatorUnderTest> MessageEndpoints<TopicSettings, TopicTypes>
    for RemoteMessageEndpoints<T>
{
    type Value = <T::Payload as PayloadShape>::Value;

    fn description(&self) -> &TopicDescription {
        &self.description
    }

    fn value(&self, n: u64) -> Self::Value {
        <T::Payload as PayloadShape>::value(n)
    }

    fn encode(&self, value: &Self::Value) -> Vec<u8> {
        serialization::serialize(&T::Message::from(value.clone()))
    }

    fn decode(&self, bytes: &[u8]) -> Self::Value {
        serialization::deserialize::<T::Message>(bytes).into()
    }

    fn send(&self, value: Self::Value) {
        self.publisher
            .publish(&self.encode(&value))
            .expect("the peer publishes the message");
    }

    /// The next message from another publisher than the peer's own.
    fn receive(&self) -> Option<Self::Value> {
        loop {
            let (message, info) = take_serialized(&self.subscription)?;
            if info.gid != *self.publisher.gid() {
                return Some(self.decode(&message));
            }
        }
    }

    /// Matched once the peer's publisher sees a subscription and its
    /// subscription sees a publisher beyond the the peers own.
    fn sync(&self, timeout: Duration) -> bool {
        /// The peer's own endpoints match each other.
        const OWN: usize = 1;

        let started = Instant::now();
        loop {
            let publisher_matched = self
                .publisher
                .subscription_count()
                .expect("the subscription count is read")
                > OWN;
            let subscription_matched = self
                .subscription
                .publisher_count()
                .expect("the publisher count is read")
                > OWN;
            if publisher_matched && subscription_matched {
                return true;
            }
            if started.elapsed() >= timeout {
                return false;
            }
            std::thread::sleep(POLL_PERIOD);
        }
    }
}

/// Remote endpoints on the topic a service maps to, speaking its payloads
/// as messages.
pub struct RemotePayloadEndpoints<T> {
    pub(super) service: ServiceDescription,
    pub(super) endpoints: RemoteMessageEndpoints<T>,
}

impl<T: TranslatorUnderTest> DiscoverableEndpoints for RemotePayloadEndpoints<T> {
    fn service(&self) -> &ServiceDescription {
        &self.service
    }

    fn sync(&self, timeout: Duration) -> bool {
        MessageEndpoints::sync(&self.endpoints, timeout)
    }
}

impl<T: TranslatorUnderTest> PayloadEndpoints<<T::Payload as PayloadShape>::Value>
    for RemotePayloadEndpoints<T>
{
    fn send_payload(&self, payload: <T::Payload as PayloadShape>::Value) {
        self.endpoints.send(payload);
    }

    fn receive_payload(&self) -> Option<<T::Payload as PayloadShape>::Value> {
        self.endpoints.receive()
    }
}
