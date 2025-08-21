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

#ifndef IOX2_LISTENER_HPP
#define IOX2_LISTENER_HPP

#include "iox/duration.hpp"
#include "iox/expected.hpp"
#include "iox/function.hpp"
#include "iox/optional.hpp"
#include "iox2/event_id.hpp"
#include "iox2/file_descriptor.hpp"
#include "iox2/internal/callback_context.hpp"
#include "iox2/internal/iceoryx2.hpp"
#include "iox2/listener_error.hpp"
#include "iox2/service_type.hpp"
#include "iox2/unique_port_id.hpp"

namespace iox2 {
/// Represents the receiving endpoint of an event based communication.
template <ServiceType>
class Listener {
  public:
    Listener(Listener&& rhs) noexcept;
    auto operator=(Listener&& rhs) noexcept -> Listener&;
    ~Listener();

    Listener(const Listener&) = delete;
    auto operator=(const Listener&) -> Listener& = delete;

    /// Returns a [`FileDescriptorView`] to the underlying [`FileDescriptor`] of the [`Listener`].
    auto file_descriptor() const -> FileDescriptorView;

    /// Returns the [`UniqueListenerId`] of the [`Listener`]
    auto id() const -> UniqueListenerId;

    /// Non-blocking wait for new [`EventId`]s. Collects either all [`EventId`]s that were received
    /// until the call of [`Listener::try_wait_all()`] or a reasonable batch that represent the
    /// currently available [`EventId`]s in buffer.
    /// For every received [`EventId`] the provided callback is called with the [`EventId`] as
    /// input argument.
    auto try_wait_all(const iox::function<void(EventId)>& callback) -> iox::expected<void, ListenerWaitError>;

    /// Blocking wait for new [`EventId`]s until the provided timeout has passed. Collects either
    /// all [`EventId`]s that were received
    /// until the call of [`Listener::timed_wait_all()`] or a reasonable batch that represent the
    /// currently available [`EventId`]s in buffer.
    /// For every received [`EventId`] the provided callback is called with the [`EventId`] as
    /// input argument.
    auto timed_wait_all(const iox::function<void(EventId)>& callback, const iox::units::Duration& timeout)
        -> iox::expected<void, ListenerWaitError>;

    /// Blocking wait for new [`EventId`]s. Collects either
    /// all [`EventId`]s that were received
    /// until the call of [`Listener::timed_wait_all()`] or a reasonable batch that represent the
    /// currently available [`EventId`]s in buffer.
    /// For every received [`EventId`] the provided callback is called with the [`EventId`] as
    /// input argument.
    auto blocking_wait_all(const iox::function<void(EventId)>& callback) -> iox::expected<void, ListenerWaitError>;

    /// Non-blocking wait for a new [`EventId`]. If no [`EventId`] was notified it returns [`None`].
    /// On error it returns [`ListenerWaitError`] is returned which describes the error
    /// in detail.
    auto try_wait_one() -> iox::expected<iox::optional<EventId>, ListenerWaitError>;

    /// Blocking wait for a new [`EventId`] until either an [`EventId`] was received or the timeout
    /// has passed. If no [`EventId`] was notified it returns [`None`].
    /// On error it returns [`ListenerWaitError`] is returned which describes the error
    /// in detail.
    auto timed_wait_one(const iox::units::Duration& timeout)
        -> iox::expected<iox::optional<EventId>, ListenerWaitError>;

    /// Blocking wait for a new [`EventId`].
    /// Sporadic wakeups can occur and if no [`EventId`] was notified it returns [`None`].
    /// On error it returns [`ListenerWaitError`] is returned which describes the error
    /// in detail.
    auto blocking_wait_one() -> iox::expected<iox::optional<EventId>, ListenerWaitError>;

    /// Returns the deadline of the corresponding [`Service`].
    auto deadline() const -> iox::optional<iox::units::Duration>;

  private:
    template <ServiceType>
    friend class PortFactoryListener;
    template <ServiceType>
    friend class WaitSet;

    explicit Listener(iox2_listener_h handle);
    void drop();

    iox2_listener_h m_handle = nullptr;
};

template <ServiceType S>
inline Listener<S>::Listener(iox2_listener_h handle)
    : m_handle { handle } {
}

template <ServiceType S>
inline Listener<S>::Listener(Listener&& rhs) noexcept {
    *this = std::move(rhs);
}

template <ServiceType S>
inline auto Listener<S>::operator=(Listener&& rhs) noexcept -> Listener& {
    if (this != &rhs) {
        drop();
        m_handle = std::move(rhs.m_handle);
        rhs.m_handle = nullptr;
    }

    return *this;
}

template <ServiceType S>
inline Listener<S>::~Listener() {
    drop();
}

template <ServiceType S>
inline void Listener<S>::drop() {
    if (m_handle != nullptr) {
        iox2_listener_drop(m_handle);
        m_handle = nullptr;
    }
}

template <ServiceType>
struct IsListenerFdBased;

template <>
struct IsListenerFdBased<ServiceType::Ipc> {
    static constexpr const bool VALUE = IOX2_IS_IPC_LISTENER_FD_BASED;
};

template <>
struct IsListenerFdBased<ServiceType::Local> {
    static constexpr const bool VALUE = IOX2_IS_LOCAL_LISTENER_FD_BASED;
};

template <ServiceType S>
inline auto Listener<S>::file_descriptor() const -> FileDescriptorView {
    static_assert(IsListenerFdBased<S>::VALUE,
                  "This Listener variant is not based on a file descriptor. It cannot be attached to a WaitSet nor can "
                  "the underlying file descriptor be acquired.");
    return FileDescriptorView(iox2_listener_get_file_descriptor(&m_handle));
}

template <ServiceType S>
inline auto Listener<S>::id() const -> UniqueListenerId {
    iox2_unique_listener_id_h id_handle = nullptr;

    iox2_listener_id(&m_handle, nullptr, &id_handle);
    return UniqueListenerId { id_handle };
}

template <ServiceType S>
inline auto Listener<S>::deadline() const -> iox::optional<iox::units::Duration> {
    uint64_t seconds = 0;
    uint32_t nanoseconds = 0;

    if (iox2_listener_deadline(&m_handle, &seconds, &nanoseconds)) {
        return { iox::units::Duration::fromSeconds(seconds) + iox::units::Duration::fromNanoseconds(nanoseconds) };
    }

    return iox::nullopt;
}

inline void wait_callback(const iox2_event_id_t* event_id, iox2_callback_context context) {
    auto* callback = internal::ctx_cast<iox::function<void(EventId)>>(context);
    callback->value()(EventId(*event_id));
}

template <ServiceType S>
inline auto Listener<S>::try_wait_all(const iox::function<void(EventId)>& callback)
    -> iox::expected<void, ListenerWaitError> {
    auto ctx = internal::ctx(callback);

    auto result = iox2_listener_try_wait_all(&m_handle, wait_callback, static_cast<void*>(&ctx));
    if (result == IOX2_OK) {
        return iox::ok();
    }

    return iox::err(iox::into<ListenerWaitError>(result));
}

template <ServiceType S>
inline auto Listener<S>::timed_wait_all(const iox::function<void(EventId)>& callback,
                                        const iox::units::Duration& timeout) -> iox::expected<void, ListenerWaitError> {
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
inline auto Listener<S>::blocking_wait_all(const iox::function<void(EventId)>& callback)
    -> iox::expected<void, ListenerWaitError> {
    auto ctx = internal::ctx(callback);

    auto result = iox2_listener_blocking_wait_all(&m_handle, wait_callback, static_cast<void*>(&ctx));
    if (result == IOX2_OK) {
        return iox::ok();
    }

    return iox::err(iox::into<ListenerWaitError>(result));
}

template <ServiceType S>
inline auto Listener<S>::try_wait_one() -> iox::expected<iox::optional<EventId>, ListenerWaitError> {
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
inline auto Listener<S>::timed_wait_one(const iox::units::Duration& timeout)
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
inline auto Listener<S>::blocking_wait_one() -> iox::expected<iox::optional<EventId>, ListenerWaitError> {
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
} // namespace iox2

#endif
