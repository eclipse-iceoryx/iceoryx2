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

#include "iox2/event_id.hpp"
#include "iox/assertions_addendum.hpp"

namespace iox2 {
EventId::EventId(const uint64_t low, const uint64_t high)
    : m_value { high, low } {
}

auto EventId::as_value_high() const -> uint64_t {
    return m_value.high;
}

auto EventId::as_value_low() const -> uint64_t {
    return m_value.low;
}

auto operator<<(std::ostream& stream, const EventId& value) -> std::ostream& {
    std::cout << "EventId { m_value.high: " << value.m_value.high << ", m_value.low: " << value.m_value.low << "}";
    return stream;
}
} // namespace iox2
