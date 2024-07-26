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

  private:
    template <ServiceType>
    friend class PortFactoryEvent;

    explicit StaticConfigEvent(iox2_static_config_event_t value);

    iox2_static_config_event_t m_value;
};
} // namespace iox2

#endif
