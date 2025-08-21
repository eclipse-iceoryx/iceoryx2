// Copyright (c) 2025 Contributors to the Eclipse Foundation
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

#include "iox2/service_id.hpp"

namespace iox2 {
auto ServiceId::max_number_of_characters() -> uint64_t {
    return IOX2_SERVICE_ID_LENGTH;
}

auto ServiceId::c_str() const -> const char* {
    return m_value.c_str();
}

ServiceId::ServiceId(const iox::string<IOX2_SERVICE_ID_LENGTH>& value)
    : m_value { value } {
}
} // namespace iox2
