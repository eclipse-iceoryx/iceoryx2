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

#ifndef IOX2_SERVICE_BUILDER_REQUEST_RESPONSE_HPP
#define IOX2_SERVICE_BUILDER_REQUEST_RESPONSE_HPP

#include "iox/builder_addendum.hpp"
#include "iox/expected.hpp"
#include "iox2/attribute_verifier.hpp"
#include "iox2/port_factory_request_response.hpp"
#include "iox2/service_builder_request_response_error.hpp"

namespace iox2 {
template <typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader,
          ServiceType S>
class ServiceBuilderRequestResponse {
    /// If the [`Service`] is created, it defines the request [`Alignment`] of the payload for the
    /// service. If an existing [`Service`] is opened it requires the service to have at least the
    /// defined [`Alignment`]. If the Payload [`Alignment`] is greater than the provided
    /// [`Alignment`] then the Payload [`Alignment`] is used.
    IOX_BUILDER_OPTIONAL(uint64_t, request_payload_alignment);

    /// If the [`Service`] is created, it defines the response [`Alignment`] of the payload for the
    /// service. If an existing [`Service`] is opened it requires the service to have at least the
    /// defined [`Alignment`]. If the Payload [`Alignment`] is greater than the provided
    /// [`Alignment`] then the Payload [`Alignment`] is used.
    IOX_BUILDER_OPTIONAL(uint64_t, response_payload_alignment);

    /// If the [`Service`] is created, defines the overflow behavior of the service for requests.
    /// If an existing [`Service`] is opened it requires the service to have the defined overflow
    /// behavior.
    IOX_BUILDER_OPTIONAL(bool, enable_safe_overflow_for_requests);

    /// If the [`Service`] is created, defines the overflow behavior of the service for responses.
    /// If an existing [`Service`] is opened it requires the service to have the defined overflow
    /// behavior.
    IOX_BUILDER_OPTIONAL(bool, enable_safe_overflow_for_responses);

    /// Defines how many active requests a [`Server`](crate::port::server::Server) can hold in
    /// parallel per [`Client`](crate::port::client::Client). The objects are used to send answers to a request that was
    /// received earlier from a [`Client`](crate::port::client::Client)
    IOX_BUILDER_OPTIONAL(uint64_t, max_active_requests_per_client);

    /// If the [`Service`] is created it defines how many responses fit in the
    /// [`Clients`](crate::port::client::Client)s buffer. If an existing
    /// [`Service`] is opened it defines the minimum required.
    IOX_BUILDER_OPTIONAL(uint64_t, max_response_buffer_size);

    /// If the [`Service`] is created it defines how many [`crate::port::server::Server`]s shall
    /// be supported at most. If an existing [`Service`] is opened it defines how many
    /// [`crate::port::server::Server`]s must be at least supported.
    IOX_BUILDER_OPTIONAL(uint64_t, max_servers);

    /// If the [`Service`] is created it defines how many [`crate::port::client::Client`]s shall
    /// be supported at most. If an existing [`Service`] is opened it defines how many
    /// [`crate::port::client::Client`]s must be at least supported.
    IOX_BUILDER_OPTIONAL(uint64_t, max_clients);

    /// If the [`Service`] is created it defines how many [`Node`](crate::node::Node)s shall
    /// be able to open it in parallel. If an existing [`Service`] is opened it defines how many
    /// [`Node`](crate::node::Node)s must be at least supported.
    IOX_BUILDER_OPTIONAL(uint64_t, max_nodes);

    /// If the [`Service`] is created it defines how many [`Response`](crate::response::Response)s shall
    /// be able to be borrowed in parallel per [`PendingResponse`](crate::pending_response::PendingResponse). If an
    /// existing [`Service`] is opened it defines how many borrows must be at least supported.
    IOX_BUILDER_OPTIONAL(uint64_t, max_borrowed_responses_per_pending_response);

  public:
    /// Sets the request user header type of the [`Service`].
    template <typename NewRequestHeader>
    auto request_user_header() && -> ServiceBuilderRequestResponse<RequestPayload,
                                                                   NewRequestHeader,
                                                                   ResponsePayload,
                                                                   ResponseHeader,
                                                                   S>&&;

    /// Sets the response user header type of the [`Service`].
    template <typename NewResponseHeader>
    auto response_user_header() && -> ServiceBuilderRequestResponse<RequestPayload,
                                                                    RequestHeader,
                                                                    ResponsePayload,
                                                                    NewResponseHeader,
                                                                    S>&&;

    /// If the [`Service`] exists, it will be opened otherwise a new [`Service`] will be
    /// created.
    auto open_or_create() && -> iox::expected<
        PortFactoryRequestResponse<S, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>,
        RequestResponseOpenOrCreateError>;

    /// If the [`Service`] exists, it will be opened otherwise a new [`Service`] will be
    /// created. It defines a set of attributes.
    ///
    /// If the [`Service`] already exists all attribute requirements must be satisfied,
    /// and service payload type must be the same, otherwise the open process will fail.
    /// If the [`Service`] does not exist the required attributes will be defined in the [`Service`].
    auto open_or_create_with_attributes(const AttributeVerifier& required_attributes) && -> iox::expected<
        PortFactoryRequestResponse<S, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>,
        RequestResponseOpenOrCreateError>;

    /// Opens an existing [`Service`].
    auto open() && -> iox::expected<
        PortFactoryRequestResponse<S, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>,
        RequestResponseOpenError>;

    /// Opens an existing [`Service`] with attribute requirements. If the defined attribute
    /// requirements are not satisfied the open process will fail.
    auto open_with_attributes(const AttributeVerifier& required_attributes) && -> iox::expected<
        PortFactoryRequestResponse<S, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>,
        RequestResponseOpenError>;

    /// Creates a new [`Service`].
    auto create() && -> iox::expected<
        PortFactoryRequestResponse<S, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>,
        RequestResponseCreateError>;

    /// Creates a new [`Service`] with a set of attributes.
    auto create_with_attributes(const AttributeVerifier& attributes) && -> iox::expected<
        PortFactoryRequestResponse<S, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>,
        RequestResponseCreateError>;

  private:
    template <ServiceType>
    friend class ServiceBuilder;

    explicit ServiceBuilderRequestResponse(iox2_service_builder_h handle);

    void set_parameters();

    iox2_service_builder_pub_sub_h m_handle = nullptr;
};

template <typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader,
          ServiceType S>
template <typename NewRequestHeader>
inline auto ServiceBuilderRequestResponse<RequestPayload, RequestHeader, ResponsePayload, ResponseHeader, S>::
    request_user_header() && -> ServiceBuilderRequestResponse<RequestPayload,
                                                              NewRequestHeader,
                                                              ResponsePayload,
                                                              ResponseHeader,
                                                              S>&& {
    IOX_TODO();
}

template <typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader,
          ServiceType S>
template <typename NewResponseHeader>
inline auto ServiceBuilderRequestResponse<RequestPayload, RequestHeader, ResponsePayload, ResponseHeader, S>::
    response_user_header() && -> ServiceBuilderRequestResponse<RequestPayload,
                                                               RequestHeader,
                                                               ResponsePayload,
                                                               NewResponseHeader,
                                                               S>&& {
    IOX_TODO();
}

template <typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader,
          ServiceType S>
inline auto ServiceBuilderRequestResponse<RequestPayload, RequestHeader, ResponsePayload, ResponseHeader, S>::
    open_or_create() && -> iox::expected<
        PortFactoryRequestResponse<S, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>,
        RequestResponseOpenOrCreateError> {
    IOX_TODO();
}

template <typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader,
          ServiceType S>
inline auto ServiceBuilderRequestResponse<RequestPayload, RequestHeader, ResponsePayload, ResponseHeader, S>::
    open_or_create_with_attributes([[maybe_unused]] const AttributeVerifier& required_attributes) && -> iox::expected<
        PortFactoryRequestResponse<S, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>,
        RequestResponseOpenOrCreateError> {
    IOX_TODO();
}

template <typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader,
          ServiceType S>
inline auto
ServiceBuilderRequestResponse<RequestPayload, RequestHeader, ResponsePayload, ResponseHeader, S>::open() && -> iox::
    expected<PortFactoryRequestResponse<S, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>,
             RequestResponseOpenError> {
    IOX_TODO();
}

template <typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader,
          ServiceType S>
inline auto
ServiceBuilderRequestResponse<RequestPayload, RequestHeader, ResponsePayload, ResponseHeader, S>::open_with_attributes(
    [[maybe_unused]] const AttributeVerifier& required_attributes) && -> iox::
    expected<PortFactoryRequestResponse<S, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>,
             RequestResponseOpenError> {
    IOX_TODO();
}

template <typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader,
          ServiceType S>
inline auto
ServiceBuilderRequestResponse<RequestPayload, RequestHeader, ResponsePayload, ResponseHeader, S>::create() && -> iox::
    expected<PortFactoryRequestResponse<S, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>,
             RequestResponseCreateError> {
    IOX_TODO();
}

template <typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader,
          ServiceType S>
inline auto ServiceBuilderRequestResponse<RequestPayload, RequestHeader, ResponsePayload, ResponseHeader, S>::
    create_with_attributes([[maybe_unused]] const AttributeVerifier& attributes) && -> iox::expected<
        PortFactoryRequestResponse<S, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>,
        RequestResponseCreateError> {
    IOX_TODO();
}

template <typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader,
          ServiceType S>
inline ServiceBuilderRequestResponse<RequestPayload, RequestHeader, ResponsePayload, ResponseHeader, S>::
    ServiceBuilderRequestResponse([[maybe_unused]] iox2_service_builder_h handle) {
    IOX_TODO();
}

template <typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader,
          ServiceType S>
inline void
ServiceBuilderRequestResponse<RequestPayload, RequestHeader, ResponsePayload, ResponseHeader, S>::set_parameters() {
    IOX_TODO();
}
} // namespace iox2
#endif
