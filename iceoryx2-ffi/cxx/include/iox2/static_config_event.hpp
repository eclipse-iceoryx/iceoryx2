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

#ifndef IOX2_STATIC_CONFIG_EVENT_HPP
#define IOX2_STATIC_CONFIG_EVENT_HPP

#include "iox/duration.hpp"
#include "iox/optional.hpp"
#include "iox2/attribute_set.hpp"
#include "iox2/event_id.hpp"
#include "iox2/iceoryx2.h"
#include "iox2/internal/iceoryx2.hpp"

namespace iox2 {
/// The static configuration of an [`MessagingPattern::Event`]
/// based service. Contains all parameters that do not change during the lifetime of a
/// [`Service`].
class StaticConfigEvent {
  public:
    /// Returns the maximum supported amount of [`Node`]s that can open the
    /// [`Service`] in parallel.
    auto max_nodes() const -> size_t;

    /// Returns the maximum supported amount of [`Notifier`] ports
    auto max_notifiers() const -> size_t;

    /// Returns the maximum supported amount of [`Listener`] ports
    auto max_listeners() const -> size_t;

    /// Returns the largest [`EventId`] that is supported by the service
    auto event_id_max_value() const -> size_t;

    /// Returns the emitted [`EventId`] when a new notifier is created.
    auto notifier_created_event() const -> iox::optional<EventId>;

    /// Returns the emitted [`EventId`] when a notifier is dropped.
    auto notifier_dropped_event() const -> iox::optional<EventId>;

    /// Returns the emitted [`EventId`] when a notifier is identified as dead.
    auto notifier_dead_event() const -> iox::optional<EventId>;

    /// Returns the deadline of the service. If no new notification is signaled from any
    /// [`Notifier`] after the given deadline, it is rated
    /// as an error and all [`Listener`]s that are attached
    /// to a [`WaitSet`] are woken up and notified about the missed
    /// deadline.
    auto deadline() const -> iox::optional<iox::units::Duration>;

  private:
    template <ServiceType>
    friend class PortFactoryEvent;

    explicit StaticConfigEvent(iox2_static_config_event_t value);

    iox2_static_config_event_t m_value;
};
} // namespace iox2

#endif
