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

#include "iox2/port_factory_event.hpp"
#include "iox/assertions_addendum.hpp"
#include "iox2/iceoryx2.h"

namespace iox2 {
template <ServiceType S>
PortFactoryEvent<S>::PortFactoryEvent(iox2_port_factory_event_h handle)
    : m_handle { handle } {
}

template <ServiceType S>
PortFactoryEvent<S>::~PortFactoryEvent() {
    drop();
}

template <ServiceType S>
PortFactoryEvent<S>::PortFactoryEvent(PortFactoryEvent&& rhs) noexcept
    : m_handle { nullptr } {
    *this = std::move(rhs);
}

template <ServiceType S>
auto PortFactoryEvent<S>::operator=(PortFactoryEvent&& rhs) noexcept -> PortFactoryEvent& {
    if (this != &rhs) {
        drop();
        m_handle = std::move(rhs.m_handle);
        rhs.m_handle = nullptr;
    }

    return *this;
}

template <ServiceType S>
void PortFactoryEvent<S>::drop() noexcept {
    if (m_handle != nullptr) {
        iox2_port_factory_event_drop(m_handle);
        m_handle = nullptr;
    }
}

template <ServiceType S>
auto PortFactoryEvent<S>::name() const -> ServiceNameView {
    auto* ref_handle = iox2_cast_port_factory_event_ref_h(m_handle);
    const auto* service_name_ptr = iox2_port_factory_event_service_name(ref_handle);
    return ServiceNameView(service_name_ptr);
}

template <ServiceType S>
auto PortFactoryEvent<S>::uuid() const -> iox::string<IOX2_SERVICE_ID_LENGTH> {
    IOX_TODO();
}

template <ServiceType S>
auto PortFactoryEvent<S>::attributes() const -> const AttributeSet& {
    IOX_TODO();
}

template <ServiceType S>
auto PortFactoryEvent<S>::static_config() const -> StaticConfigEvent {
    auto* ref_handle = iox2_cast_port_factory_event_ref_h(m_handle);
    iox2_static_config_event_t static_config {};
    iox2_port_factory_event_static_config(ref_handle, &static_config);

    return StaticConfigEvent(static_config);
}

template <ServiceType S>
auto PortFactoryEvent<S>::dynamic_config() const -> const DynamicConfigEvent& {
    IOX_TODO();
}

template <ServiceType S>
auto PortFactoryEvent<S>::nodes(const iox::function<CallbackProgression(NodeState<S>)>& callback) const
    -> iox::expected<void, NodeListFailure> {
    IOX_TODO();
}

template <ServiceType S>
auto PortFactoryEvent<S>::listener_builder() const -> PortFactoryListener<S> {
    auto* ref_handle = iox2_cast_port_factory_event_ref_h(m_handle);
    return PortFactoryListener<S> { iox2_port_factory_event_listener_builder(ref_handle, nullptr) };
}

template <ServiceType S>
auto PortFactoryEvent<S>::notifier_builder() const -> PortFactoryNotifier<S> {
    auto* ref_handle = iox2_cast_port_factory_event_ref_h(m_handle);
    return PortFactoryNotifier<S> { iox2_port_factory_event_notifier_builder(ref_handle, nullptr) };
}

template class PortFactoryEvent<ServiceType::Ipc>;
template class PortFactoryEvent<ServiceType::Local>;
} // namespace iox2
