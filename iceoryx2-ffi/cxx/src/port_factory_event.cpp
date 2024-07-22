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
auto PortFactoryEvent<S>::name() const -> const ServiceName& {
    IOX_TODO();
}

template <ServiceType S>
auto PortFactoryEvent<S>::uuid() const -> iox::string<SERVICE_ID_LENGTH> {
    IOX_TODO();
}

template <ServiceType S>
auto PortFactoryEvent<S>::attributes() const -> const AttributeSet& {
    IOX_TODO();
}

template <ServiceType S>
auto PortFactoryEvent<S>::static_config() const -> const StaticConfigEvent& {
    IOX_TODO();
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
    IOX_TODO();
}

template <ServiceType S>
auto PortFactoryEvent<S>::notifier_builder() const -> PortFactoryNotifier<S> {
    IOX_TODO();
}

template class PortFactoryEvent<ServiceType::Ipc>;
template class PortFactoryEvent<ServiceType::Local>;
} // namespace iox2
