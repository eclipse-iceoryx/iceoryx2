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

#include "iox2/listener.hpp"
#include "iox/assertions_addendum.hpp"

namespace iox2 {
template <ServiceType S>
Listener<S>::Listener(iox2_listener_h handle)
    : m_handle { handle } {
}

template <ServiceType S>
Listener<S>::Listener(Listener&& rhs) noexcept
    : m_handle { nullptr } {
    *this = std::move(rhs);
}

template <ServiceType S>
auto Listener<S>::operator=(Listener&& rhs) noexcept -> Listener& {
    if (this != &rhs) {
        drop();
        m_handle = std::move(rhs.m_handle);
        rhs.m_handle = nullptr;
    }

    return *this;
}

template <ServiceType S>
Listener<S>::~Listener() {
    drop();
}

template <ServiceType S>
void Listener<S>::drop() {
    if (m_handle != nullptr) {
        iox2_listener_drop(m_handle);
        m_handle = nullptr;
    }
}

template <ServiceType S>
auto Listener<S>::id() const -> UniqueListenerId {
    IOX_TODO();
}

template <ServiceType S>
auto Listener<S>::try_wait_all(const iox::function<void(EventId)>& callback) -> iox::expected<void, ListenerWaitError> {
    IOX_TODO();
}

template <ServiceType S>
auto Listener<S>::timed_wait_all(const iox::function<void(EventId)>& callback,
                                 const iox::units::Duration& timeout) -> iox::expected<void, ListenerWaitError> {
    IOX_TODO();
}

template <ServiceType S>
auto Listener<S>::blocking_wait_all(const iox::function<void(EventId)>& callback)
    -> iox::expected<void, ListenerWaitError> {
    IOX_TODO();
}

template <ServiceType S>
auto Listener<S>::try_wait_one() -> iox::expected<iox::optional<EventId>, ListenerWaitError> {
    IOX_TODO();
}

template <ServiceType S>
auto Listener<S>::timed_wait_one(const iox::units::Duration& timeout)
    -> iox::expected<iox::optional<EventId>, ListenerWaitError> {
    IOX_TODO();
}

template <ServiceType S>
auto Listener<S>::blocking_wait_one() -> iox::expected<iox::optional<EventId>, ListenerWaitError> {
    IOX_TODO();
}

template class Listener<ServiceType::Ipc>;
template class Listener<ServiceType::Local>;
} // namespace iox2
