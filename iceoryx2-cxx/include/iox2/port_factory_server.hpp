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

#ifndef IOX2_PORTFACTORY_SERVER_HPP
#define IOX2_PORTFACTORY_SERVER_HPP

#include "iox/builder_addendum.hpp"
#include "iox/expected.hpp"
#include "iox2/server.hpp"
#include "iox2/server_error.hpp"
#include "iox2/service_type.hpp"
#include "iox2/unable_to_deliver_strategy.hpp"

namespace iox2 {
/// Factory to create a new [`Server`] port/endpoint for
/// [`MessagingPattern::RequestResponse`]
/// based communication.
template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
class PortFactoryServer {
  public:
    /// Sets the [`UnableToDeliverStrategy`] which defines how the [`Server`] shall behave
    /// when a [`Client`] cannot receive a [`Response`] since
    /// its internal buffer is full.
#ifdef DOXYGEN_MACRO_FIX
    auto unable_to_deliver_strategy(const UnableToDeliverStrategy value) -> decltype(auto);
#else
    IOX_BUILDER_OPTIONAL(UnableToDeliverStrategy, unable_to_deliver_strategy);
#endif

    /// Defines the maximum number of [`ResponseMut`] that the [`Server`] can
    /// loan in parallel per [`ActiveRequest`].
#ifdef DOXYGEN_MACRO_FIX
    auto max_loaned_responses_per_request(const uint64_t value) -> decltype(auto);
#else
    IOX_BUILDER_OPTIONAL(uint64_t, max_loaned_responses_per_request);
#endif

  public:
    PortFactoryServer(const PortFactoryServer&) = delete;
    PortFactoryServer(PortFactoryServer&&) = default;
    auto operator=(const PortFactoryServer&) -> PortFactoryServer& = delete;
    auto operator=(PortFactoryServer&&) -> PortFactoryServer& = default;
    ~PortFactoryServer() = default;

    /// Sets the maximum initial slice length configured for this [`Server`].
    template <typename T = ResponsePayload, typename = std::enable_if_t<iox::IsSlice<T>::VALUE, void>>
    auto initial_max_slice_len(uint64_t value) && -> PortFactoryServer&&;

    /// Defines the allocation strategy that is used when the provided
    /// [`PortFactoryServer::initial_max_slice_len()`] is exhausted. This happens when the user
    /// acquires more than max slice len in [`ActiveRequest::loan_slice()`] or
    /// [`ActiveRequest::loan_slice_uninit()`].
    template <typename T = ResponsePayload, typename = std::enable_if_t<iox::IsSlice<T>::VALUE, void>>
    auto allocation_strategy(AllocationStrategy value) && -> PortFactoryServer&&;

    /// Creates a new [`Server`] or returns a [`ServerCreateError`] on failure.
    auto create() && -> iox::expected<
        Server<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>,
        ServerCreateError>;

  private:
    template <ServiceType, typename, typename, typename, typename>
    friend class PortFactoryRequestResponse;

    explicit PortFactoryServer(iox2_port_factory_server_builder_h handle);

    iox2_port_factory_server_builder_h m_handle = nullptr;
    iox::optional<uint64_t> m_max_slice_len;
    iox::optional<AllocationStrategy> m_allocation_strategy;
};

template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
template <typename T, typename>
inline auto PortFactoryServer<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>::
    initial_max_slice_len(uint64_t value) && -> PortFactoryServer&& {
    m_max_slice_len.emplace(value);
    return std::move(*this);
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
template <typename T, typename>
inline auto
PortFactoryServer<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>::allocation_strategy(
    AllocationStrategy value) && -> PortFactoryServer&& {
    m_allocation_strategy.emplace(value);
    return std::move(*this);
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
inline auto
PortFactoryServer<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>::create() && -> iox::
    expected<Server<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>,
             ServerCreateError> {
    m_unable_to_deliver_strategy.and_then([&](auto value) {
        iox2_port_factory_server_builder_unable_to_deliver_strategy(
            &m_handle, static_cast<iox2_unable_to_deliver_strategy_e>(iox::into<int>(value)));
    });
    m_max_slice_len
        .and_then([&](auto value) { iox2_port_factory_server_builder_set_initial_max_slice_len(&m_handle, value); })
        .or_else([&]() { iox2_port_factory_server_builder_set_initial_max_slice_len(&m_handle, 1); });
    m_max_loaned_responses_per_request.and_then(
        [&](auto value) { iox2_port_factory_server_builder_set_max_loaned_responses_per_request(&m_handle, value); });
    m_allocation_strategy.and_then([&](auto value) {
        iox2_port_factory_server_builder_set_allocation_strategy(&m_handle,
                                                                 iox::into<iox2_allocation_strategy_e>(value));
    });

    iox2_server_h server_handle {};
    auto result = iox2_port_factory_server_builder_create(m_handle, nullptr, &server_handle);

    if (result == IOX2_OK) {
        return iox::ok(
            Server<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>(server_handle));
    }

    return iox::err(iox::into<ServerCreateError>(result));
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
inline PortFactoryServer<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>::
    PortFactoryServer(iox2_port_factory_server_builder_h handle)
    : m_handle { handle } {
}
} // namespace iox2
#endif
