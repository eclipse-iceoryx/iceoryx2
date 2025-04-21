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
void ServiceBuilderEvent<S>::set_parameters() {
    m_max_notifiers.and_then([&](auto value) { iox2_service_builder_event_set_max_notifiers(&m_handle, value); });
    m_max_listeners.and_then([&](auto value) { iox2_service_builder_event_set_max_listeners(&m_handle, value); });

    if (m_verify_notifier_created_event) {
        m_notifier_created_event
            .and_then(
                [&](auto value) { iox2_service_builder_event_set_notifier_created_event(&m_handle, value.as_value()); })
            .or_else([&]() { iox2_service_builder_event_disable_notifier_created_event(&m_handle); });
    }

    if (m_verify_notifier_dropped_event) {
        m_notifier_dropped_event
            .and_then(
                [&](auto value) { iox2_service_builder_event_set_notifier_dropped_event(&m_handle, value.as_value()); })
            .or_else([&]() { iox2_service_builder_event_disable_notifier_dropped_event(&m_handle); });
    }

    if (m_verify_notifier_dead_event) {
        m_notifier_dead_event
            .and_then(
                [&](auto value) { iox2_service_builder_event_set_notifier_dead_event(&m_handle, value.as_value()); })
            .or_else([&]() { iox2_service_builder_event_disable_notifier_dead_event(&m_handle); });
    }

    if (m_verify_deadline) {
        m_deadline
            .and_then([&](auto value) {
                iox2_service_builder_event_set_deadline(
                    &m_handle,
                    value.toSeconds(),
                    value.toNanoseconds() - (value.toSeconds() * iox::units::Duration::NANOSECS_PER_SEC));
            })
            .or_else([&]() { iox2_service_builder_event_disable_deadline(&m_handle); });
    }

    m_max_nodes.and_then([&](auto value) { iox2_service_builder_event_set_max_nodes(&m_handle, value); });
    m_event_id_max_value.and_then(
        [&](auto value) { iox2_service_builder_event_set_event_id_max_value(&m_handle, value); });
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
auto ServiceBuilderEvent<S>::deadline(iox::units::Duration deadline) && -> ServiceBuilderEvent&& {
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
auto ServiceBuilderEvent<S>::open_or_create() && -> iox::expected<PortFactoryEvent<S>, EventOpenOrCreateError> {
    set_parameters();
    iox2_port_factory_event_h event_handle {};
    auto result = iox2_service_builder_event_open_or_create(m_handle, nullptr, &event_handle);

    if (result == IOX2_OK) {
        return iox::ok(PortFactoryEvent<S>(event_handle));
    }

    return iox::err(iox::into<EventOpenOrCreateError>(result));
}

template <ServiceType S>
auto ServiceBuilderEvent<S>::open() && -> iox::expected<PortFactoryEvent<S>, EventOpenError> {
    set_parameters();

    iox2_port_factory_event_h event_handle {};
    auto result = iox2_service_builder_event_open(m_handle, nullptr, &event_handle);

    if (result == IOX2_OK) {
        return iox::ok(PortFactoryEvent<S>(event_handle));
    }

    return iox::err(iox::into<EventOpenError>(result));
}

template <ServiceType S>
auto ServiceBuilderEvent<S>::create() && -> iox::expected<PortFactoryEvent<S>, EventCreateError> {
    set_parameters();

    iox2_port_factory_event_h event_handle {};
    auto result = iox2_service_builder_event_create(m_handle, nullptr, &event_handle);

    if (result == IOX2_OK) {
        return iox::ok(PortFactoryEvent<S>(event_handle));
    }

    return iox::err(iox::into<EventCreateError>(result));
}

template <ServiceType S>
auto ServiceBuilderEvent<S>::open_or_create_with_attributes(
    const AttributeVerifier& required_attributes) && -> iox::expected<PortFactoryEvent<S>, EventOpenOrCreateError> {
    set_parameters();

    iox2_port_factory_event_h event_handle {};
    auto result = iox2_service_builder_event_open_or_create_with_attributes(
        m_handle, &required_attributes.m_handle, nullptr, &event_handle);

    if (result == IOX2_OK) {
        return iox::ok(PortFactoryEvent<S>(event_handle));
    }

    return iox::err(iox::into<EventOpenOrCreateError>(result));
}

template <ServiceType S>
auto ServiceBuilderEvent<S>::open_with_attributes(
    const AttributeVerifier& required_attributes) && -> iox::expected<PortFactoryEvent<S>, EventOpenError> {
    set_parameters();

    iox2_port_factory_event_h event_handle {};
    auto result = iox2_service_builder_event_open_with_attributes(
        m_handle, &required_attributes.m_handle, nullptr, &event_handle);

    if (result == IOX2_OK) {
        return iox::ok(PortFactoryEvent<S>(event_handle));
    }

    return iox::err(iox::into<EventOpenError>(result));
}

template <ServiceType S>
auto ServiceBuilderEvent<S>::create_with_attributes(
    const AttributeSpecifier& attributes) && -> iox::expected<PortFactoryEvent<S>, EventCreateError> {
    set_parameters();

    iox2_port_factory_event_h event_handle {};
    auto result =
        iox2_service_builder_event_create_with_attributes(m_handle, &attributes.m_handle, nullptr, &event_handle);

    if (result == IOX2_OK) {
        return iox::ok(PortFactoryEvent<S>(event_handle));
    }

    return iox::err(iox::into<EventCreateError>(result));
}

template class ServiceBuilderEvent<ServiceType::Ipc>;
template class ServiceBuilderEvent<ServiceType::Local>;
} // namespace iox2
