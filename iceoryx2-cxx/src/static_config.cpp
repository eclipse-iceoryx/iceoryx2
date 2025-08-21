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
#include "iox/into.hpp"

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
    return iox::into<MessagingPattern>(static_cast<int>(m_value.messaging_pattern));
}
} // namespace iox2

auto operator<<(std::ostream& stream, const iox2::StaticConfig& value) -> std::ostream& {
    stream << "iox2::StaticConfig { id: " << value.id() << ", name: " << value.name()
           << ", messaging_pattern: " << value.messaging_pattern() << " }";
    return stream;
}
