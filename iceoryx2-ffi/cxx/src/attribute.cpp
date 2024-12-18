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

namespace iox2 {
auto AttributeView::key() const -> Attribute::Key {
    Attribute::Key key;
    key.unsafe_raw_access([this](auto* buffer, const auto& info) {
        iox2_attribute_key(m_handle, buffer, info.total_size);
        return iox2_attribute_key_len(m_handle);
    });
    return key;
}

auto AttributeView::value() const -> Attribute::Value {
    Attribute::Value value;
    value.unsafe_raw_access([this](auto* buffer, const auto& info) {
        iox2_attribute_value(m_handle, buffer, info.total_size);
        return iox2_attribute_value_len(m_handle);
    });
    return value;
}

AttributeView::AttributeView(iox2_attribute_h_ref handle)
    : m_handle { handle } {
}
} // namespace iox2

auto operator<<(std::ostream& stream, const iox2::AttributeView& value) -> std::ostream& {
    stream << "Attribute { key = \"" << value.key().c_str() << "\", value = \"" << value.value().c_str() << "\" }";
    return stream;
}
