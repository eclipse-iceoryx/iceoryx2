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

#include "iox2/event_activation.hpp"

namespace iox2 {
EventActivation::EventActivation(iox2_event_id_t value, uint64_t count)
    : m_id { EventId(value) }
    , m_count { count } {
}

auto EventActivation::id() const -> EventId {
    return m_id;
}

auto EventActivation::count() const -> uint64_t {
    return m_count;
}

auto operator<<(std::ostream& stream, const EventActivation& value) -> std::ostream& {
    stream << "EventActivation { m_id: " << value.id() << ", m_count: " << value.count() << " }";
    return stream;
}
} // namespace iox2
