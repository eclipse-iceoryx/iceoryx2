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
#include "iox2/internal/iceoryx2.hpp"
#include "iox2/port_factory_event.hpp"
#include "iox2/service_builder_event_error.hpp"
#include "iox2/service_type.hpp"

#include <cstdint>

namespace iox2 {
/// Builder to create new [`MessagingPattern::Event`] based [`Service`]s
template <ServiceType S>
class ServiceBuilderEvent {
    /// If the [`Service`] is created it defines how many [`Node`]s shall
    /// be able to open it in parallel. If an existing [`Service`] is opened it defines how many
    /// [`Node`]s must be at least supported.
    IOX_BUILDER_OPTIONAL(uint64_t, max_nodes);

    /// If the [`Service`] is created it set the greatest supported [`NodeId`] value
    /// If an existing [`Service`] is opened it defines the value size the [`NodeId`]
    /// must at least support.
    IOX_BUILDER_OPTIONAL(uint64_t, event_id_max_value);

    /// If the [`Service`] is created it defines how many [`Notifier`] shall
    /// be supported at most. If an existing [`Service`] is opened it defines how many
    /// [`Notifier`] must be at least supported.
    IOX_BUILDER_OPTIONAL(uint64_t, max_notifiers);

    /// If the [`Service`] is created it defines how many [`Listener`] shall
    /// be supported at most. If an existing [`Service`] is opened it defines how many
    /// [`Listener`] must be at least supported.
    IOX_BUILDER_OPTIONAL(uint64_t, max_listeners);

  public:
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
};
} // namespace iox2

#endif
