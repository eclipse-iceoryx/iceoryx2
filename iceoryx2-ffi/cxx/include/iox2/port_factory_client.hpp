// Copyright (c) 2025 Contributors to the Eclipse Foundation
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

#ifndef IOX2_PORTFACTORY_CLIENT_HPP
#define IOX2_PORTFACTORY_CLIENT_HPP

#include "iox/builder_addendum.hpp"
#include "iox/expected.hpp"
#include "iox2/client.hpp"
#include "iox2/client_error.hpp"
#include "iox2/service_type.hpp"
#include "iox2/unable_to_deliver_strategy.hpp"

namespace iox2 {
/// Factory to create a new [`Client`] port/endpoint for
/// [`MessagingPattern::RequestResponse`](crate::service::messaging_pattern::MessagingPattern::RequestResponse)
/// based communication.
template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
class PortFactoryClient {
    /// Sets the [`UnableToDeliverStrategy`] which defines how the [`Client`] shall behave
    /// when a [`Server`](crate::port::server::Server) cannot receive a
    /// [`RequestMut`](crate::request_mut::RequestMut) since
    /// its internal buffer is full.
    IOX_BUILDER_OPTIONAL(UnableToDeliverStrategy, unable_to_deliver_strategy);

  public:
    PortFactoryClient(const PortFactoryClient&) = delete;
    PortFactoryClient(PortFactoryClient&&) = default;
    auto operator=(const PortFactoryClient&) -> PortFactoryClient& = delete;
    auto operator=(PortFactoryClient&&) -> PortFactoryClient& = default;
    ~PortFactoryClient() = default;

    /// Creates a new [`Client`] or returns a [`ClientCreateError`] on failure.
    auto create() && -> iox::expected<Client<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>,
                                      ClientCreateError>;

  private:
    template <ServiceType, typename, typename, typename, typename>
    friend class PortFactoryRequestResponse;

    explicit PortFactoryClient(iox2_port_factory_client_builder_h handle);

    iox2_port_factory_client_builder_h m_handle = nullptr;
};

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline auto
PortFactoryClient<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::create() && -> iox::
    expected<Client<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>, ClientCreateError> {
    m_unable_to_deliver_strategy.and_then([&](auto value) {
        iox2_port_factory_client_builder_unable_to_deliver_strategy(
            &m_handle, static_cast<iox2_unable_to_deliver_strategy_e>(iox::into<int>(value)));
    });

    iox2_client_h client_handle {};
    auto result = iox2_port_factory_client_builder_create(m_handle, nullptr, &client_handle);

    if (result == IOX2_OK) {
        return iox::ok(Client<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>(client_handle));
    }

    return iox::err(iox::into<ClientCreateError>(result));
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline PortFactoryClient<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::PortFactoryClient(
    iox2_port_factory_client_builder_h handle)
    : m_handle { handle } {
}
} // namespace iox2
#endif
