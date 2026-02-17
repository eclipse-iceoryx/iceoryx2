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

#include "iox2/static_config.hpp"
#include "iox2/bb/detail/assertions.hpp"
#include "iox2/bb/into.hpp"
#include "iox2/messaging_pattern.hpp"
#include "iox2/static_config_blackboard.hpp"
#include "iox2/static_config_request_response.hpp"

namespace iox2 {
StaticConfig::StaticConfig(iox2_static_config_t value)
    : m_value { value } {
}

StaticConfig::StaticConfig(StaticConfig&& rhs) noexcept
    : m_value { std::move(rhs.m_value) } {
    rhs.m_value.attributes = nullptr;
}

auto StaticConfig::operator=(StaticConfig&& rhs) noexcept -> StaticConfig& {
    if (this != &rhs) {
        drop();
        m_value = std::move(rhs.m_value);
        rhs.m_value.attributes = nullptr;
    }
    return *this;
}

StaticConfig::~StaticConfig() {
    drop();
}

void StaticConfig::drop() {
    if (m_value.attributes != nullptr) {
        iox2_attribute_set_drop(m_value.attributes);
        m_value.attributes = nullptr;
    }
}

auto StaticConfig::attributes() const -> AttributeSetView {
    return AttributeSetView(iox2_cast_attribute_set_ptr(m_value.attributes));
}

auto StaticConfig::id() const -> const char* {
    return &m_value.id[0];
}

auto StaticConfig::name() const -> const char* {
    return &m_value.name[0];
}

auto StaticConfig::messaging_pattern() const -> MessagingPattern {
    return iox2::bb::into<MessagingPattern>(static_cast<int>(m_value.messaging_pattern));
}

auto StaticConfig::blackboard() const -> StaticConfigBlackboard {
    IOX2_ENFORCE(messaging_pattern() == MessagingPattern::Blackboard,
                 "This is not a service with a blackboard messaging pattern.");

    // NOLINTNEXTLINE(cppcoreguidelines-pro-type-union-access) C binding does not have variants
    return StaticConfigBlackboard(m_value.details.blackboard);
}

auto StaticConfig::event() const -> StaticConfigEvent {
    IOX2_ENFORCE(messaging_pattern() == MessagingPattern::Event,
                 "This is not a service with an event messaging pattern.");

    // NOLINTNEXTLINE(cppcoreguidelines-pro-type-union-access) C binding does not have variants
    return StaticConfigEvent(m_value.details.event);
}

auto StaticConfig::publish_subscribe() const -> StaticConfigPublishSubscribe {
    IOX2_ENFORCE(messaging_pattern() == MessagingPattern::PublishSubscribe,
                 "This is not a service with a publish-subscribe messaging pattern.");

    // NOLINTNEXTLINE(cppcoreguidelines-pro-type-union-access) C binding does not have variants
    return StaticConfigPublishSubscribe(m_value.details.publish_subscribe);
}

auto StaticConfig::request_response() const -> StaticConfigRequestResponse {
    IOX2_ENFORCE(messaging_pattern() == MessagingPattern::RequestResponse,
                 "This is not a service with a request-response messaging pattern.");

    // NOLINTNEXTLINE(cppcoreguidelines-pro-type-union-access) C binding does not have variants
    return StaticConfigRequestResponse(m_value.details.request_response);
}
} // namespace iox2

auto operator<<(std::ostream& stream, const iox2::StaticConfig& value) -> std::ostream& {
    stream << "iox2::StaticConfig { id: " << value.id() << ", name: " << value.name()
           << ", messaging_pattern: " << value.messaging_pattern() << ", ";

    switch (value.messaging_pattern()) {
    case iox2::MessagingPattern::Blackboard: {
        stream << value.blackboard() << " }";
        break;
    }
    case iox2::MessagingPattern::Event: {
        stream << value.event() << " }";
        break;
    }
    case iox2::MessagingPattern::PublishSubscribe: {
        stream << value.publish_subscribe() << " }";
        break;
    }
    case iox2::MessagingPattern::RequestResponse: {
        stream << value.request_response() << " }";
        break;
    }
    }
    return stream;
}
