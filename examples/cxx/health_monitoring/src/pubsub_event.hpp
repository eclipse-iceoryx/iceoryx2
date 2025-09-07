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

#ifndef IOX2_EXAMPLE_HEALTH_MONITORING_PUBSUB_EVENT_HPP
#define IOX2_EXAMPLE_HEALTH_MONITORING_PUBSUB_EVENT_HPP

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
    ProcessDied = 7,
    Unknown = 8,
};

struct ServiceTuple {
    iox2::PortFactoryEvent<iox2::ServiceType::Ipc> event;
    iox2::PortFactoryPublishSubscribe<iox2::ServiceType::Ipc, uint64_t, void> pubsub;
};

inline auto open_service(const iox2::Node<iox2::ServiceType::Ipc>& node, const iox2::ServiceName& service_name)
    -> ServiceTuple {
    auto service_pubsub = node.service_builder(service_name).publish_subscribe<uint64_t>().open().expect("");
    auto service_event = node.service_builder(service_name).event().open().expect("");

    return { std::move(service_event), std::move(service_pubsub) };
}

namespace iox {
template <>
inline auto from<PubSubEvent, iox2::EventId>(const PubSubEvent value) noexcept -> iox2::EventId {
    return iox2::EventId(static_cast<uint64_t>(value));
}
} // namespace iox


#endif
