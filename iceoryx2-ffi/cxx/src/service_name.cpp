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
#include "iox/into.hpp"

#include <cstring>

namespace iox2 {
ServiceName::ServiceName(iox2_service_name_h handle)
    : m_handle { handle } {
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

    return iox::err(iox::from<iox2_semantic_string_error_e, SemanticStringError>(
        static_cast<iox2_semantic_string_error_e>(ret_val)));
}

auto ServiceName::to_string() const -> iox::string<SERVICE_NAME_LENGTH> {
    const auto* ptr = iox2_cast_service_name_ptr(m_handle);
    size_t len = 0;
    const auto* c_ptr = iox2_service_name_as_c_str(ptr, &len);
    return { iox::TruncateToCapacity, c_ptr, len };
}

} // namespace iox2
