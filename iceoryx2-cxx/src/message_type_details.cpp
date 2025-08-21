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

#include "iox2/message_type_details.hpp"
#include "iox/into.hpp"

namespace iox2 {
TypeDetail::TypeDetail(iox2_type_detail_t value)
    : m_value { value } {
}

auto TypeDetail::variant() const -> TypeVariant {
    return iox::into<TypeVariant>(static_cast<int>(m_value.variant));
}

auto TypeDetail::type_name() const -> const char* {
    return &m_value.type_name[0];
}

auto TypeDetail::size() const -> size_t {
    return m_value.size;
}

auto TypeDetail::alignment() const -> size_t {
    return m_value.alignment;
}

MessageTypeDetails::MessageTypeDetails(iox2_message_type_details_t value)
    : m_value { value } {
}

auto MessageTypeDetails::header() const -> TypeDetail {
    return TypeDetail(m_value.header);
}

auto MessageTypeDetails::user_header() const -> TypeDetail {
    return TypeDetail(m_value.user_header);
}

auto MessageTypeDetails::payload() const -> TypeDetail {
    return TypeDetail(m_value.payload);
}
} // namespace iox2
