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

#ifndef IOX2_PORTFACTORY_REQUEST_RESPONSE_HPP
#define IOX2_PORTFACTORY_REQUEST_RESPONSE_HPP

#include "iox/assertions_addendum.hpp"
#include "iox/expected.hpp"
#include "iox/function.hpp"
#include "iox2/attribute_set.hpp"
#include "iox2/callback_progression.hpp"
#include "iox2/dynamic_config_request_response.hpp"
#include "iox2/node_failure_enums.hpp"
#include "iox2/node_state.hpp"
#include "iox2/port_factory_client.hpp"
#include "iox2/port_factory_server.hpp"
#include "iox2/service_id.hpp"
#include "iox2/service_name.hpp"
#include "iox2/service_type.hpp"
#include "iox2/static_config_request_response.hpp"

namespace iox2 {
/// The factory for
/// [`MessagingPattern::RequestResponse`](crate::service::messaging_pattern::MessagingPattern::RequestResponse).
/// It can acquire dynamic and static service informations and create
/// [`crate::port::client::Client`]
/// or [`crate::port::server::Server`] ports.
template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
class PortFactoryRequestResponse {
  public:
    PortFactoryRequestResponse(PortFactoryRequestResponse&& rhs) noexcept;
    auto operator=(PortFactoryRequestResponse&& rhs) noexcept -> PortFactoryRequestResponse&;
    ~PortFactoryRequestResponse();

    PortFactoryRequestResponse(const PortFactoryRequestResponse&) = delete;
    auto operator=(const PortFactoryRequestResponse&) -> PortFactoryRequestResponse& = delete;

    /// Returns the [`ServiceName`] of the service
    auto name() const -> const ServiceName&;

    /// Returns the [`ServiceId`] of the [`Service`]
    auto service_id() const -> const ServiceId&;

    /// Returns the attributes defined in the [`Service`]
    auto attributes() const -> AttributeSetView;

    /// Returns the StaticConfig of the [`Service`].
    /// Contains all settings that never change during the lifetime of the service.
    auto static_config() const -> StaticConfigRequestResponse;

    /// Returns the DynamicConfig of the [`Service`].
    /// Contains all dynamic settings, like the current participants etc..
    auto dynamic_config() const -> const DynamicConfigRequestResponse&;

    /// Iterates over all [`Node`]s of the [`Service`]
    /// and calls for every [`Node`] the provided callback. If an error occurs
    /// while acquiring the [`Node`]s corresponding [`NodeState`] the error is
    /// forwarded to the callback as input argument.
    auto nodes(const iox::function<CallbackProgression(NodeState<Service>)>& callback) const
        -> iox::expected<void, NodeListFailure>;

    /// Returns a [`PortFactoryClient`] to create a new
    /// [`crate::port::client::Client`] port.
    auto client_builder() const
        -> PortFactoryClient<Service, RequestPayload, RequestPayload, ResponsePayload, ResponseHeader>;

    /// Returns a [`PortFactoryServer`] to create a new
    /// [`crate::port::server::Server`] port.
    auto server_builder() const
        -> PortFactoryServer<Service, RequestPayload, RequestHeader, ResponsePayload, ResponsePayload>;

  private:
    template <ServiceType, typename, typename, typename, typename>
    friend class ServiceBuilderRequestResponse;

    explicit PortFactoryRequestResponse();
    void drop();
};

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline PortFactoryRequestResponse<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::
    PortFactoryRequestResponse(PortFactoryRequestResponse&& rhs) noexcept {
    *this = std::move(rhs);
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline auto
PortFactoryRequestResponse<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::operator=(
    PortFactoryRequestResponse&& rhs) noexcept -> PortFactoryRequestResponse& {
    IOX_TODO();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline PortFactoryRequestResponse<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::
    ~PortFactoryRequestResponse() {
    drop();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline auto
PortFactoryRequestResponse<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::name() const
    -> const ServiceName& {
    IOX_TODO();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline auto
PortFactoryRequestResponse<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::service_id() const
    -> const ServiceId& {
    IOX_TODO();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline auto
PortFactoryRequestResponse<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::attributes() const
    -> AttributeSetView {
    IOX_TODO();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline auto
PortFactoryRequestResponse<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::static_config()
    const -> StaticConfigRequestResponse {
    IOX_TODO();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline auto
PortFactoryRequestResponse<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::dynamic_config()
    const -> const DynamicConfigRequestResponse& {
    IOX_TODO();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline auto PortFactoryRequestResponse<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::nodes(
    const iox::function<CallbackProgression(NodeState<Service>)>& callback) const
    -> iox::expected<void, NodeListFailure> {
    IOX_TODO();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline auto
PortFactoryRequestResponse<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::client_builder()
    const -> PortFactoryClient<Service, RequestPayload, RequestPayload, ResponsePayload, ResponseHeader> {
    IOX_TODO();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline auto
PortFactoryRequestResponse<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::server_builder()
    const -> PortFactoryServer<Service, RequestPayload, RequestHeader, ResponsePayload, ResponsePayload> {
    IOX_TODO();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline PortFactoryRequestResponse<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::
    PortFactoryRequestResponse() {
    IOX_TODO();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline void
PortFactoryRequestResponse<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::drop() {
    IOX_TODO();
}
} // namespace iox2

#endif

