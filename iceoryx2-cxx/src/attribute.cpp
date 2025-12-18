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
#include "iox2/bb/static_string.hpp"

namespace iox2 {
auto AttributeView::key() const -> Attribute::Key {
    // NOLINTNEXTLINE(hicpp-avoid-c-arrays, cppcoreguidelines-avoid-c-arrays, modernize-avoid-c-arrays) temporary storage, required by C API
    char buffer[IOX2_ATTRIBUTE_KEY_LENGTH];
    iox2_attribute_key(m_handle, &buffer[0], IOX2_ATTRIBUTE_KEY_LENGTH);
    return bb::StaticString<IOX2_ATTRIBUTE_KEY_LENGTH>::from_utf8_unchecked(buffer);
}

auto AttributeView::value() const -> Attribute::Value {
    // NOLINTNEXTLINE(hicpp-avoid-c-arrays, cppcoreguidelines-avoid-c-arrays, modernize-avoid-c-arrays) temporary storage, required by C API
    char buffer[IOX2_ATTRIBUTE_VALUE_LENGTH];
    iox2_attribute_value(m_handle, &buffer[0], IOX2_ATTRIBUTE_VALUE_LENGTH);
    return bb::StaticString<IOX2_ATTRIBUTE_VALUE_LENGTH>::from_utf8_unchecked(buffer);
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
