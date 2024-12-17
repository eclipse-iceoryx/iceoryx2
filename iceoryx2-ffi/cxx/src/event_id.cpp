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

namespace iox2 {
EventId::EventId(const size_t value)
    : m_value { value } {
}

EventId::EventId(iox2_event_id_t value)
    : m_value { value } {
}

auto EventId::as_value() const -> size_t {
    return m_value.value;
}

auto operator<<(std::ostream& stream, const EventId& value) -> std::ostream& {
    stream << "EventId { m_value: " << value.as_value() << " }";
    return stream;
}

auto operator==(const EventId& lhs, const EventId& rhs) -> bool {
    return lhs.as_value() == rhs.as_value();
}

auto operator!=(const EventId& lhs, const EventId& rhs) -> bool {
    return lhs.as_value() != rhs.as_value();
}

auto operator<(const EventId& lhs, const EventId& rhs) -> bool {
    return lhs.as_value() < rhs.as_value();
}

auto operator<=(const EventId& lhs, const EventId& rhs) -> bool {
    return lhs.as_value() <= rhs.as_value();
}

auto operator>(const EventId& lhs, const EventId& rhs) -> bool {
    return lhs.as_value() > rhs.as_value();
}

auto operator>=(const EventId& lhs, const EventId& rhs) -> bool {
    return lhs.as_value() >= rhs.as_value();
}
} // namespace iox2
