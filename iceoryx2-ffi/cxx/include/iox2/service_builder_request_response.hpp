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
#include "iox/layout.hpp"
#include "iox2/attribute_specifier.hpp"
#include "iox2/attribute_verifier.hpp"
#include "iox2/internal/iceoryx2.hpp"
#include "iox2/internal/service_builder_internal.hpp"
#include "iox2/payload_info.hpp"
#include "iox2/port_factory_request_response.hpp"
#include "iox2/service_builder_request_response_error.hpp"
#include "iox2/service_type.hpp"

namespace iox2 {
template <typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader,
          ServiceType S>
class ServiceBuilderRequestResponse {
  public:
    /// If the [`Service`] is created, it defines the request [`Alignment`] of the payload for the
    /// service. If an existing [`Service`] is opened it requires the service to have at least the
    /// defined [`Alignment`]. If the Payload [`Alignment`] is greater than the provided
    /// [`Alignment`] then the Payload [`Alignment`] is used.
#ifdef DOXYGEN_MACRO_FIX
    auto request_payload_alignment(const uint64_t value) -> decltype(auto);
#else
    IOX_BUILDER_OPTIONAL(uint64_t, request_payload_alignment);
#endif

    /// If the [`Service`] is created, it defines the response [`Alignment`] of the payload for the
    /// service. If an existing [`Service`] is opened it requires the service to have at least the
    /// defined [`Alignment`]. If the Payload [`Alignment`] is greater than the provided
    /// [`Alignment`] then the Payload [`Alignment`] is used.
#ifdef DOXYGEN_MACRO_FIX
    auto response_payload_alignment(const uint64_t value) -> decltype(auto);
#else
    IOX_BUILDER_OPTIONAL(uint64_t, response_payload_alignment);
#endif

    /// If the [`Service`] is created, defines the overflow behavior of the service for requests.
    /// If an existing [`Service`] is opened it requires the service to have the defined overflow
    /// behavior.
#ifdef DOXYGEN_MACRO_FIX
    auto enable_safe_overflow_for_requests(const bool value) -> decltype(auto);
#else
    IOX_BUILDER_OPTIONAL(bool, enable_safe_overflow_for_requests);
#endif

    /// If the [`Service`] is created, defines the overflow behavior of the service for responses.
    /// If an existing [`Service`] is opened it requires the service to have the defined overflow
    /// behavior.
#ifdef DOXYGEN_MACRO_FIX
    auto enable_safe_overflow_for_responses(const bool value) -> decltype(auto);
#else
    IOX_BUILDER_OPTIONAL(bool, enable_safe_overflow_for_responses);
#endif

    /// Defines how many active requests a [`Server`] can hold in
    /// parallel per [`Client`]. The objects are used to send answers to a request that was
    /// received earlier from a [`Client`]
#ifdef DOXYGEN_MACRO_FIX
    auto max_active_requests_per_client(const uint64_t value) -> decltype(auto);
#else
    IOX_BUILDER_OPTIONAL(uint64_t, max_active_requests_per_client);
#endif

    /// If the [`Service`] is created it defines how many responses fit in the
    /// [`Clients`]s buffer. If an existing
    /// [`Service`] is opened it defines the minimum required.
#ifdef DOXYGEN_MACRO_FIX
    auto max_response_buffer_size(const uint64_t value) -> decltype(auto);
#else
    IOX_BUILDER_OPTIONAL(uint64_t, max_response_buffer_size);
#endif

    /// If the [`Service`] is created it defines how many [`Server`]s shall
    /// be supported at most. If an existing [`Service`] is opened it defines how many
    /// [`Server`]s must be at least supported.
#ifdef DOXYGEN_MACRO_FIX
    auto max_servers(const uint64_t value) -> decltype(auto);
#else
    IOX_BUILDER_OPTIONAL(uint64_t, max_servers);
#endif

    /// If the [`Service`] is created it defines how many [`Client`]s shall
    /// be supported at most. If an existing [`Service`] is opened it defines how many
    /// [`Client`]s must be at least supported.
#ifdef DOXYGEN_MACRO_FIX
    auto max_clients(const uint64_t value) -> decltype(auto);
#else
    IOX_BUILDER_OPTIONAL(uint64_t, max_clients);
#endif

    /// If the [`Service`] is created it defines how many [`Node`]s shall
    /// be able to open it in parallel. If an existing [`Service`] is opened it defines how many
    /// [`Node`]s must be at least supported.
#ifdef DOXYGEN_MACRO_FIX
    auto max_nodes(const uint64_t value) -> decltype(auto);
#else
    IOX_BUILDER_OPTIONAL(uint64_t, max_nodes);
#endif

    /// If the [`Service`] is created it defines how many [`Response`]s shall
    /// be able to be borrowed in parallel per [`PendingResponse`]. If an
    /// existing [`Service`] is opened it defines how many borrows must be at least supported.
#ifdef DOXYGEN_MACRO_FIX
    auto max_borrowed_responses_per_pending_response(const uint64_t value) -> decltype(auto);
#else
    IOX_BUILDER_OPTIONAL(uint64_t, max_borrowed_responses_per_pending_response);
#endif

    /// If the [`Service`] is created it defines how many [`RequestMut`] a
    /// [`Client`] can loan in parallel.
#ifdef DOXYGEN_MACRO_FIX
    auto max_loaned_requests(const uint64_t value) -> decltype(auto);
#else
    IOX_BUILDER_OPTIONAL(uint64_t, max_loaned_requests);
#endif

    /// If the [`Service`] is created, defines the fire-and-forget behavior of the service for requests.
#ifdef DOXYGEN_MACRO_FIX
    auto enable_fire_and_forget_requests(const bool value) -> decltype(auto);
#else
    IOX_BUILDER_OPTIONAL(bool, enable_fire_and_forget_requests);
#endif

  public:
    /// Sets the request user header type of the [`Service`].
    template <typename NewRequestUserHeader>
    auto request_user_header() && -> ServiceBuilderRequestResponse<RequestPayload,
                                                                   NewRequestUserHeader,
                                                                   ResponsePayload,
                                                                   ResponseUserHeader,
                                                                   S>&&;

    /// Sets the response user header type of the [`Service`].
    template <typename NewResponseUserHeader>
    auto response_user_header() && -> ServiceBuilderRequestResponse<RequestPayload,
                                                                    RequestUserHeader,
                                                                    ResponsePayload,
                                                                    NewResponseUserHeader,
                                                                    S>&&;

    /// If the [`Service`] exists, it will be opened otherwise a new [`Service`] will be
    /// created.
    auto open_or_create() && -> iox::expected<
        PortFactoryRequestResponse<S, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>,
        RequestResponseOpenOrCreateError>;

    /// If the [`Service`] exists, it will be opened otherwise a new [`Service`] will be
    /// created. It defines a set of attributes.
    ///
    /// If the [`Service`] already exists all attribute requirements must be satisfied,
    /// and service payload type must be the same, otherwise the open process will fail.
    /// If the [`Service`] does not exist the required attributes will be defined in the [`Service`].
    auto open_or_create_with_attributes(const AttributeVerifier& required_attributes) && -> iox::expected<
        PortFactoryRequestResponse<S, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>,
        RequestResponseOpenOrCreateError>;

    /// Opens an existing [`Service`].
    auto open() && -> iox::expected<
        PortFactoryRequestResponse<S, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>,
        RequestResponseOpenError>;

    /// Opens an existing [`Service`] with attribute requirements. If the defined attribute
    /// requirements are not satisfied the open process will fail.
    auto open_with_attributes(const AttributeVerifier& required_attributes) && -> iox::expected<
        PortFactoryRequestResponse<S, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>,
        RequestResponseOpenError>;

    /// Creates a new [`Service`].
    auto create() && -> iox::expected<
        PortFactoryRequestResponse<S, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>,
        RequestResponseCreateError>;

    /// Creates a new [`Service`] with a set of attributes.
    auto create_with_attributes(const AttributeSpecifier& attributes) && -> iox::expected<
        PortFactoryRequestResponse<S, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>,
        RequestResponseCreateError>;

  private:
    template <ServiceType>
    friend class ServiceBuilder;

    explicit ServiceBuilderRequestResponse(iox2_service_builder_h handle);

    void set_parameters();

    iox2_service_builder_request_response_h m_handle = nullptr;
};

template <typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader,
          ServiceType S>
template <typename NewRequestUserHeader>
inline auto ServiceBuilderRequestResponse<RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader, S>::
    request_user_header() && -> ServiceBuilderRequestResponse<RequestPayload,
                                                              NewRequestUserHeader,
                                                              ResponsePayload,
                                                              ResponseUserHeader,
                                                              S>&& {
    return std::move(
        // required here since we just change the template header type but the builder structure stays the same
        // NOLINTNEXTLINE(cppcoreguidelines-pro-type-reinterpret-cast)
        *reinterpret_cast<ServiceBuilderRequestResponse<RequestPayload,
                                                        NewRequestUserHeader,
                                                        ResponsePayload,
                                                        ResponseUserHeader,
                                                        S>*>(this));
}

template <typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader,
          ServiceType S>
template <typename NewResponseUserHeader>
inline auto ServiceBuilderRequestResponse<RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader, S>::
    response_user_header() && -> ServiceBuilderRequestResponse<RequestPayload,
                                                               RequestUserHeader,
                                                               ResponsePayload,
                                                               NewResponseUserHeader,
                                                               S>&& {
    return std::move(
        // required here since we just change the template header type but the builder structure stays the same
        // NOLINTNEXTLINE(cppcoreguidelines-pro-type-reinterpret-cast)
        *reinterpret_cast<ServiceBuilderRequestResponse<RequestPayload,
                                                        RequestUserHeader,
                                                        ResponsePayload,
                                                        NewResponseUserHeader,
                                                        S>*>(this));
}

template <typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader,
          ServiceType S>
inline auto ServiceBuilderRequestResponse<RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader, S>::
    open_or_create() && -> iox::expected<
        PortFactoryRequestResponse<S, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>,
        RequestResponseOpenOrCreateError> {
    set_parameters();

    iox2_port_factory_request_response_h port_factory_handle {};
    auto result = iox2_service_builder_request_response_open_or_create(m_handle, nullptr, &port_factory_handle);

    if (result == IOX2_OK) {
        return iox::ok(
            PortFactoryRequestResponse<S, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>(
                port_factory_handle));
    }

    return iox::err(iox::into<RequestResponseOpenOrCreateError>(result));
}

template <typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader,
          ServiceType S>
inline auto ServiceBuilderRequestResponse<RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader, S>::
    open_or_create_with_attributes(const AttributeVerifier& required_attributes) && -> iox::expected<
        PortFactoryRequestResponse<S, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>,
        RequestResponseOpenOrCreateError> {
    set_parameters();

    iox2_port_factory_request_response_h port_factory_handle {};
    auto result = iox2_service_builder_request_response_open_or_create_with_attributes(
        m_handle, &required_attributes.m_handle, nullptr, &port_factory_handle);

    if (result == IOX2_OK) {
        return iox::ok(
            PortFactoryRequestResponse<S, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>(
                port_factory_handle));
    }

    return iox::err(iox::into<RequestResponseOpenOrCreateError>(result));
}

template <typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader,
          ServiceType S>
inline auto ServiceBuilderRequestResponse<RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader, S>::
    open() && -> iox::expected<
        PortFactoryRequestResponse<S, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>,
        RequestResponseOpenError> {
    set_parameters();

    iox2_port_factory_request_response_h port_factory_handle {};
    auto result = iox2_service_builder_request_response_open(m_handle, nullptr, &port_factory_handle);

    if (result == IOX2_OK) {
        return iox::ok(
            PortFactoryRequestResponse<S, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>(
                port_factory_handle));
    }

    return iox::err(iox::into<RequestResponseOpenError>(result));
}

template <typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader,
          ServiceType S>
inline auto ServiceBuilderRequestResponse<RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader, S>::
    open_with_attributes(const AttributeVerifier& required_attributes) && -> iox::expected<
        PortFactoryRequestResponse<S, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>,
        RequestResponseOpenError> {
    set_parameters();

    iox2_port_factory_request_response_h port_factory_handle {};
    auto result = iox2_service_builder_request_response_open_with_attributes(
        m_handle, &required_attributes.m_handle, nullptr, &port_factory_handle);

    if (result == IOX2_OK) {
        return iox::ok(
            PortFactoryRequestResponse<S, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>(
                port_factory_handle));
    }

    return iox::err(iox::into<RequestResponseOpenError>(result));
}

template <typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader,
          ServiceType S>
inline auto ServiceBuilderRequestResponse<RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader, S>::
    create() && -> iox::expected<
        PortFactoryRequestResponse<S, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>,
        RequestResponseCreateError> {
    set_parameters();

    iox2_port_factory_request_response_h port_factory_handle {};
    auto result = iox2_service_builder_request_response_create(m_handle, nullptr, &port_factory_handle);

    if (result == IOX2_OK) {
        return iox::ok(
            PortFactoryRequestResponse<S, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>(
                port_factory_handle));
    }

    return iox::err(iox::into<RequestResponseCreateError>(result));
}

template <typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader,
          ServiceType S>
inline auto ServiceBuilderRequestResponse<RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader, S>::
    create_with_attributes(const AttributeSpecifier& attributes) && -> iox::expected<
        PortFactoryRequestResponse<S, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>,
        RequestResponseCreateError> {
    set_parameters();

    iox2_port_factory_request_response_h port_factory_handle {};
    auto result = iox2_service_builder_request_response_create_with_attributes(
        m_handle, &attributes.m_handle, nullptr, &port_factory_handle);

    if (result == IOX2_OK) {
        return iox::ok(
            PortFactoryRequestResponse<S, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>(
                port_factory_handle));
    }

    return iox::err(iox::into<RequestResponseCreateError>(result));
}

template <typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader,
          ServiceType S>
inline ServiceBuilderRequestResponse<RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader, S>::
    ServiceBuilderRequestResponse(iox2_service_builder_h handle)
    : m_handle { iox2_service_builder_request_response(handle) } {
}

template <typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader,
          ServiceType S>
inline void ServiceBuilderRequestResponse<RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader, S>::
    set_parameters() {
    m_request_payload_alignment.and_then(
        [&](auto value) { iox2_service_builder_request_response_request_payload_alignment(&m_handle, value); });
    m_response_payload_alignment.and_then(
        [&](auto value) { iox2_service_builder_request_response_response_payload_alignment(&m_handle, value); });
    m_enable_safe_overflow_for_requests.and_then(
        [&](auto value) { iox2_service_builder_request_response_enable_safe_overflow_for_requests(&m_handle, value); });
    m_enable_safe_overflow_for_responses.and_then([&](auto value) {
        iox2_service_builder_request_response_enable_safe_overflow_for_responses(&m_handle, value);
    });
    m_max_active_requests_per_client.and_then(
        [&](auto value) { iox2_service_builder_request_response_max_active_requests_per_client(&m_handle, value); });
    m_max_response_buffer_size.and_then(
        [&](auto value) { iox2_service_builder_request_response_max_response_buffer_size(&m_handle, value); });
    m_max_servers.and_then([&](auto value) { iox2_service_builder_request_response_max_servers(&m_handle, value); });
    m_max_clients.and_then([&](auto value) { iox2_service_builder_request_response_max_clients(&m_handle, value); });
    m_max_nodes.and_then([&](auto value) { iox2_service_builder_request_response_set_max_nodes(&m_handle, value); });
    m_max_borrowed_responses_per_pending_response.and_then([&](auto value) {
        iox2_service_builder_request_response_max_borrowed_responses_per_pending_response(&m_handle, value);
    });
    m_max_loaned_requests.and_then(
        [&](auto value) { iox2_service_builder_request_response_max_loaned_requests(&m_handle, value); });
    m_enable_fire_and_forget_requests.and_then(
        [&](auto value) { iox2_service_builder_request_response_enable_fire_and_forget_requests(&m_handle, value); });

    // request payload type details
    using RequestValueType = typename PayloadInfo<RequestPayload>::ValueType;
    auto type_variant_request_payload =
        iox::IsSlice<RequestPayload>::VALUE ? iox2_type_variant_e_DYNAMIC : iox2_type_variant_e_FIXED_SIZE;

    const auto* request_payload_type_name = internal::get_payload_type_name<RequestPayload>();
    const auto request_payload_type_name_len = strlen(request_payload_type_name);
    const auto request_payload_type_size = sizeof(RequestValueType);
    const auto request_payload_type_align = alignof(RequestValueType);

    const auto request_payload_result =
        iox2_service_builder_request_response_set_request_payload_type_details(&m_handle,
                                                                               type_variant_request_payload,
                                                                               request_payload_type_name,
                                                                               request_payload_type_name_len,
                                                                               request_payload_type_size,
                                                                               request_payload_type_align);

    if (request_payload_result != IOX2_OK) {
        IOX_PANIC("This should never happen! Implementation failure while setting the RequestPayload-Type.");
    }

    // response payload type details
    using ResponseValueType = typename PayloadInfo<ResponsePayload>::ValueType;
    auto type_variant_response_payload =
        iox::IsSlice<ResponsePayload>::VALUE ? iox2_type_variant_e_DYNAMIC : iox2_type_variant_e_FIXED_SIZE;

    const auto* response_payload_type_name = internal::get_payload_type_name<ResponsePayload>();
    const auto response_payload_type_name_len = strlen(response_payload_type_name);
    const auto response_payload_type_size = sizeof(ResponseValueType);
    const auto response_payload_type_align = alignof(ResponseValueType);

    const auto response_payload_result =
        iox2_service_builder_request_response_set_response_payload_type_details(&m_handle,
                                                                                type_variant_response_payload,
                                                                                response_payload_type_name,
                                                                                response_payload_type_name_len,
                                                                                response_payload_type_size,
                                                                                response_payload_type_align);

    if (response_payload_result != IOX2_OK) {
        IOX_PANIC("This should never happen! Implementation failure while setting the ResponsePayload-Type.");
    }

    // request header type details
    const auto request_header_layout = iox::Layout::from<RequestUserHeader>();
    const auto* request_header_type_name = internal::get_user_header_type_name<RequestUserHeader>();
    const auto request_header_type_name_len = strlen(request_header_type_name);
    const auto request_header_type_size = request_header_layout.size();
    const auto request_header_type_align = request_header_layout.alignment();

    const auto request_header_result =
        iox2_service_builder_request_response_set_request_header_type_details(&m_handle,
                                                                              iox2_type_variant_e_FIXED_SIZE,
                                                                              request_header_type_name,
                                                                              request_header_type_name_len,
                                                                              request_header_type_size,
                                                                              request_header_type_align);

    if (request_header_result != IOX2_OK) {
        IOX_PANIC("This should never happen! Implementation failure while setting the Request-Header-Type.");
    }

    // response header type details
    const auto response_header_layout = iox::Layout::from<ResponseUserHeader>();
    const auto* response_header_type_name = internal::get_user_header_type_name<ResponseUserHeader>();
    const auto response_header_type_name_len = strlen(response_header_type_name);
    const auto response_header_type_size = response_header_layout.size();
    const auto response_header_type_align = response_header_layout.alignment();

    const auto response_header_result =
        iox2_service_builder_request_response_set_response_header_type_details(&m_handle,
                                                                               iox2_type_variant_e_FIXED_SIZE,
                                                                               response_header_type_name,
                                                                               response_header_type_name_len,
                                                                               response_header_type_size,
                                                                               response_header_type_align);

    if (response_header_result != IOX2_OK) {
        IOX_PANIC("This should never happen! Implementation failure while setting the Response-Header-Type.");
    }
}
} // namespace iox2
#endif
