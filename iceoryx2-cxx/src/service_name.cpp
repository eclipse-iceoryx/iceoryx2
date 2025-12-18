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

#include "iox2/service_name.hpp"
#include "iox2/bb/detail/assertions.hpp"
#include "iox2/internal/iceoryx2.hpp"

#include <cstring>

namespace iox2 {
auto ServiceNameView::to_string() const -> iox2::bb::StaticString<IOX2_SERVICE_NAME_LENGTH> {
    size_t len = 0;
    const auto* chars = iox2_service_name_as_chars(m_ptr, &len);
    return iox2::bb::StaticString<IOX2_SERVICE_NAME_LENGTH>::from_utf8_null_terminated_unchecked_truncated(
        chars, IOX2_SERVICE_NAME_LENGTH);
}

auto ServiceNameView::to_owned() const -> ServiceName {
    size_t len = 0;
    const auto* chars = iox2_service_name_as_chars(m_ptr, &len);
    auto service_name_result = ServiceName::create_impl(chars, len);
    IOX2_ASSERT(service_name_result.has_value(), "ServiceNameView shall contain valid ServiceName");
    return service_name_result.value();
}

ServiceNameView::ServiceNameView(iox2_service_name_ptr ptr)
    : m_ptr { ptr } {
}

ServiceName::ServiceName(iox2_service_name_h handle)
    : m_handle { handle } {
}

ServiceName::~ServiceName() {
    drop();
}

ServiceName::ServiceName(ServiceName&& rhs) noexcept {
    *this = std::move(rhs);
}

auto ServiceName::operator=(ServiceName&& rhs) noexcept -> ServiceName& {
    if (this != &rhs) {
        drop();
        m_handle = std::move(rhs.m_handle);
        rhs.m_handle = nullptr;
    }

    return *this;
}

ServiceName::ServiceName(const ServiceName& rhs) {
    *this = rhs;
}

auto ServiceName::operator=(const ServiceName& rhs) -> ServiceName& {
    if (this != &rhs) {
        drop();

        const auto* ptr = iox2_cast_service_name_ptr(rhs.m_handle);
        size_t len = 0;
        const auto* chars = iox2_service_name_as_chars(ptr, &len);
        IOX2_ASSERT(iox2_service_name_new(nullptr, chars, len, &m_handle) == IOX2_OK,
                    "ServiceName shall always contain a valid value.");
    }

    return *this;
}

void ServiceName::drop() noexcept {
    if (m_handle != nullptr) {
        iox2_service_name_drop(m_handle);
        m_handle = nullptr;
    }
}

auto ServiceName::create(const char* value) -> iox2::bb::Expected<ServiceName, bb::SemanticStringError> {
    return ServiceName::create_impl(value, strnlen(value, IOX2_SERVICE_NAME_LENGTH + 1));
}

auto ServiceName::create_impl(const char* value, const size_t value_len)
    -> bb::Expected<ServiceName, bb::SemanticStringError> {
    iox2_service_name_h handle {};
    if (value_len > IOX2_SERVICE_NAME_LENGTH) {
        return bb::err(bb::SemanticStringError::ExceedsMaximumLength);
    }

    const auto ret_val = iox2_service_name_new(nullptr, value, value_len, &handle);

    if (ret_val == IOX2_OK) {
        return ServiceName { handle };
    }

    return bb::err(bb::into<bb::SemanticStringError>(ret_val));
}

auto ServiceName::to_string() const -> iox2::bb::StaticString<IOX2_SERVICE_NAME_LENGTH> {
    return as_view().to_string();
}

auto ServiceName::as_view() const -> ServiceNameView {
    return ServiceNameView(iox2_cast_service_name_ptr(m_handle));
}
} // namespace iox2
