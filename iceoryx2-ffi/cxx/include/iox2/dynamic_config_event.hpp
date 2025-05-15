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

#ifndef IOX2_DYNAMIC_CONFIG_EVENT_HPP
#define IOX2_DYNAMIC_CONFIG_EVENT_HPP

#include "iox/function.hpp"
#include "iox2/internal/iceoryx2.hpp"
#include "iox2/listener_details.hpp"
#include "iox2/notifier_details.hpp"

#include <cstdint>

namespace iox2 {
/// The dynamic configuration of an [`MessagingPattern::Event`]
/// based service. Contains dynamic parameters like the connected endpoints etc..
class DynamicConfigEvent {
  public:
    DynamicConfigEvent(const DynamicConfigEvent&) = delete;
    DynamicConfigEvent(DynamicConfigEvent&&) = delete;
    auto operator=(const DynamicConfigEvent&) -> DynamicConfigEvent& = delete;
    auto operator=(DynamicConfigEvent&&) -> DynamicConfigEvent& = delete;
    ~DynamicConfigEvent() = default;

    /// Returns how many [`Listener`] ports are currently connected.
    auto number_of_listeners() const -> uint64_t;

    /// Returns how many [`Notifier`] ports are currently connected.
    auto number_of_notifiers() const -> uint64_t;

    /// Iterates over all [`Notifier`]s and calls the
    /// callback with the corresponding [`NotifierDetailsView`].
    /// The callback shall return [`CallbackProgression::Continue`] when the iteration shall
    /// continue otherwise [`CallbackProgression::Stop`].
    void list_notifiers(const iox::function<CallbackProgression(NotifierDetailsView)>& callback) const;

    /// Iterates over all [`Listener`]s and calls the
    /// callback with the corresponding [`ListenerDetailsView`].
    /// The callback shall return [`CallbackProgression::Continue`] when the iteration shall
    /// continue otherwise [`CallbackProgression::Stop`].
    void list_listeners(const iox::function<CallbackProgression(ListenerDetailsView)>& callback) const;

  private:
    template <ServiceType>
    friend class PortFactoryEvent;

    explicit DynamicConfigEvent(iox2_port_factory_event_h handle);

    iox2_port_factory_event_h m_handle = nullptr;
};
} // namespace iox2

#endif
