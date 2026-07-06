// Copyright (c) 2026 Contributors to the Eclipse Foundation
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

#include "iox2/port_name.hpp"
#include "iox2/bb/detail/assertions.hpp"
#include "iox2/bb/expected.hpp"
#include "iox2/internal/iceoryx2.hpp"

#include <cstring>

namespace iox2 {
auto PortNameView::to_string() const -> iox2::bb::StaticString<IOX2_PORT_NAME_LENGTH> {
    size_t len = 0;
    const auto* chars = iox2_port_name_as_chars(m_ptr, &len);
    return iox2::bb::StaticString<IOX2_PORT_NAME_LENGTH>::from_utf8_null_terminated_unchecked_truncated(
        chars, IOX2_PORT_NAME_LENGTH);
}

auto PortNameView::to_owned() const -> PortName {
    size_t len = 0;
    const auto* chars = iox2_port_name_as_chars(m_ptr, &len);
    auto port_name = PortName::create_impl(chars, len);
    if (!port_name.has_value()) {
        IOX2_PANIC("PortNameView contains always valid PortName)");
    }
    return port_name.value();
}

PortNameView::PortNameView(iox2_port_name_ptr ptr)
    : m_ptr { ptr } {
}

PortName::PortName(iox2_port_name_h handle)
    : m_handle { handle } {
}

PortName::~PortName() {
    drop();
}

PortName::PortName(PortName&& rhs) noexcept {
    *this = std::move(rhs);
}

auto PortName::operator=(PortName&& rhs) noexcept -> PortName& {
    if (this != &rhs) {
        drop();
        m_handle = rhs.m_handle;
        rhs.m_handle = nullptr;
    }

    return *this;
}

PortName::PortName(const PortName& rhs) {
    *this = rhs;
}

auto PortName::operator=(const PortName& rhs) -> PortName& {
    if (this != &rhs) {
        drop();

        const auto* ptr = iox2_cast_port_name_ptr(rhs.m_handle);
        size_t len = 0;
        const auto* chars = iox2_port_name_as_chars(ptr, &len);
        IOX2_ASSERT(iox2_port_name_new(nullptr, chars, len, &m_handle) == IOX2_OK,
                    "PortName shall always contain a valid value.");
    }

    return *this;
}

void PortName::drop() noexcept {
    if (m_handle != nullptr) {
        iox2_port_name_drop(m_handle);
        m_handle = nullptr;
    }
}

auto PortName::create(const char* value) -> iox2::bb::Expected<PortName, bb::SemanticStringError> {
    return PortName::create_impl(value, strnlen(value, IOX2_PORT_NAME_LENGTH + 1));
}

auto PortName::create_impl(const char* value, size_t value_len)
    -> iox2::bb::Expected<PortName, bb::SemanticStringError> {
    if (value_len > IOX2_PORT_NAME_LENGTH) {
        return bb::err(bb::SemanticStringError::ExceedsMaximumLength);
    }

    iox2_port_name_h handle {};
    const auto ret_val = iox2_port_name_new(nullptr, value, value_len, &handle);
    if (ret_val == IOX2_OK) {
        return PortName { handle };
    }

    return bb::err(iox2::bb::into<bb::SemanticStringError>(ret_val));
}

auto PortName::to_string() const -> iox2::bb::StaticString<IOX2_PORT_NAME_LENGTH> {
    return as_view().to_string();
}

auto PortName::as_view() const -> PortNameView {
    return PortNameView(iox2_cast_port_name_ptr(m_handle));
}

} // namespace iox2
