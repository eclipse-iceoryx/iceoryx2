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

#ifndef IOX2_NOTIFIER_HPP
#define IOX2_NOTIFIER_HPP

#include "iox/duration.hpp"
#include "iox/expected.hpp"
#include "iox2/event_id.hpp"
#include "iox2/internal/iceoryx2.hpp"
#include "iox2/notifier_error.hpp"
#include "iox2/service_type.hpp"
#include "iox2/unique_port_id.hpp"

namespace iox2 {
/// Represents the sending endpoint of an event based communication.
template <ServiceType S>
class Notifier {
  public:
    Notifier(Notifier&&) noexcept;
    auto operator=(Notifier&&) noexcept -> Notifier&;
    ~Notifier();

    Notifier(const Notifier&) = delete;
    auto operator=(const Notifier&) -> Notifier& = delete;

    /// Returns the [`UniqueNotifierId`] of the [`Notifier`]
    auto id() const -> UniqueNotifierId;

    /// Notifies all [`Listener`] connected to the service with the default
    /// event id provided on creation.
    /// Returns on success the number of [`Listener`]s that were notified otherwise it returns
    /// [`NotifierNotifyError`].
    auto notify() const -> iox::expected<size_t, NotifierNotifyError>;

    /// Notifies all [`Listener`] connected to the service with a custom [`EventId`].
    /// Returns on success the number of [`Listener`]s that were notified otherwise it returns
    /// [`NotifierNotifyError`].
    auto notify_with_custom_event_id(EventId event_id) const -> iox::expected<size_t, NotifierNotifyError>;

    /// Returns the deadline of the corresponding [`Service`].
    auto deadline() const -> iox::optional<iox::units::Duration>;

  private:
    template <ServiceType>
    friend class PortFactoryNotifier;

    explicit Notifier(iox2_notifier_h handle);
    void drop();

    iox2_notifier_h m_handle = nullptr;
};
} // namespace iox2

#endif
