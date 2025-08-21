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

#include "iox2/dynamic_config_event.hpp"
#include "iox2/internal/callback_context.hpp"

namespace iox2 {
auto DynamicConfigEvent::number_of_listeners() const -> uint64_t {
    return iox2_port_factory_event_dynamic_config_number_of_listeners(&m_handle);
}

auto DynamicConfigEvent::number_of_notifiers() const -> uint64_t {
    return iox2_port_factory_event_dynamic_config_number_of_notifiers(&m_handle);
}

DynamicConfigEvent::DynamicConfigEvent(iox2_port_factory_event_h handle)
    : m_handle { handle } {
}

void DynamicConfigEvent::list_notifiers(const iox::function<CallbackProgression(NotifierDetailsView)>& callback) const {
    auto ctx = internal::ctx(callback);
    iox2_port_factory_event_dynamic_config_list_notifiers(
        &m_handle,
        internal::list_ports_callback<iox2_notifier_details_ptr, NotifierDetailsView>,
        static_cast<void*>(&ctx));
}

void DynamicConfigEvent::list_listeners(const iox::function<CallbackProgression(ListenerDetailsView)>& callback) const {
    auto ctx = internal::ctx(callback);
    iox2_port_factory_event_dynamic_config_list_listeners(
        &m_handle,
        internal::list_ports_callback<iox2_listener_details_ptr, ListenerDetailsView>,
        static_cast<void*>(&ctx));
}
} // namespace iox2
