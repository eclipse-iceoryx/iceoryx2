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
    // NOLINTNEXTLINE(hicpp-avoid-c-arrays, cppcoreguidelines-avoid-c-arrays, modernize-avoid-c-arrays) used as an uninitialized buffer
    char buffer[Attribute::Key::capacity()];
    iox2_attribute_key(m_handle, &buffer[0], Attribute::Key::capacity());
    return { iox::TruncateToCapacity, &buffer[0] };
}

auto AttributeView::value() const -> Attribute::Value {
    // NOLINTNEXTLINE(hicpp-avoid-c-arrays, cppcoreguidelines-avoid-c-arrays, modernize-avoid-c-arrays) used as an uninitialized buffer
    char buffer[Attribute::Key::capacity()];
    iox2_attribute_value(m_handle, &buffer[0], Attribute::Value::capacity());

    return { iox::TruncateToCapacity, &buffer[0] };
}

AttributeView::AttributeView(iox2_attribute_h_ref handle)
    : m_handle { handle } {
}
} // namespace iox2

auto operator<<(std::ostream& stream, const iox2::AttributeView& value) -> std::ostream& {
    stream << "Attribute { key = \"" << value.key().c_str() << "\", value = \"" << value.value().c_str() << "\" }";
    return stream;
}
