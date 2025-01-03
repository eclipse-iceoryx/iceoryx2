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
#include "iox2/internal/callback_context.hpp"

namespace iox2 {
template <ServiceType S>
Listener<S>::Listener(iox2_listener_h handle)
    : m_handle { handle } {
}

template <ServiceType S>
Listener<S>::Listener(Listener&& rhs) noexcept {
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
auto Listener<S>::file_descriptor() const -> FileDescriptorView {
    return FileDescriptorView(iox2_listener_get_file_descriptor(&m_handle));
}

template <ServiceType S>
auto Listener<S>::id() const -> UniqueListenerId {
    iox2_unique_listener_id_h id_handle = nullptr;

    iox2_listener_id(&m_handle, nullptr, &id_handle);
    return UniqueListenerId { id_handle };
}

template <ServiceType S>
auto Listener<S>::deadline() const -> iox::optional<iox::units::Duration> {
    uint64_t seconds = 0;
    uint32_t nanoseconds = 0;

    if (iox2_listener_deadline(&m_handle, &seconds, &nanoseconds)) {
        return { iox::units::Duration::fromSeconds(seconds) + iox::units::Duration::fromNanoseconds(nanoseconds) };
    }

    return iox::nullopt;
}

void wait_callback(const iox2_event_id_t* event_id, iox2_callback_context context) {
    auto* callback = internal::ctx_cast<iox::function<void(EventId)>>(context);
    callback->value()(EventId(*event_id));
}

template <ServiceType S>
auto Listener<S>::try_wait_all(const iox::function<void(EventId)>& callback) -> iox::expected<void, ListenerWaitError> {
    auto ctx = internal::ctx(callback);

    auto result = iox2_listener_try_wait_all(&m_handle, wait_callback, static_cast<void*>(&ctx));
    if (result == IOX2_OK) {
        return iox::ok();
    }

    return iox::err(iox::into<ListenerWaitError>(result));
}

template <ServiceType S>
auto Listener<S>::timed_wait_all(const iox::function<void(EventId)>& callback, const iox::units::Duration& timeout)
    -> iox::expected<void, ListenerWaitError> {
    auto ctx = internal::ctx(callback);
    auto timeout_timespec = timeout.timespec();

    auto result = iox2_listener_timed_wait_all(
        &m_handle, wait_callback, static_cast<void*>(&ctx), timeout_timespec.tv_sec, timeout_timespec.tv_nsec);
    if (result == IOX2_OK) {
        return iox::ok();
    }

    return iox::err(iox::into<ListenerWaitError>(result));
}

template <ServiceType S>
auto Listener<S>::blocking_wait_all(const iox::function<void(EventId)>& callback)
    -> iox::expected<void, ListenerWaitError> {
    auto ctx = internal::ctx(callback);

    auto result = iox2_listener_blocking_wait_all(&m_handle, wait_callback, static_cast<void*>(&ctx));
    if (result == IOX2_OK) {
        return iox::ok();
    }

    return iox::err(iox::into<ListenerWaitError>(result));
}

template <ServiceType S>
auto Listener<S>::try_wait_one() -> iox::expected<iox::optional<EventId>, ListenerWaitError> {
    iox2_event_id_t event_id {};
    bool has_received_one { false };

    auto result = iox2_listener_try_wait_one(&m_handle, &event_id, &has_received_one);

    if (result == IOX2_OK) {
        if (has_received_one) {
            return iox::ok(iox::optional<EventId>(EventId { event_id }));
        }

        return iox::ok(iox::optional<EventId>());
    }

    return iox::err(iox::into<ListenerWaitError>(result));
}

template <ServiceType S>
auto Listener<S>::timed_wait_one(const iox::units::Duration& timeout)
    -> iox::expected<iox::optional<EventId>, ListenerWaitError> {
    iox2_event_id_t event_id {};
    bool has_received_one { false };

    auto timespec_timeout = timeout.timespec();
    auto result = iox2_listener_timed_wait_one(
        &m_handle, &event_id, &has_received_one, timespec_timeout.tv_sec, timespec_timeout.tv_nsec);

    if (result == IOX2_OK) {
        if (has_received_one) {
            return iox::ok(iox::optional<EventId>(EventId { event_id }));
        }

        return iox::ok(iox::optional<EventId>());
    }

    return iox::err(iox::into<ListenerWaitError>(result));
}

template <ServiceType S>
auto Listener<S>::blocking_wait_one() -> iox::expected<iox::optional<EventId>, ListenerWaitError> {
    iox2_event_id_t event_id {};
    bool has_received_one { false };

    auto result = iox2_listener_blocking_wait_one(&m_handle, &event_id, &has_received_one);

    if (result == IOX2_OK) {
        if (has_received_one) {
            return iox::ok(iox::optional<EventId>(EventId { event_id }));
        }

        return iox::ok(iox::optional<EventId>());
    }

    return iox::err(iox::into<ListenerWaitError>(result));
}

template class Listener<ServiceType::Ipc>;
template class Listener<ServiceType::Local>;
} // namespace iox2
