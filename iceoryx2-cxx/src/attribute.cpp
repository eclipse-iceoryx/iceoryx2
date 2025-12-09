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

#include "iox2/attribute.hpp"
#include "iox2/container/static_string.hpp"
#include "iox2/legacy/string.hpp"

namespace iox2 {
auto AttributeView::key() const -> Attribute::Key {
    // TODO: implement unsafe_raw_access() for StaticString
    legacy::string<IOX2_ATTRIBUTE_KEY_LENGTH> key("");
    key.unsafe_raw_access([this](auto* buffer, const auto& info) -> auto {
        iox2_attribute_key(m_handle, buffer, info.total_size);
        return iox2_attribute_key_len(m_handle);
    });
    return *container::StaticString<IOX2_ATTRIBUTE_KEY_LENGTH>::from_utf8_null_terminated_unchecked(key.c_str());
}

auto AttributeView::value() const -> Attribute::Value {
    legacy::string<IOX2_ATTRIBUTE_VALUE_LENGTH> value("");
    value.unsafe_raw_access([this](auto* buffer, const auto& info) -> auto {
        iox2_attribute_value(m_handle, buffer, info.total_size);
        return iox2_attribute_value_len(m_handle);
    });
    return *container::StaticString<IOX2_ATTRIBUTE_VALUE_LENGTH>::from_utf8_null_terminated_unchecked(value.c_str());
}

AttributeView::AttributeView(iox2_attribute_h_ref handle)
    : m_handle { handle } {
}
} // namespace iox2

auto operator<<(std::ostream& stream, const iox2::AttributeView& value) -> std::ostream& {
    stream << "Attribute { key = \"" << value.key().unchecked_access().c_str() << "\", value = \""
           << value.value().unchecked_access().c_str() << "\" }";
    return stream;
}
