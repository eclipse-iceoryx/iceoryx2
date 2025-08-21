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

#include "iox2/notifier.hpp"

namespace iox2 {
template <ServiceType S>
Notifier<S>::Notifier(iox2_notifier_h handle)
    : m_handle { handle } {
}

template <ServiceType S>
Notifier<S>::Notifier(Notifier&& rhs) noexcept {
    *this = std::move(rhs);
}

template <ServiceType S>
auto Notifier<S>::operator=(Notifier&& rhs) noexcept -> Notifier& {
    if (this != &rhs) {
        drop();
        m_handle = std::move(rhs.m_handle);
        rhs.m_handle = nullptr;
    }

    return *this;
}

template <ServiceType S>
Notifier<S>::~Notifier() {
    drop();
}

template <ServiceType S>
auto Notifier<S>::id() const -> UniqueNotifierId {
    iox2_unique_notifier_id_h id_handle = nullptr;

    iox2_notifier_id(&m_handle, nullptr, &id_handle);
    return UniqueNotifierId { id_handle };
}

template <ServiceType S>
auto Notifier<S>::notify() const -> iox::expected<size_t, NotifierNotifyError> {
    size_t number_of_notified_listeners = 0;
    auto result = iox2_notifier_notify(&m_handle, &number_of_notified_listeners);

    if (result == IOX2_OK) {
        return iox::ok(number_of_notified_listeners);
    }

    return iox::err(iox::into<NotifierNotifyError>(result));
}

template <ServiceType S>
auto Notifier<S>::notify_with_custom_event_id(EventId event_id) const -> iox::expected<size_t, NotifierNotifyError> {
    size_t number_of_notified_listeners = 0;
    auto result =
        iox2_notifier_notify_with_custom_event_id(&m_handle, &event_id.m_value, &number_of_notified_listeners);

    if (result == IOX2_OK) {
        return iox::ok(number_of_notified_listeners);
    }

    return iox::err(iox::into<NotifierNotifyError>(result));
}

template <ServiceType S>
auto Notifier<S>::deadline() const -> iox::optional<iox::units::Duration> {
    uint64_t seconds = 0;
    uint32_t nanoseconds = 0;

    if (iox2_notifier_deadline(&m_handle, &seconds, &nanoseconds)) {
        return { iox::units::Duration::fromSeconds(seconds) + iox::units::Duration::fromNanoseconds(nanoseconds) };
    }

    return iox::nullopt;
}

template <ServiceType S>
void Notifier<S>::drop() {
    if (m_handle != nullptr) {
        iox2_notifier_drop(m_handle);
        m_handle = nullptr;
    }
}

template class Notifier<ServiceType::Ipc>;
template class Notifier<ServiceType::Local>;
} // namespace iox2
