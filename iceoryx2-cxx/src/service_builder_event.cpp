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

#include "iox2/service_builder_event.hpp"

namespace iox2 {
template <ServiceType S>
ServiceBuilderEvent<S>::ServiceBuilderEvent(iox2_service_builder_h handle)
    : m_handle { iox2_service_builder_event(handle) } {
}

template <ServiceType S>
// NOLINTNEXTLINE(readability-function-size) the size cannot easily be reduced due to the amount of builder parameter
void ServiceBuilderEvent<S>::set_parameters() {
    if (m_max_notifiers.has_value()) {
        iox2_service_builder_event_set_max_notifiers(&m_handle, m_max_notifiers.value());
    }
    if (m_max_listeners.has_value()) {
        iox2_service_builder_event_set_max_listeners(&m_handle, m_max_listeners.value());
    }

    if (m_verify_notifier_created_event) {
        if (m_notifier_created_event.has_value()) {
            iox2_service_builder_event_set_notifier_created_event(&m_handle,
                                                                  m_notifier_created_event.value().as_value());
        } else {
            iox2_service_builder_event_disable_notifier_created_event(&m_handle);
        }
    }

    if (m_verify_notifier_dropped_event) {
        if (m_notifier_dropped_event.has_value()) {
            iox2_service_builder_event_set_notifier_dropped_event(&m_handle,
                                                                  m_notifier_dropped_event.value().as_value());
        } else {
            iox2_service_builder_event_disable_notifier_dropped_event(&m_handle);
        }
    }

    if (m_verify_notifier_dead_event) {
        if (m_notifier_dead_event.has_value()) {
            iox2_service_builder_event_set_notifier_dead_event(&m_handle, m_notifier_dead_event.value().as_value());
        } else {
            iox2_service_builder_event_disable_notifier_dead_event(&m_handle);
        }
    }

    if (m_verify_deadline) {
        if (m_deadline.has_value()) {
            auto& deadline = m_deadline.value();
            iox2_service_builder_event_set_deadline(&m_handle, deadline.as_secs(), deadline.subsec_nanos());
        } else {
            iox2_service_builder_event_disable_deadline(&m_handle);
        }
    }

    if (m_max_nodes.has_value()) {
        iox2_service_builder_event_set_max_nodes(&m_handle, m_max_nodes.value());
    }
    if (m_event_id_max_value.has_value()) {
        iox2_service_builder_event_set_event_id_max_value(&m_handle, m_event_id_max_value.value());
    }
}

template <ServiceType S>
auto ServiceBuilderEvent<S>::notifier_dropped_event(EventId event_id) && -> ServiceBuilderEvent&& {
    m_notifier_dropped_event.emplace(event_id);
    m_verify_notifier_dropped_event = true;
    return std::move(*this);
}

template <ServiceType S>
auto ServiceBuilderEvent<S>::notifier_created_event(EventId event_id) && -> ServiceBuilderEvent&& {
    m_notifier_created_event.emplace(event_id);
    m_verify_notifier_created_event = true;
    return std::move(*this);
}

template <ServiceType S>
auto ServiceBuilderEvent<S>::notifier_dead_event(EventId event_id) && -> ServiceBuilderEvent&& {
    m_notifier_dead_event.emplace(event_id);
    m_verify_notifier_dead_event = true;
    return std::move(*this);
}


template <ServiceType S>
auto ServiceBuilderEvent<S>::deadline(iox2::bb::Duration deadline) && -> ServiceBuilderEvent&& {
    m_deadline.emplace(deadline);
    m_verify_deadline = true;
    return std::move(*this);
}

template <ServiceType S>
auto ServiceBuilderEvent<S>::disable_notifier_dropped_event() && -> ServiceBuilderEvent&& {
    m_notifier_dropped_event.reset();
    m_verify_notifier_dropped_event = true;
    return std::move(*this);
}

template <ServiceType S>
auto ServiceBuilderEvent<S>::disable_notifier_created_event() && -> ServiceBuilderEvent&& {
    m_notifier_created_event.reset();
    m_verify_notifier_created_event = true;
    return std::move(*this);
}

template <ServiceType S>
auto ServiceBuilderEvent<S>::disable_notifier_dead_event() && -> ServiceBuilderEvent&& {
    m_notifier_dead_event.reset();
    m_verify_notifier_dead_event = true;
    return std::move(*this);
}

template <ServiceType S>
auto ServiceBuilderEvent<S>::disable_deadline() && -> ServiceBuilderEvent&& {
    m_deadline.reset();
    m_verify_deadline = true;
    return std::move(*this);
}

template <ServiceType S>
auto ServiceBuilderEvent<S>::open_or_create() && -> iox2::legacy::expected<PortFactoryEvent<S>,
                                                                           EventOpenOrCreateError> {
    set_parameters();
    iox2_port_factory_event_h event_handle {};
    auto result = iox2_service_builder_event_open_or_create(m_handle, nullptr, &event_handle);

    if (result == IOX2_OK) {
        return iox2::legacy::ok(PortFactoryEvent<S>(event_handle));
    }

    return iox2::legacy::err(iox2::bb::into<EventOpenOrCreateError>(result));
}

template <ServiceType S>
auto ServiceBuilderEvent<S>::open() && -> iox2::legacy::expected<PortFactoryEvent<S>, EventOpenError> {
    set_parameters();

    iox2_port_factory_event_h event_handle {};
    auto result = iox2_service_builder_event_open(m_handle, nullptr, &event_handle);

    if (result == IOX2_OK) {
        return iox2::legacy::ok(PortFactoryEvent<S>(event_handle));
    }

    return iox2::legacy::err(iox2::bb::into<EventOpenError>(result));
}

template <ServiceType S>
auto ServiceBuilderEvent<S>::create() && -> iox2::legacy::expected<PortFactoryEvent<S>, EventCreateError> {
    set_parameters();

    iox2_port_factory_event_h event_handle {};
    auto result = iox2_service_builder_event_create(m_handle, nullptr, &event_handle);

    if (result == IOX2_OK) {
        return iox2::legacy::ok(PortFactoryEvent<S>(event_handle));
    }

    return iox2::legacy::err(iox2::bb::into<EventCreateError>(result));
}

template <ServiceType S>
auto ServiceBuilderEvent<S>::open_or_create_with_attributes(
    const AttributeVerifier&
        required_attributes) && -> iox2::legacy::expected<PortFactoryEvent<S>, EventOpenOrCreateError> {
    set_parameters();

    iox2_port_factory_event_h event_handle {};
    auto result = iox2_service_builder_event_open_or_create_with_attributes(
        m_handle, &required_attributes.m_handle, nullptr, &event_handle);

    if (result == IOX2_OK) {
        return iox2::legacy::ok(PortFactoryEvent<S>(event_handle));
    }

    return iox2::legacy::err(iox2::bb::into<EventOpenOrCreateError>(result));
}

template <ServiceType S>
auto ServiceBuilderEvent<S>::open_with_attributes(
    const AttributeVerifier& required_attributes) && -> iox2::legacy::expected<PortFactoryEvent<S>, EventOpenError> {
    set_parameters();

    iox2_port_factory_event_h event_handle {};
    auto result = iox2_service_builder_event_open_with_attributes(
        m_handle, &required_attributes.m_handle, nullptr, &event_handle);

    if (result == IOX2_OK) {
        return iox2::legacy::ok(PortFactoryEvent<S>(event_handle));
    }

    return iox2::legacy::err(iox2::bb::into<EventOpenError>(result));
}

template <ServiceType S>
auto ServiceBuilderEvent<S>::create_with_attributes(
    const AttributeSpecifier& attributes) && -> iox2::legacy::expected<PortFactoryEvent<S>, EventCreateError> {
    set_parameters();

    iox2_port_factory_event_h event_handle {};
    auto result =
        iox2_service_builder_event_create_with_attributes(m_handle, &attributes.m_handle, nullptr, &event_handle);

    if (result == IOX2_OK) {
        return iox2::legacy::ok(PortFactoryEvent<S>(event_handle));
    }

    return iox2::legacy::err(iox2::bb::into<EventCreateError>(result));
}

template class ServiceBuilderEvent<ServiceType::Ipc>;
template class ServiceBuilderEvent<ServiceType::Local>;
} // namespace iox2
