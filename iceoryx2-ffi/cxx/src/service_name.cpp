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
#include "iox/assertions.hpp"
#include "iox/into.hpp"

#include <cstring>

namespace iox2 {
ServiceName::ServiceName(iox2_service_name_h handle)
    : m_handle { handle } {
}

ServiceName::~ServiceName() {
    drop();
}

ServiceName::ServiceName(ServiceName&& rhs) noexcept
    : m_handle { std::move(rhs.m_handle) } {
    rhs.m_handle = nullptr;
}

auto ServiceName::operator=(ServiceName&& rhs) noexcept -> ServiceName& {
    if (this != &rhs) {
        drop();
        m_handle = std::move(rhs.m_handle);
        rhs.m_handle = nullptr;
    }

    return *this;
}

ServiceName::ServiceName(const ServiceName& rhs)
    : m_handle { nullptr } {
    auto value = rhs.to_string();
    IOX_ASSERT(iox2_service_name_new(nullptr, value.c_str(), value.size(), &m_handle) == IOX2_OK,
               "ServiceName shall always contain a valid value.");
}

auto ServiceName::operator=(const ServiceName& rhs) -> ServiceName& {
    if (this != &rhs) {
        drop();

        auto value = rhs.to_string();
        IOX_ASSERT(iox2_service_name_new(nullptr, value.c_str(), value.size(), &m_handle) == IOX2_OK,
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

auto ServiceName::create(const char* value) -> iox::expected<ServiceName, SemanticStringError> {
    iox2_service_name_h handle {};
    const auto value_len = strnlen(value, SERVICE_NAME_LENGTH + 1);
    if (value_len == SERVICE_NAME_LENGTH + 1) {
        return iox::err(SemanticStringError::ExceedsMaximumLength);
    }

    const auto ret_val = iox2_service_name_new(nullptr, value, value_len, &handle);

    if (ret_val == IOX2_OK) {
        return iox::ok(ServiceName { handle });
    }

    return iox::err(iox::into<SemanticStringError>(ret_val));
}

auto ServiceName::to_string() const -> iox::string<SERVICE_NAME_LENGTH> {
    const auto* ptr = iox2_cast_service_name_ptr(m_handle);
    size_t len = 0;
    const auto* c_ptr = iox2_service_name_as_c_str(ptr, &len);
    return { iox::TruncateToCapacity, c_ptr, len };
}

} // namespace iox2
