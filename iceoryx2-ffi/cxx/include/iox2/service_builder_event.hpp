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
template <ServiceType S>
class ServiceBuilderEvent {
    IOX_BUILDER_OPTIONAL(uint64_t, max_nodes);
    IOX_BUILDER_OPTIONAL(uint64_t, event_id_max_value);
    IOX_BUILDER_OPTIONAL(uint64_t, max_notifiers);
    IOX_BUILDER_OPTIONAL(uint64_t, max_listeners);

  public:
    auto open_or_create() && -> iox::expected<PortFactoryEvent<S>, EventOpenOrCreateError>;

    auto open_or_create_with_attributes(
        const AttributeVerifier& required_attributes) && -> iox::expected<PortFactoryEvent<S>, EventOpenOrCreateError>;

    auto open() && -> iox::expected<PortFactoryEvent<S>, EventOpenError>;

    auto open_with_attributes(
        const AttributeVerifier& required_attributes) && -> iox::expected<PortFactoryEvent<S>, EventOpenError>;

    auto create() && -> iox::expected<PortFactoryEvent<S>, EventCreateError>;

    auto create_with_attributes(
        const AttributeSpecifier& attributes) && -> iox::expected<PortFactoryEvent<S>, EventCreateError>;

  private:
    template <ServiceType>
    friend class ServiceBuilder;

    explicit ServiceBuilderEvent(iox2_service_builder_h handle);

    void set_parameters();

    iox2_service_builder_event_h m_handle;
};
} // namespace iox2

#endif
