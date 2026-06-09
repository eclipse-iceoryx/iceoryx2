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

#include "iox2/attribute_specifier.hpp"
#include "iox2/attribute_verifier.hpp"
#include "iox2/bb/detail/builder.hpp"
#include "iox2/bb/expected.hpp"
#include "iox2/bb/layout.hpp"
#include "iox2/bb/slice.hpp"
#include "iox2/custom_header_marker.hpp"
#include "iox2/custom_payload_marker.hpp"
#include "iox2/internal/iceoryx2.hpp"
#include "iox2/internal/service_builder_internal.hpp"
#include "iox2/message_type_details.hpp"
#include "iox2/payload_info.hpp"
#include "iox2/port_factory_request_response.hpp"
#include "iox2/service_builder_request_response_error.hpp"
#include "iox2/service_type.hpp"
#include "iox2/type_name.hpp"
#include "iox2/type_variant.hpp"

#include <cstring>
#include <type_traits>

namespace iox2 {
template <typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader,
          ServiceType S>
class ServiceBuilderRequestResponse;

/// Overrides the request user header type details with values provided at runtime instead of derived
/// from the compile-time `RequestUserHeader`. Only available for `CustomHeaderMarker`.
///
/// # Safety
///
///  * It is preferred to let the type details be derived from the provided type; overriding them is
///    only meant for advanced usage.
///  * The provided [`TypeDetail`] must accurately describe the request user header type that is
///    accessed at runtime; a mismatching size or alignment leads to undefined behavior.
template <typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader,
          ServiceType S>
auto set_request_header_type_details(
    ServiceBuilderRequestResponse<RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader, S>& builder,
    const TypeDetail& value) -> std::enable_if_t<std::is_same<RequestUserHeader, CustomHeaderMarker>::value>;

/// Overrides the response user header type details with values provided at runtime instead of
/// derived from the compile-time `ResponseUserHeader`. Only available for `CustomHeaderMarker`.
///
/// # Safety
///
///  * It is preferred to let the type details be derived from the provided type; overriding them is
///    only meant for advanced usage.
///  * The provided [`TypeDetail`] must accurately describe the response user header type that is
///    accessed at runtime; a mismatching size or alignment leads to undefined behavior.
template <typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader,
          ServiceType S>
auto set_response_header_type_details(
    ServiceBuilderRequestResponse<RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader, S>& builder,
    const TypeDetail& value) -> std::enable_if_t<std::is_same<ResponseUserHeader, CustomHeaderMarker>::value>;

/// Overrides the request payload type details with values provided at runtime instead of derived
/// from the compile-time `RequestPayload`. Only available for `bb::Slice<CustomPayloadMarker>`.
///
/// # Safety
///
///  * It is preferred to let the type details be derived from the provided type; overriding them is
///    only meant for advanced usage.
///  * The provided [`TypeDetail`] must accurately describe the request payload type that is loaned,
///    sent and received at runtime; a mismatching size or alignment leads to undefined behavior.
template <typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader,
          ServiceType S>
auto set_request_payload_type_details(
    ServiceBuilderRequestResponse<RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader, S>& builder,
    const TypeDetail& value) -> std::enable_if_t<std::is_same<RequestPayload, bb::Slice<CustomPayloadMarker>>::value>;

/// Overrides the response payload type details with values provided at runtime instead of derived
/// from the compile-time `ResponsePayload`. Only available for `bb::Slice<CustomPayloadMarker>`.
///
/// # Safety
///
///  * It is preferred to let the type details be derived from the provided type; overriding them is
///    only meant for advanced usage.
///  * The provided [`TypeDetail`] must accurately describe the response payload type that is loaned,
///    sent and received at runtime; a mismatching size or alignment leads to undefined behavior.
template <typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader,
          ServiceType S>
auto set_response_payload_type_details(
    ServiceBuilderRequestResponse<RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader, S>& builder,
    const TypeDetail& value) -> std::enable_if_t<std::is_same<ResponsePayload, bb::Slice<CustomPayloadMarker>>::value>;

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
    IOX2_BUILDER_OPTIONAL(uint64_t, request_payload_alignment);
#endif

    /// If the [`Service`] is created, it defines the response [`Alignment`] of the payload for the
    /// service. If an existing [`Service`] is opened it requires the service to have at least the
    /// defined [`Alignment`]. If the Payload [`Alignment`] is greater than the provided
    /// [`Alignment`] then the Payload [`Alignment`] is used.
#ifdef DOXYGEN_MACRO_FIX
    auto response_payload_alignment(const uint64_t value) -> decltype(auto);
#else
    IOX2_BUILDER_OPTIONAL(uint64_t, response_payload_alignment);
#endif

    /// If the [`Service`] is created, defines the overflow behavior of the service for requests.
    /// If an existing [`Service`] is opened it requires the service to have the defined overflow
    /// behavior.
#ifdef DOXYGEN_MACRO_FIX
    auto enable_safe_overflow_for_requests(const bool value) -> decltype(auto);
#else
    IOX2_BUILDER_OPTIONAL(bool, enable_safe_overflow_for_requests);
#endif

    /// If the [`Service`] is created, defines the overflow behavior of the service for responses.
    /// If an existing [`Service`] is opened it requires the service to have the defined overflow
    /// behavior.
#ifdef DOXYGEN_MACRO_FIX
    auto enable_safe_overflow_for_responses(const bool value) -> decltype(auto);
#else
    IOX2_BUILDER_OPTIONAL(bool, enable_safe_overflow_for_responses);
#endif

    /// Defines how many active requests a [`Server`] can hold in
    /// parallel per [`Client`]. The objects are used to send answers to a request that was
    /// received earlier from a [`Client`]
#ifdef DOXYGEN_MACRO_FIX
    auto max_active_requests_per_client(const uint64_t value) -> decltype(auto);
#else
    IOX2_BUILDER_OPTIONAL(uint64_t, max_active_requests_per_client);
#endif

    /// If the [`Service`] is created it defines how many responses fit in the
    /// [`Clients`]s buffer. If an existing
    /// [`Service`] is opened it defines the minimum required.
#ifdef DOXYGEN_MACRO_FIX
    auto max_response_buffer_size(const uint64_t value) -> decltype(auto);
#else
    IOX2_BUILDER_OPTIONAL(uint64_t, max_response_buffer_size);
#endif

    /// If the [`Service`] is created it defines how many [`Server`]s shall
    /// be supported at most. If an existing [`Service`] is opened it defines how many
    /// [`Server`]s must be at least supported.
#ifdef DOXYGEN_MACRO_FIX
    auto max_servers(const uint64_t value) -> decltype(auto);
#else
    IOX2_BUILDER_OPTIONAL(uint64_t, max_servers);
#endif

    /// If the [`Service`] is created it defines how many [`Client`]s shall
    /// be supported at most. If an existing [`Service`] is opened it defines how many
    /// [`Client`]s must be at least supported.
#ifdef DOXYGEN_MACRO_FIX
    auto max_clients(const uint64_t value) -> decltype(auto);
#else
    IOX2_BUILDER_OPTIONAL(uint64_t, max_clients);
#endif

    /// If the [`Service`] is created it defines how many [`Node`]s shall
    /// be able to open it in parallel. If an existing [`Service`] is opened it defines how many
    /// [`Node`]s must be at least supported.
#ifdef DOXYGEN_MACRO_FIX
    auto max_nodes(const uint64_t value) -> decltype(auto);
#else
    IOX2_BUILDER_OPTIONAL(uint64_t, max_nodes);
#endif

    /// If the [`Service`] is created it defines how many [`Response`]s shall
    /// be able to be borrowed in parallel per [`PendingResponse`]. If an
    /// existing [`Service`] is opened it defines how many borrows must be at least supported.
#ifdef DOXYGEN_MACRO_FIX
    auto max_borrowed_responses_per_pending_response(const uint64_t value) -> decltype(auto);
#else
    IOX2_BUILDER_OPTIONAL(uint64_t, max_borrowed_responses_per_pending_response);
#endif

    /// If the [`Service`] is created it defines how many [`RequestMut`] a
    /// [`Client`] can loan in parallel.
#ifdef DOXYGEN_MACRO_FIX
    auto max_loaned_requests(const uint64_t value) -> decltype(auto);
#else
    IOX2_BUILDER_OPTIONAL(uint64_t, max_loaned_requests);
#endif

    /// If the [`Service`] is created, defines the fire-and-forget behavior of the service for requests.
#ifdef DOXYGEN_MACRO_FIX
    auto enable_fire_and_forget_requests(const bool value) -> decltype(auto);
#else
    IOX2_BUILDER_OPTIONAL(bool, enable_fire_and_forget_requests);
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

    /// Returns the builder as an r-value so the fluent chain can be resumed after a free function,
    /// such as [`set_request_header_type_details()`] or [`set_request_payload_type_details()`], has
    /// been applied to the named builder.
    auto resume_build() & -> ServiceBuilderRequestResponse<RequestPayload,
                                                           RequestUserHeader,
                                                           ResponsePayload,
                                                           ResponseUserHeader,
                                                           S>&&;

    /// If the [`Service`] exists, it will be opened otherwise a new [`Service`] will be
    /// created.
    auto open_or_create() && -> bb::Expected<
        PortFactoryRequestResponse<S, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>,
        RequestResponseOpenOrCreateError>;

    /// If the [`Service`] exists, it will be opened otherwise a new [`Service`] will be
    /// created. It defines a set of attributes.
    ///
    /// If the [`Service`] already exists all attribute requirements must be satisfied,
    /// and service payload type must be the same, otherwise the open process will fail.
    /// If the [`Service`] does not exist the required attributes will be defined in the [`Service`].
    auto open_or_create_with_attributes(const AttributeVerifier& required_attributes) && -> bb::Expected<
        PortFactoryRequestResponse<S, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>,
        RequestResponseOpenOrCreateError>;

    /// Opens an existing [`Service`].
    auto open() && -> bb::Expected<
        PortFactoryRequestResponse<S, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>,
        RequestResponseOpenError>;

    /// Opens an existing [`Service`] with attribute requirements. If the defined attribute
    /// requirements are not satisfied the open process will fail.
    auto open_with_attributes(const AttributeVerifier& required_attributes) && -> bb::Expected<
        PortFactoryRequestResponse<S, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>,
        RequestResponseOpenError>;

    /// Creates a new [`Service`].
    auto create() && -> bb::Expected<
        PortFactoryRequestResponse<S, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>,
        RequestResponseCreateError>;

    /// Creates a new [`Service`] with a set of attributes.
    auto create_with_attributes(const AttributeSpecifier& attributes) && -> bb::Expected<
        PortFactoryRequestResponse<S, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>,
        RequestResponseCreateError>;

  private:
    template <ServiceType>
    friend class ServiceBuilder;

    template <typename ReqP, typename ReqH, typename ResP, typename ResH, ServiceType St>
    friend auto set_request_header_type_details(ServiceBuilderRequestResponse<ReqP, ReqH, ResP, ResH, St>& builder,
                                                const TypeDetail& value)
        -> std::enable_if_t<std::is_same<ReqH, CustomHeaderMarker>::value>;
    template <typename ReqP, typename ReqH, typename ResP, typename ResH, ServiceType St>
    friend auto set_response_header_type_details(ServiceBuilderRequestResponse<ReqP, ReqH, ResP, ResH, St>& builder,
                                                 const TypeDetail& value)
        -> std::enable_if_t<std::is_same<ResH, CustomHeaderMarker>::value>;
    template <typename ReqP, typename ReqH, typename ResP, typename ResH, ServiceType St>
    friend auto set_request_payload_type_details(ServiceBuilderRequestResponse<ReqP, ReqH, ResP, ResH, St>& builder,
                                                 const TypeDetail& value)
        -> std::enable_if_t<std::is_same<ReqP, bb::Slice<CustomPayloadMarker>>::value>;
    template <typename ReqP, typename ReqH, typename ResP, typename ResH, ServiceType St>
    friend auto set_response_payload_type_details(ServiceBuilderRequestResponse<ReqP, ReqH, ResP, ResH, St>& builder,
                                                  const TypeDetail& value)
        -> std::enable_if_t<std::is_same<ResP, bb::Slice<CustomPayloadMarker>>::value>;

    explicit ServiceBuilderRequestResponse(iox2_service_builder_h handle);

    void set_parameters();
    void derive_request_header_type_details();
    void override_request_header_type_details(const TypeDetail& value);
    void derive_response_header_type_details();
    void override_response_header_type_details(const TypeDetail& value);
    void derive_request_payload_type_details();
    void override_request_payload_type_details(const TypeDetail& value);
    void derive_response_payload_type_details();
    void override_response_payload_type_details(const TypeDetail& value);

    iox2_service_builder_request_response_h m_handle = nullptr;
    bb::Optional<TypeDetail> m_request_header_type_details_override;
    bb::Optional<TypeDetail> m_response_header_type_details_override;
    bb::Optional<TypeDetail> m_request_payload_type_details_override;
    bb::Optional<TypeDetail> m_response_payload_type_details_override;
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
    resume_build() & -> ServiceBuilderRequestResponse<RequestPayload,
                                                      RequestUserHeader,
                                                      ResponsePayload,
                                                      ResponseUserHeader,
                                                      S>&& {
    return std::move(*this);
}

template <typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader,
          ServiceType S>
inline auto ServiceBuilderRequestResponse<RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader, S>::
    open_or_create() && -> bb::Expected<
        PortFactoryRequestResponse<S, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>,
        RequestResponseOpenOrCreateError> {
    set_parameters();

    iox2_port_factory_request_response_h port_factory_handle {};
    auto result = iox2_service_builder_request_response_open_or_create(m_handle, nullptr, &port_factory_handle);

    if (result == IOX2_OK) {
        return PortFactoryRequestResponse<S, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>(
            port_factory_handle);
    }

    return bb::err(bb::into<RequestResponseOpenOrCreateError>(result));
}

template <typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader,
          ServiceType S>
inline auto ServiceBuilderRequestResponse<RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader, S>::
    open_or_create_with_attributes(const AttributeVerifier& required_attributes) && -> bb::Expected<
        PortFactoryRequestResponse<S, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>,
        RequestResponseOpenOrCreateError> {
    set_parameters();

    iox2_port_factory_request_response_h port_factory_handle {};
    auto result = iox2_service_builder_request_response_open_or_create_with_attributes(
        m_handle, &required_attributes.m_handle, nullptr, &port_factory_handle);

    if (result == IOX2_OK) {
        return PortFactoryRequestResponse<S, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>(
            port_factory_handle);
    }

    return bb::err(bb::into<RequestResponseOpenOrCreateError>(result));
}

template <typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader,
          ServiceType S>
inline auto ServiceBuilderRequestResponse<RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader, S>::
    open() && -> bb::Expected<
        PortFactoryRequestResponse<S, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>,
        RequestResponseOpenError> {
    set_parameters();

    iox2_port_factory_request_response_h port_factory_handle {};
    auto result = iox2_service_builder_request_response_open(m_handle, nullptr, &port_factory_handle);

    if (result == IOX2_OK) {
        return PortFactoryRequestResponse<S, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>(
            port_factory_handle);
    }

    return bb::err(bb::into<RequestResponseOpenError>(result));
}

template <typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader,
          ServiceType S>
inline auto ServiceBuilderRequestResponse<RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader, S>::
    open_with_attributes(const AttributeVerifier& required_attributes) && -> bb::Expected<
        PortFactoryRequestResponse<S, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>,
        RequestResponseOpenError> {
    set_parameters();

    iox2_port_factory_request_response_h port_factory_handle {};
    auto result = iox2_service_builder_request_response_open_with_attributes(
        m_handle, &required_attributes.m_handle, nullptr, &port_factory_handle);

    if (result == IOX2_OK) {
        return PortFactoryRequestResponse<S, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>(
            port_factory_handle);
    }

    return bb::err(bb::into<RequestResponseOpenError>(result));
}

template <typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader,
          ServiceType S>
inline auto ServiceBuilderRequestResponse<RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader, S>::
    create() && -> bb::Expected<
        PortFactoryRequestResponse<S, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>,
        RequestResponseCreateError> {
    set_parameters();

    iox2_port_factory_request_response_h port_factory_handle {};
    auto result = iox2_service_builder_request_response_create(m_handle, nullptr, &port_factory_handle);

    if (result == IOX2_OK) {
        return PortFactoryRequestResponse<S, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>(
            port_factory_handle);
    }

    return bb::err(bb::into<RequestResponseCreateError>(result));
}

template <typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader,
          ServiceType S>
inline auto ServiceBuilderRequestResponse<RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader, S>::
    create_with_attributes(const AttributeSpecifier& attributes) && -> bb::Expected<
        PortFactoryRequestResponse<S, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>,
        RequestResponseCreateError> {
    set_parameters();

    iox2_port_factory_request_response_h port_factory_handle {};
    auto result = iox2_service_builder_request_response_create_with_attributes(
        m_handle, &attributes.m_handle, nullptr, &port_factory_handle);

    if (result == IOX2_OK) {
        return PortFactoryRequestResponse<S, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>(
            port_factory_handle);
    }

    return bb::err(bb::into<RequestResponseCreateError>(result));
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
    // NOLINTNEXTLINE(readability-function-size) the size cannot easily be reduced due to the amount of builder parameter
    set_parameters() {
    if (m_request_payload_alignment.has_value()) {
        iox2_service_builder_request_response_request_payload_alignment(&m_handle, m_request_payload_alignment.value());
    }
    if (m_response_payload_alignment.has_value()) {
        iox2_service_builder_request_response_response_payload_alignment(&m_handle,
                                                                         m_response_payload_alignment.value());
    }
    if (m_enable_safe_overflow_for_requests.has_value()) {
        iox2_service_builder_request_response_enable_safe_overflow_for_requests(
            &m_handle, m_enable_safe_overflow_for_requests.value());
    }
    if (m_enable_safe_overflow_for_responses.has_value()) {
        iox2_service_builder_request_response_enable_safe_overflow_for_responses(
            &m_handle, m_enable_safe_overflow_for_responses.value());
    }
    if (m_max_active_requests_per_client.has_value()) {
        iox2_service_builder_request_response_max_active_requests_per_client(&m_handle,
                                                                             m_max_active_requests_per_client.value());
    }
    if (m_max_response_buffer_size.has_value()) {
        iox2_service_builder_request_response_max_response_buffer_size(&m_handle, m_max_response_buffer_size.value());
    }
    if (m_max_servers.has_value()) {
        iox2_service_builder_request_response_max_servers(&m_handle, m_max_servers.value());
    }
    if (m_max_clients.has_value()) {
        iox2_service_builder_request_response_max_clients(&m_handle, m_max_clients.value());
    }
    if (m_max_nodes.has_value()) {
        iox2_service_builder_request_response_set_max_nodes(&m_handle, m_max_nodes.value());
    }
    if (m_max_borrowed_responses_per_pending_response.has_value()) {
        iox2_service_builder_request_response_max_borrowed_responses_per_pending_response(
            &m_handle, m_max_borrowed_responses_per_pending_response.value());
    }
    if (m_max_loaned_requests.has_value()) {
        iox2_service_builder_request_response_max_loaned_requests(&m_handle, m_max_loaned_requests.value());
    }
    if (m_enable_fire_and_forget_requests.has_value()) {
        iox2_service_builder_request_response_enable_fire_and_forget_requests(
            &m_handle, m_enable_fire_and_forget_requests.value());
    }

    if (m_request_header_type_details_override.has_value()) {
        override_request_header_type_details(m_request_header_type_details_override.value());
    } else {
        derive_request_header_type_details();
    }

    if (m_response_header_type_details_override.has_value()) {
        override_response_header_type_details(m_response_header_type_details_override.value());
    } else {
        derive_response_header_type_details();
    }

    if (m_request_payload_type_details_override.has_value()) {
        override_request_payload_type_details(m_request_payload_type_details_override.value());
    } else {
        derive_request_payload_type_details();
    }

    if (m_response_payload_type_details_override.has_value()) {
        override_response_payload_type_details(m_response_payload_type_details_override.value());
    } else {
        derive_response_payload_type_details();
    }
}

template <typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader,
          ServiceType S>
inline void ServiceBuilderRequestResponse<RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader, S>::
    derive_request_header_type_details() {
    // request header type details derived from the compile-time RequestUserHeader
    const auto header_layout = bb::Layout::from<RequestUserHeader>();
    const auto header_type_name = internal::get_type_name<RequestUserHeader>();

    const auto result = iox2_service_builder_request_response_set_request_header_type_details(
        &m_handle,
        iox2_type_variant_e_FIXED_SIZE,
        header_type_name.unchecked_access().c_str(),
        header_type_name.size(),
        header_layout.size(),
        header_layout.alignment());

    if (result != IOX2_OK) {
        IOX2_PANIC("This should never happen! Implementation failure while setting the Request-Header-Type.");
    }
}

template <typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader,
          ServiceType S>
inline void ServiceBuilderRequestResponse<RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader, S>::
    override_request_header_type_details(const TypeDetail& value) {
    // request header type details provided at runtime
    const auto type_variant =
        value.variant() == TypeVariant::FixedSize ? iox2_type_variant_e_FIXED_SIZE : iox2_type_variant_e_DYNAMIC;

    const auto result = iox2_service_builder_request_response_set_request_header_type_details(
        &m_handle, type_variant, value.type_name(), std::strlen(value.type_name()), value.size(), value.alignment());

    if (result != IOX2_OK) {
        IOX2_PANIC("This should never happen! Implementation failure while setting the Request-Header-Type.");
    }
}

template <typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader,
          ServiceType S>
inline void ServiceBuilderRequestResponse<RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader, S>::
    derive_response_header_type_details() {
    // response header type details derived from the compile-time ResponseUserHeader
    const auto header_layout = bb::Layout::from<ResponseUserHeader>();
    const auto header_type_name = internal::get_type_name<ResponseUserHeader>();

    const auto result = iox2_service_builder_request_response_set_response_header_type_details(
        &m_handle,
        iox2_type_variant_e_FIXED_SIZE,
        header_type_name.unchecked_access().c_str(),
        header_type_name.size(),
        header_layout.size(),
        header_layout.alignment());

    if (result != IOX2_OK) {
        IOX2_PANIC("This should never happen! Implementation failure while setting the Response-Header-Type.");
    }
}

template <typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader,
          ServiceType S>
inline void ServiceBuilderRequestResponse<RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader, S>::
    override_response_header_type_details(const TypeDetail& value) {
    // response header type details provided at runtime
    const auto type_variant =
        value.variant() == TypeVariant::FixedSize ? iox2_type_variant_e_FIXED_SIZE : iox2_type_variant_e_DYNAMIC;

    const auto result = iox2_service_builder_request_response_set_response_header_type_details(
        &m_handle, type_variant, value.type_name(), std::strlen(value.type_name()), value.size(), value.alignment());

    if (result != IOX2_OK) {
        IOX2_PANIC("This should never happen! Implementation failure while setting the Response-Header-Type.");
    }
}

template <typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader,
          ServiceType S>
inline void ServiceBuilderRequestResponse<RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader, S>::
    derive_request_payload_type_details() {
    using ValueType = typename PayloadInfo<RequestPayload>::ValueType;

    // request payload type details derived from the compile-time RequestPayload
    const auto payload_type_name = internal::get_type_name<RequestPayload>();
    const auto type_variant =
        bb::IsSlice<RequestPayload>::VALUE ? iox2_type_variant_e_DYNAMIC : iox2_type_variant_e_FIXED_SIZE;

    const auto result = iox2_service_builder_request_response_set_request_payload_type_details(
        &m_handle,
        type_variant,
        payload_type_name.unchecked_access().c_str(),
        payload_type_name.size(),
        sizeof(ValueType),
        alignof(ValueType));

    if (result != IOX2_OK) {
        IOX2_PANIC("This should never happen! Implementation failure while setting the RequestPayload-Type.");
    }
}

template <typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader,
          ServiceType S>
inline void ServiceBuilderRequestResponse<RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader, S>::
    override_request_payload_type_details(const TypeDetail& value) {
    // request payload type details provided at runtime
    const auto type_variant =
        value.variant() == TypeVariant::FixedSize ? iox2_type_variant_e_FIXED_SIZE : iox2_type_variant_e_DYNAMIC;

    const auto result = iox2_service_builder_request_response_set_request_payload_type_details(
        &m_handle, type_variant, value.type_name(), std::strlen(value.type_name()), value.size(), value.alignment());

    if (result != IOX2_OK) {
        IOX2_PANIC("This should never happen! Implementation failure while setting the RequestPayload-Type.");
    }
}

template <typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader,
          ServiceType S>
inline void ServiceBuilderRequestResponse<RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader, S>::
    derive_response_payload_type_details() {
    using ValueType = typename PayloadInfo<ResponsePayload>::ValueType;

    // response payload type details derived from the compile-time ResponsePayload
    const auto payload_type_name = internal::get_type_name<ResponsePayload>();
    const auto type_variant =
        bb::IsSlice<ResponsePayload>::VALUE ? iox2_type_variant_e_DYNAMIC : iox2_type_variant_e_FIXED_SIZE;

    const auto result = iox2_service_builder_request_response_set_response_payload_type_details(
        &m_handle,
        type_variant,
        payload_type_name.unchecked_access().c_str(),
        payload_type_name.size(),
        sizeof(ValueType),
        alignof(ValueType));

    if (result != IOX2_OK) {
        IOX2_PANIC("This should never happen! Implementation failure while setting the ResponsePayload-Type.");
    }
}

template <typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader,
          ServiceType S>
inline void ServiceBuilderRequestResponse<RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader, S>::
    override_response_payload_type_details(const TypeDetail& value) {
    // response payload type details provided at runtime
    const auto type_variant =
        value.variant() == TypeVariant::FixedSize ? iox2_type_variant_e_FIXED_SIZE : iox2_type_variant_e_DYNAMIC;

    const auto result = iox2_service_builder_request_response_set_response_payload_type_details(
        &m_handle, type_variant, value.type_name(), std::strlen(value.type_name()), value.size(), value.alignment());

    if (result != IOX2_OK) {
        IOX2_PANIC("This should never happen! Implementation failure while setting the ResponsePayload-Type.");
    }
}

template <typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader,
          ServiceType S>
inline auto set_request_header_type_details(
    ServiceBuilderRequestResponse<RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader, S>& builder,
    const TypeDetail& value) -> std::enable_if_t<std::is_same<RequestUserHeader, CustomHeaderMarker>::value> {
    builder.m_request_header_type_details_override = bb::Optional<TypeDetail>(value);
}

template <typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader,
          ServiceType S>
inline auto set_response_header_type_details(
    ServiceBuilderRequestResponse<RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader, S>& builder,
    const TypeDetail& value) -> std::enable_if_t<std::is_same<ResponseUserHeader, CustomHeaderMarker>::value> {
    builder.m_response_header_type_details_override = bb::Optional<TypeDetail>(value);
}

template <typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader,
          ServiceType S>
inline auto set_request_payload_type_details(
    ServiceBuilderRequestResponse<RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader, S>& builder,
    const TypeDetail& value) -> std::enable_if_t<std::is_same<RequestPayload, bb::Slice<CustomPayloadMarker>>::value> {
    builder.m_request_payload_type_details_override = bb::Optional<TypeDetail>(value);
}

template <typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader,
          ServiceType S>
inline auto set_response_payload_type_details(
    ServiceBuilderRequestResponse<RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader, S>& builder,
    const TypeDetail& value) -> std::enable_if_t<std::is_same<ResponsePayload, bb::Slice<CustomPayloadMarker>>::value> {
    builder.m_response_payload_type_details_override = bb::Optional<TypeDetail>(value);
}
} // namespace iox2
#endif
