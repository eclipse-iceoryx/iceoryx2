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

#ifndef IOX2_PORTFACTORY_EVENT_HPP
#define IOX2_PORTFACTORY_EVENT_HPP

#include "attribute_set.hpp"
#include "callback_progression.hpp"
#include "dynamic_config_event.hpp"
#include "iox/expected.hpp"
#include "iox/function.hpp"
#include "iox/string.hpp"
#include "iox2/iceoryx2_settings.hpp"
#include "node_failure_enums.hpp"
#include "node_state.hpp"
#include "port_factory_listener.hpp"
#include "port_factory_notifier.hpp"
#include "service_name.hpp"
#include "service_type.hpp"
#include "static_config_event.hpp"

namespace iox2 {
/// Represents the port factory of a [`Service`] with [`MessagingPattern::Event`].
template <ServiceType S>
class PortFactoryEvent {
  public:
    PortFactoryEvent(PortFactoryEvent&&) noexcept;
    auto operator=(PortFactoryEvent&&) noexcept -> PortFactoryEvent&;
    ~PortFactoryEvent();

    PortFactoryEvent(const PortFactoryEvent&) = delete;
    auto operator=(const PortFactoryEvent&) -> PortFactoryEvent& = delete;

    /// Returns the [`ServiceName`] of the service
    auto name() const -> ServiceNameView;

    /// Returns the uuid of the [`Service`]
    auto uuid() const -> iox::string<SERVICE_ID_LENGTH>;

    /// Returns the attributes defined in the [`Service`]
    auto attributes() const -> const AttributeSet&;

    /// Returns the StaticConfig of the [`Service`].
    /// Contains all settings that never change during the lifetime of the service.
    auto static_config() const -> StaticConfigEvent;

    /// Returns the DynamicConfig of the [`Service`].
    /// Contains all dynamic settings, like the current participants etc..
    auto dynamic_config() const -> const DynamicConfigEvent&;

    /// Iterates over all [`Node`]s of the [`Service`]
    /// and calls for every [`Node`] the provided callback. If an error occurs
    /// while acquiring the [`Node`]s corresponding [`NodeState`] the error is
    /// forwarded to the callback as input argument.
    auto nodes(const iox::function<CallbackProgression(NodeState<S>)>& callback) const
        -> iox::expected<void, NodeListFailure>;

    /// Returns a [`PortFactoryListener`] to create a new [`Listener`] port
    auto listener_builder() const -> PortFactoryListener<S>;

    /// Returns a [`PortFactoryNotifier`] to create a new [`Notifier`] port
    auto notifier_builder() const -> PortFactoryNotifier<S>;

  private:
    template <ServiceType>
    friend class ServiceBuilderEvent;

    explicit PortFactoryEvent(iox2_port_factory_event_h handle);
    void drop() noexcept;

    iox2_port_factory_event_h m_handle;
};
} // namespace iox2

#endif
