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

#include "iox2/attribute_specifier.hpp"

namespace iox2 {
AttributeSpecifier::AttributeSpecifier() {
    iox2_attribute_specifier_new(nullptr, &m_handle);
}

AttributeSpecifier::AttributeSpecifier(AttributeSpecifier&& rhs) noexcept
    : m_handle { std::move(rhs.m_handle) } {
    rhs.m_handle = nullptr;
}

AttributeSpecifier::~AttributeSpecifier() {
    drop();
}

auto AttributeSpecifier::operator=(AttributeSpecifier&& rhs) noexcept -> AttributeSpecifier& {
    if (this != &rhs) {
        drop();
        m_handle = std::move(rhs.m_handle);
        rhs.m_handle = nullptr;
    }

    return *this;
}

auto AttributeSpecifier::define(const Attribute::Key& key, const Attribute::Value& value) -> AttributeSpecifier&& {
    iox2_attribute_specifier_define(&m_handle, key.c_str(), value.c_str());
    return std::move(*this);
}

auto AttributeSpecifier::attributes() const -> AttributeSetView {
    return AttributeSetView(iox2_attribute_specifier_attributes(&m_handle));
}

void AttributeSpecifier::drop() {
    if (m_handle != nullptr) {
        iox2_attribute_specifier_drop(m_handle);
        m_handle = nullptr;
    }
}
} // namespace iox2
