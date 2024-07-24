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
} // namespace iox2
