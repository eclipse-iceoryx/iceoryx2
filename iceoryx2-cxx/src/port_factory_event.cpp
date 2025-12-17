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
#include "iox2/bb/expected.hpp"
#include "iox2/iceoryx2.h"
#include "iox2/internal/callback_context.hpp"
#include "iox2/legacy/uninitialized_array.hpp"

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
PortFactoryEvent<S>::PortFactoryEvent(PortFactoryEvent&& rhs) noexcept {
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
    const auto* service_name_ptr = iox2_port_factory_event_service_name(&m_handle);
    return ServiceNameView(service_name_ptr);
}

template <ServiceType S>
auto PortFactoryEvent<S>::service_id() const -> ServiceId {
    iox2::legacy::UninitializedArray<char, IOX2_SERVICE_ID_LENGTH> buffer;
    iox2_port_factory_event_service_id(&m_handle, &buffer[0], IOX2_SERVICE_ID_LENGTH);

    return ServiceId(iox2::bb::StaticString<IOX2_SERVICE_ID_LENGTH>::from_utf8_null_terminated_unchecked_truncated(
        &buffer[0], IOX2_SERVICE_ID_LENGTH));
}

template <ServiceType S>
auto PortFactoryEvent<S>::attributes() const -> AttributeSetView {
    return AttributeSetView(iox2_port_factory_event_attributes(&m_handle));
}

template <ServiceType S>
auto PortFactoryEvent<S>::static_config() const -> StaticConfigEvent {
    iox2_static_config_event_t static_config {};
    iox2_port_factory_event_static_config(&m_handle, &static_config);

    return StaticConfigEvent(static_config);
}

template <ServiceType S>
auto PortFactoryEvent<S>::dynamic_config() const -> DynamicConfigEvent {
    return DynamicConfigEvent(m_handle);
}

template <ServiceType S>
auto PortFactoryEvent<S>::nodes(const iox2::bb::StaticFunction<CallbackProgression(NodeState<S>)>& callback) const
    -> bb::Expected<void, NodeListFailure> {
    auto ctx = internal::ctx(callback);

    const auto ret_val = iox2_port_factory_event_nodes(&m_handle, internal::list_callback<S>, static_cast<void*>(&ctx));

    if (ret_val == IOX2_OK) {
        return {};
    }

    return bb::err(bb::into<NodeListFailure>(ret_val));
}

template <ServiceType S>
auto PortFactoryEvent<S>::listener_builder() const -> PortFactoryListener<S> {
    return PortFactoryListener<S> { iox2_port_factory_event_listener_builder(&m_handle, nullptr) };
}

template <ServiceType S>
auto PortFactoryEvent<S>::notifier_builder() const -> PortFactoryNotifier<S> {
    return PortFactoryNotifier<S> { iox2_port_factory_event_notifier_builder(&m_handle, nullptr) };
}

template class PortFactoryEvent<ServiceType::Ipc>;
template class PortFactoryEvent<ServiceType::Local>;
} // namespace iox2
