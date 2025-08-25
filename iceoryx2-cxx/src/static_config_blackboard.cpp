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

#include "iox2/static_config_blackboard.hpp"

namespace iox2 {
StaticConfigBlackboard::StaticConfigBlackboard(iox2_static_config_blackboard_t value)
    : m_value { value } {
}

auto StaticConfigBlackboard::max_nodes() const -> size_t {
    return m_value.max_nodes;
}

auto StaticConfigBlackboard::max_readers() const -> size_t {
    return m_value.max_readers;
}

auto StaticConfigBlackboard::type_details() const -> TypeDetail {
    return TypeDetail(m_value.type_details);
}

} // namespace iox2
