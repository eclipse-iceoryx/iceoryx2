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
#include "iox2/internal/iceoryx2.hpp"
#include "iox2/listener_error.hpp"
#include "iox2/service_type.hpp"
#include "iox2/unique_port_id.hpp"

namespace iox2 {
/// Represents the receiving endpoint of an event based communication.
template <ServiceType>
class Listener {
  public:
    Listener(Listener&&) noexcept;
    auto operator=(Listener&&) noexcept -> Listener&;
    ~Listener();

    Listener(const Listener&) = delete;
    auto operator=(const Listener&) -> Listener& = delete;

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
    auto timed_wait_all(const iox::function<void(EventId)>& callback,
                        const iox::units::Duration& timeout) -> iox::expected<void, ListenerWaitError>;

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
    auto
    timed_wait_one(const iox::units::Duration& timeout) -> iox::expected<iox::optional<EventId>, ListenerWaitError>;

    /// Blocking wait for a new [`EventId`].
    /// Sporadic wakeups can occur and if no [`EventId`] was notified it returns [`None`].
    /// On error it returns [`ListenerWaitError`] is returned which describes the error
    /// in detail.
    auto blocking_wait_one() -> iox::expected<iox::optional<EventId>, ListenerWaitError>;

  private:
    class Unsafe {
      public:
        Unsafe(const Unsafe&) noexcept = delete;
        Unsafe(Unsafe&&) noexcept = delete;
        Unsafe& operator=(const Unsafe&) noexcept = delete;
        Unsafe& operator=(Unsafe&&) noexcept = delete;

        ~Unsafe() noexcept = default;

        /// Returns a file descriptor to the underlying Listener. The file descriptor must not be closed and only be
        /// used for event multiplexing!
        auto file_descriptor() && -> iox::optional<FileDescriptor>;

      private:
        friend class Listener;
        Unsafe(Listener& listener);

      private:
        const Listener& m_self;
    };

  public:
    /// This function gives access to unsafe operations on the Listener. The returned object must never be stored but
    /// just used to access the unsafe functions.
    auto unsafe() -> Unsafe;

  private:
    template <ServiceType>
    friend class PortFactoryListener;
    template <ServiceType>
    friend class WaitSet;

    explicit Listener(iox2_listener_h handle);
    void drop();

    iox2_listener_h m_handle;
};
} // namespace iox2

#endif
