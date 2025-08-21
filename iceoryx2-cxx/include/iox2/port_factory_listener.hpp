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

#ifndef IOX2_PORTFACTORY_LISTENER_HPP
#define IOX2_PORTFACTORY_LISTENER_HPP

#include "iox/expected.hpp"
#include "iox2/internal/iceoryx2.hpp"
#include "iox2/listener.hpp"
#include "iox2/service_type.hpp"

namespace iox2 {
/// Factory to create a new [`Listener`] port/endpoint for [`MessagingPattern::Event`] based
/// communication.
template <ServiceType S>
class PortFactoryListener {
  public:
    PortFactoryListener(PortFactoryListener&&) noexcept = default;
    auto operator=(PortFactoryListener&&) noexcept -> PortFactoryListener& = default;
    ~PortFactoryListener() = default;

    PortFactoryListener(const PortFactoryListener&) = delete;
    auto operator=(const PortFactoryListener&) -> PortFactoryListener& = delete;

    /// Creates the [`Listener`] port or returns a [`ListenerCreateError`] on failure.
    auto create() && -> iox::expected<Listener<S>, ListenerCreateError>;

  private:
    template <ServiceType>
    friend class PortFactoryEvent;

    explicit PortFactoryListener(iox2_port_factory_listener_builder_h handle);

    iox2_port_factory_listener_builder_h m_handle = nullptr;
};

template <ServiceType S>
inline PortFactoryListener<S>::PortFactoryListener(iox2_port_factory_listener_builder_h handle)
    : m_handle { handle } {
}

template <ServiceType S>
inline auto PortFactoryListener<S>::create() && -> iox::expected<Listener<S>, ListenerCreateError> {
    iox2_listener_h listener_handle { nullptr };
    auto result = iox2_port_factory_listener_builder_create(m_handle, nullptr, &listener_handle);

    if (result == IOX2_OK) {
        return iox::ok(Listener<S> { listener_handle });
    }

    return iox::err(iox::into<ListenerCreateError>(result));
}
} // namespace iox2

#endif
