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

#ifndef IOX2_SERVICE_EVENT_BUILDER_HPP
#define IOX2_SERVICE_EVENT_BUILDER_HPP

#include "iox/builder_addendum.hpp"
#include "iox/expected.hpp"
#include "iox2/attribute_specifier.hpp"
#include "iox2/attribute_verifier.hpp"
#include "iox2/event_id.hpp"
#include "iox2/internal/iceoryx2.hpp"
#include "iox2/port_factory_event.hpp"
#include "iox2/service_builder_event_error.hpp"
#include "iox2/service_type.hpp"

#include <cstdint>

namespace iox2 {
/// Builder to create new [`MessagingPattern::Event`] based [`Service`]s
template <ServiceType S>
class ServiceBuilderEvent {
  public:
    /// If the [`Service`] is created it defines how many [`Node`]s shall
    /// be able to open it in parallel. If an existing [`Service`] is opened it defines how many
    /// [`Node`]s must be at least supported.
#ifdef DOXYGEN_MACRO_FIX
    auto max_nodes(const uint64_t value) -> decltype(auto);
#else
    IOX_BUILDER_OPTIONAL(uint64_t, max_nodes);
#endif

    /// If the [`Service`] is created it set the greatest supported [`NodeId`] value
    /// If an existing [`Service`] is opened it defines the value size the [`NodeId`]
    /// must at least support.
#ifdef DOXYGEN_MACRO_FIX
    auto event_id_max_value(const uint64_t value) -> decltype(auto);
#else
    IOX_BUILDER_OPTIONAL(uint64_t, event_id_max_value);
#endif

    /// If the [`Service`] is created it defines how many [`Notifier`] shall
    /// be supported at most. If an existing [`Service`] is opened it defines how many
    /// [`Notifier`] must be at least supported.
#ifdef DOXYGEN_MACRO_FIX
    auto max_notifiers(const uint64_t value) -> decltype(auto);
#else
    IOX_BUILDER_OPTIONAL(uint64_t, max_notifiers);
#endif

    /// If the [`Service`] is created it defines how many [`Listener`] shall
    /// be supported at most. If an existing [`Service`] is opened it defines how many
    /// [`Listener`] must be at least supported.
#ifdef DOXYGEN_MACRO_FIX
    auto max_listeners(const uint64_t value) -> decltype(auto);
#else
    IOX_BUILDER_OPTIONAL(uint64_t, max_listeners);
#endif

  public:
    /// If the [`Service`] is created it defines the event that shall be emitted by every
    /// [`Notifier`] before it is dropped. If [`None`] is
    /// provided a [`Notifier`] will not emit an event.
    auto notifier_dropped_event(EventId event_id) && -> ServiceBuilderEvent&&;

    /// If the [`Service`] is created it defines the event that shall be emitted by every newly
    /// created [`Notifier`].
    auto notifier_created_event(EventId event_id) && -> ServiceBuilderEvent&&;

    /// If the [`Service`] is created it defines the event that shall be emitted when a
    /// [`Notifier`] is identified as dead. If [`None`] is
    /// provided no event will be emitted.
    auto notifier_dead_event(EventId event_id) && -> ServiceBuilderEvent&&;

    /// Enables the deadline property of the service. There must be a notification emitted by any
    /// [`Notifier`] after at least the provided `deadline`.
    auto deadline(iox::units::Duration deadline) && -> ServiceBuilderEvent&&;

    /// If the [`Service`] is created it disables sending an event when a notifier was dropped.
    auto disable_notifier_dropped_event() && -> ServiceBuilderEvent&&;

    /// If the [`Service`] is created it disables sending an event when a new notifier was created.
    auto disable_notifier_created_event() && -> ServiceBuilderEvent&&;

    /// If the [`Service`] is created it disables sending an event when a notifier was identified
    /// as dead.
    auto disable_notifier_dead_event() && -> ServiceBuilderEvent&&;

    /// Disables the deadline property of the service. [`Notifier`]
    /// can signal notifications at any rate.
    auto disable_deadline() && -> ServiceBuilderEvent&&;

    /// If the [`Service`] exists, it will be opened otherwise a new [`Service`] will be
    /// created.
    auto open_or_create() && -> iox::expected<PortFactoryEvent<S>, EventOpenOrCreateError>;

    /// If the [`Service`] exists, it will be opened otherwise a new [`Service`] will be
    /// created. It defines a set of attributes. If the [`Service`] already exists all attribute
    /// requirements must be satisfied otherwise the open process will fail. If the [`Service`]
    /// does not exist the required attributes will be defined in the [`Service`].
    auto open_or_create_with_attributes(
        const AttributeVerifier& required_attributes) && -> iox::expected<PortFactoryEvent<S>, EventOpenOrCreateError>;

    /// Opens an existing [`Service`].
    auto open() && -> iox::expected<PortFactoryEvent<S>, EventOpenError>;

    /// Opens an existing [`Service`] with attribute requirements. If the defined attribute
    /// requirements are not satisfied the open process will fail.
    auto open_with_attributes(
        const AttributeVerifier& required_attributes) && -> iox::expected<PortFactoryEvent<S>, EventOpenError>;

    /// Creates a new [`Service`].
    auto create() && -> iox::expected<PortFactoryEvent<S>, EventCreateError>;

    /// Creates a new [`Service`] with a set of attributes.
    auto create_with_attributes(
        const AttributeSpecifier& attributes) && -> iox::expected<PortFactoryEvent<S>, EventCreateError>;

  private:
    template <ServiceType>
    friend class ServiceBuilder;

    explicit ServiceBuilderEvent(iox2_service_builder_h handle);

    void set_parameters();

    iox2_service_builder_event_h m_handle = nullptr;

    iox::optional<EventId> m_notifier_dead_event;
    iox::optional<EventId> m_notifier_created_event;
    iox::optional<EventId> m_notifier_dropped_event;
    iox::optional<iox::units::Duration> m_deadline;
    bool m_verify_notifier_dead_event = false;
    bool m_verify_notifier_created_event = false;
    bool m_verify_notifier_dropped_event = false;
    bool m_verify_deadline = false;
};
} // namespace iox2

#endif
