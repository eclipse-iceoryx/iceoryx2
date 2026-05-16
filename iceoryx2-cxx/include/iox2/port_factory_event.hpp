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

#include "iox2/attribute_set.hpp"
#include "iox2/bb/expected.hpp"
#include "iox2/bb/static_function.hpp"
#include "iox2/callback_progression.hpp"
#include "iox2/cleanup_state.hpp"
#include "iox2/dynamic_config_event.hpp"
#include "iox2/internal/iceoryx2.hpp"
#include "iox2/node_failure_enums.hpp"
#include "iox2/node_state.hpp"
#include "iox2/port_factory_listener.hpp"
#include "iox2/port_factory_notifier.hpp"
#include "iox2/service_hash.hpp"
#include "iox2/service_name.hpp"
#include "iox2/service_type.hpp"
#include "iox2/static_config_event.hpp"

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

    /// Returns the [`ServiceHash`] of the [`Service`]
    auto service_hash() const -> ServiceHash;

    /// Returns the attributes defined in the [`Service`]
    auto attributes() const -> AttributeSetView;

    /// Returns the StaticConfig of the [`Service`].
    /// Contains all settings that never change during the lifetime of the service.
    auto static_config() const -> StaticConfigEvent;

    /// Returns the DynamicConfig of the [`Service`].
    /// Contains all dynamic settings, like the current participants etc..
    auto dynamic_config() const -> DynamicConfigEvent;

    /// Iterates over all [`Node`]s of the [`Service`]
    /// and calls for every [`Node`] the provided callback. If an error occurs
    /// while acquiring the [`Node`]s corresponding [`NodeState`] the error is
    /// forwarded to the callback as input argument.
    auto nodes(const iox2::bb::StaticFunction<CallbackProgression(NodeState<S>)>& callback) const
        -> bb::Expected<void, NodeListFailure>;

    /// Returns a [`PortFactoryListener`] to create a new [`Listener`] port
    auto listener_builder() const -> PortFactoryListener<S>;

    /// Returns a [`PortFactoryNotifier`] to create a new [`Notifier`] port
    auto notifier_builder() const -> PortFactoryNotifier<S>;

    /// Removes the stale system resources of all dead [`Node`]s connected to this service.
    ///
    /// If a [`Node`] cannot be cleaned up since the process has insufficient permissions or it
    /// is currently being cleaned up by another process then the [`Node`] is skipped.
    auto try_cleanup_dead_nodes() const -> CleanupState;

    /// Removes the stale system resources of all dead [`Node`]s connected to this service.
    ///
    /// If a [`Node`] cannot be cleaned up since the process has insufficient permissions then the
    /// [`Node`] is skipped. If it is currently being cleaned up by another process then the
    /// cleaner will wait until the timeout as either passed or the cleaned was finished.
    ///
    /// The timeout is applied to every individual dead [`Node`] the function needs to wait on.
    auto blocking_cleanup_dead_nodes(iox2::bb::Duration timeout) const -> CleanupState;

  private:
    template <ServiceType>
    friend class ServiceBuilderEvent;

    explicit PortFactoryEvent(iox2_port_factory_event_h handle);
    void drop() noexcept;

    iox2_port_factory_event_h m_handle = nullptr;
};
} // namespace iox2

#endif
