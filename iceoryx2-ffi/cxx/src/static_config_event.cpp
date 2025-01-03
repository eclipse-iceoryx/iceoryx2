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

#include "iox2/static_config_event.hpp"

namespace iox2 {
StaticConfigEvent::StaticConfigEvent(iox2_static_config_event_t value)
    : m_value { value } {
}

auto StaticConfigEvent::max_nodes() const -> size_t {
    return m_value.max_nodes;
}

auto StaticConfigEvent::max_notifiers() const -> size_t {
    return m_value.max_notifiers;
}

auto StaticConfigEvent::max_listeners() const -> size_t {
    return m_value.max_listeners;
}

auto StaticConfigEvent::event_id_max_value() const -> size_t {
    return m_value.event_id_max_value;
}

auto StaticConfigEvent::notifier_created_event() const -> iox::optional<EventId> {
    if (!m_value.has_notifier_created_event) {
        return iox::nullopt;
    }

    return { EventId(m_value.notifier_created_event) };
}

auto StaticConfigEvent::notifier_dropped_event() const -> iox::optional<EventId> {
    if (!m_value.has_notifier_dropped_event) {
        return iox::nullopt;
    }

    return { EventId(m_value.notifier_dropped_event) };
}

auto StaticConfigEvent::notifier_dead_event() const -> iox::optional<EventId> {
    if (!m_value.has_notifier_dead_event) {
        return iox::nullopt;
    }

    return { EventId(m_value.notifier_dead_event) };
}

auto StaticConfigEvent::deadline() const -> iox::optional<iox::units::Duration> {
    if (!m_value.has_deadline) {
        return iox::nullopt;
    }

    return { iox::units::Duration::fromSeconds(m_value.deadline_seconds)
             + iox::units::Duration::fromNanoseconds(m_value.deadline_nanoseconds) };
}

} // namespace iox2
