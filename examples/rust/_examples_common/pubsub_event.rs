// Copyright (c) 2024 Contributors to the Eclipse Foundation
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

use iceoryx2::port::event_id::EventId;

pub enum PubSubEvent {
    PublisherConnected = 0,
    PublisherDisconnected = 1,
    SubscriberConnected = 2,
    SubscriberDisconnected = 3,
    SentSample = 4,
    ReceivedSample = 5,
    SentHistory = 6,
    ProcessDied = 7,
    Unknown,
}

impl From<PubSubEvent> for EventId {
    fn from(value: PubSubEvent) -> Self {
        EventId::new(value as usize)
    }
}

impl From<EventId> for PubSubEvent {
    fn from(value: EventId) -> Self {
        match value.as_value() {
            0 => PubSubEvent::PublisherConnected,
            1 => PubSubEvent::PublisherDisconnected,
            2 => PubSubEvent::SubscriberConnected,
            3 => PubSubEvent::SubscriberDisconnected,
            4 => PubSubEvent::SentSample,
            5 => PubSubEvent::ReceivedSample,
            6 => PubSubEvent::SentHistory,
            7 => PubSubEvent::ProcessDied,
            _ => PubSubEvent::Unknown,
        }
    }
}
