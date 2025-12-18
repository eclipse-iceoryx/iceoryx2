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

#include "iox2/attribute_verifier.hpp"
#include "iox2/bb/expected.hpp"
#include "iox2/bb/static_vector.hpp"

namespace iox2 {
AttributeVerifier::AttributeVerifier() {
    iox2_attribute_verifier_new(nullptr, &m_handle);
}

AttributeVerifier::AttributeVerifier(AttributeVerifier&& rhs) noexcept
    : m_handle { std::move(rhs.m_handle) } {
    rhs.m_handle = nullptr;
}

void AttributeVerifier::drop() {
    if (m_handle != nullptr) {
        iox2_attribute_verifier_drop(m_handle);
        m_handle = nullptr;
    }
}

AttributeVerifier::~AttributeVerifier() {
    drop();
}

auto AttributeVerifier::operator=(AttributeVerifier&& rhs) noexcept -> AttributeVerifier& {
    if (this != &rhs) {
        drop();
        m_handle = std::move(rhs.m_handle);
        rhs.m_handle = nullptr;
    }

    return *this;
}

auto AttributeVerifier::require(const Attribute::Key& key, const Attribute::Value& value)
    -> bb::Expected<void, AttributeDefinitionError> {
    auto result =
        iox2_attribute_verifier_require(&m_handle, key.unchecked_access().c_str(), value.unchecked_access().c_str());
    if (result == IOX2_OK) {
        return {};
    }

    return bb::err(bb::into<AttributeDefinitionError>(result));
}

auto AttributeVerifier::require_key(const Attribute::Key& key) -> bb::Expected<void, AttributeDefinitionError> {
    auto result = iox2_attribute_verifier_require_key(&m_handle, key.unchecked_access().c_str());
    if (result == IOX2_OK) {
        return {};
    }

    return bb::err(bb::into<AttributeDefinitionError>(result));
}

auto AttributeVerifier::attributes() const -> AttributeSetView {
    return AttributeSetView(iox2_attribute_verifier_attributes(&m_handle));
}

auto AttributeVerifier::keys() const -> iox2::bb::StaticVector<Attribute::Key, IOX2_MAX_ATTRIBUTES_PER_SERVICE> {
    auto number_of_keys = iox2_attribute_verifier_number_of_keys(&m_handle);
    iox2::bb::StaticVector<Attribute::Key, IOX2_MAX_ATTRIBUTES_PER_SERVICE> attributes;
    for (uint64_t i = 0; i < number_of_keys; ++i) {
        // NOLINTNEXTLINE(hicpp-avoid-c-arrays, cppcoreguidelines-avoid-c-arrays, modernize-avoid-c-arrays) used as an uninitialized buffer
        char buffer[Attribute::Key::capacity()];
        iox2_attribute_verifier_key(&m_handle, i, &buffer[0], Attribute::Key::capacity());
        attributes.try_push_back(Attribute::Key::from_utf8_unchecked(buffer));
    }

    return attributes;
}

auto AttributeVerifier::verify_requirements(const AttributeSetView& rhs) const -> bb::Expected<void, Attribute::Key> {
    // NOLINTNEXTLINE(hicpp-avoid-c-arrays, cppcoreguidelines-avoid-c-arrays, modernize-avoid-c-arrays) used as an uninitialized buffer
    char buffer[Attribute::Key::capacity()];
    if (iox2_attribute_verifier_verify_requirements(&m_handle, rhs.m_handle, &buffer[0], Attribute::Key::capacity())
        == IOX2_OK) {
        return {};
    }

    return bb::err(Attribute::Key::from_utf8_unchecked(buffer));
}


} // namespace iox2
