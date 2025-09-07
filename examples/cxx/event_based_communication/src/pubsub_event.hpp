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

#ifndef IOX2_EXAMPLES_PUBSUB_EVENT_HPP
#define IOX2_EXAMPLES_PUBSUB_EVENT_HPP

#include "iox/into.hpp"
#include "iox2/iceoryx2.hpp"

#include <cstdint>

enum class PubSubEvent : uint8_t {
    PublisherConnected = 0,
    PublisherDisconnected = 1,
    SubscriberConnected = 2,
    SubscriberDisconnected = 3,
    SentSample = 4,
    ReceivedSample = 5,
    SentHistory = 6,
    Unknown = 7
};

namespace iox {
template <>
constexpr auto from<PubSubEvent, size_t>(const PubSubEvent value) noexcept -> size_t {
    return static_cast<uint8_t>(value);
}

template <>
constexpr auto from<size_t, PubSubEvent>(const size_t value) noexcept -> PubSubEvent {
    switch (value) {
    case into<size_t>(PubSubEvent::PublisherConnected):
        return PubSubEvent::PublisherConnected;
    case into<size_t>(PubSubEvent::PublisherDisconnected):
        return PubSubEvent::PublisherDisconnected;
    case into<size_t>(PubSubEvent::SubscriberConnected):
        return PubSubEvent::SubscriberConnected;
    case into<size_t>(PubSubEvent::SubscriberDisconnected):
        return PubSubEvent::SubscriberDisconnected;
    case into<size_t>(PubSubEvent::SentSample):
        return PubSubEvent::SentSample;
    case into<size_t>(PubSubEvent::ReceivedSample):
        return PubSubEvent::ReceivedSample;
    case into<size_t>(PubSubEvent::SentHistory):
        return PubSubEvent::SentHistory;
    default:
        return PubSubEvent::Unknown;
    }
    IOX_UNREACHABLE();
}
} // namespace iox


#endif
