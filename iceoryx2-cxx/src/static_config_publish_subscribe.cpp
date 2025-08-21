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

#include "iox2/static_config_publish_subscribe.hpp"

namespace iox2 {
StaticConfigPublishSubscribe::StaticConfigPublishSubscribe(iox2_static_config_publish_subscribe_t value)
    : m_value { value } {
}

auto StaticConfigPublishSubscribe::max_nodes() const -> uint64_t {
    return m_value.max_nodes;
}

auto StaticConfigPublishSubscribe::max_publishers() const -> uint64_t {
    return m_value.max_publishers;
}

auto StaticConfigPublishSubscribe::max_subscribers() const -> uint64_t {
    return m_value.max_subscribers;
}

auto StaticConfigPublishSubscribe::history_size() const -> uint64_t {
    return m_value.history_size;
}

auto StaticConfigPublishSubscribe::subscriber_max_buffer_size() const -> uint64_t {
    return m_value.subscriber_max_buffer_size;
}

auto StaticConfigPublishSubscribe::subscriber_max_borrowed_samples() const -> uint64_t {
    return m_value.subscriber_max_borrowed_samples;
}

auto StaticConfigPublishSubscribe::has_safe_overflow() const -> bool {
    return m_value.enable_safe_overflow;
}

auto StaticConfigPublishSubscribe::message_type_details() const -> MessageTypeDetails {
    return MessageTypeDetails(m_value.message_type_details);
}


} // namespace iox2
