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
#include "iox2/client.hpp"
#include "iox2/client_error.hpp"
#include "iox2/container/optional.hpp"
#include "iox2/legacy/expected.hpp"
#include "iox2/service_type.hpp"
#include "iox2/unable_to_deliver_strategy.hpp"

namespace iox2 {
/// Factory to create a new [`Client`] port/endpoint for
/// [`MessagingPattern::RequestResponse`] based communication.
template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
class PortFactoryClient {
  public:
    /// Sets the [`UnableToDeliverStrategy`] which defines how the [`Client`] shall behave
    /// when a [`Server`] cannot receive a [`RequestMut`] since
    /// its internal buffer is full.
#ifdef DOXYGEN_MACRO_FIX
    auto unable_to_deliver_strategy(const UnableToDeliverStrategy value) -> decltype(auto);
#else
    IOX2_BUILDER_OPTIONAL(UnableToDeliverStrategy, unable_to_deliver_strategy);
#endif

  public:
    PortFactoryClient(const PortFactoryClient&) = delete;
    PortFactoryClient(PortFactoryClient&&) = default;
    auto operator=(const PortFactoryClient&) -> PortFactoryClient& = delete;
    auto operator=(PortFactoryClient&&) -> PortFactoryClient& = default;
    ~PortFactoryClient() = default;

    /// Sets the maximum number of elements that can be loaned in a slice.
    template <typename T = RequestPayload, typename = std::enable_if_t<iox::IsSlice<T>::VALUE, void>>
    auto initial_max_slice_len(uint64_t value) && -> PortFactoryClient&&;

    /// Defines the allocation strategy that is used when the provided
    /// [`PortFactoryClient::initial_max_slice_len()`] is exhausted. This happens when the user
    /// acquires more than max slice len in [`Client::loan_slice()`] or
    /// [`Client::loan_slice_uninit()`].
    template <typename T = RequestPayload, typename = std::enable_if_t<iox::IsSlice<T>::VALUE, void>>
    auto allocation_strategy(AllocationStrategy value) && -> PortFactoryClient&&;

    /// Creates a new [`Client`] or returns a [`ClientCreateError`] on failure.
    auto create() && -> iox2::legacy::expected<
        Client<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>,
        ClientCreateError>;

  private:
    template <ServiceType, typename, typename, typename, typename>
    friend class PortFactoryRequestResponse;

    explicit PortFactoryClient(iox2_port_factory_client_builder_h handle);

    iox2_port_factory_client_builder_h m_handle = nullptr;
    container::Optional<uint64_t> m_max_slice_len;
    container::Optional<AllocationStrategy> m_allocation_strategy;
};

template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
template <typename T, typename>
inline auto PortFactoryClient<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>::
    initial_max_slice_len(uint64_t value) && -> PortFactoryClient&& {
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
PortFactoryClient<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>::allocation_strategy(
    AllocationStrategy value) && -> PortFactoryClient&& {
    m_allocation_strategy.emplace(value);
    return std::move(*this);
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
inline auto PortFactoryClient<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>::
    create() && -> iox2::legacy::expected<
        Client<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>,
        ClientCreateError> {
    if (m_unable_to_deliver_strategy.has_value()) {
        iox2_port_factory_client_builder_unable_to_deliver_strategy(
            &m_handle,
            static_cast<iox2_unable_to_deliver_strategy_e>(bb::into<int>(m_unable_to_deliver_strategy.value())));
    }
    if (m_max_slice_len.has_value()) {
        iox2_port_factory_client_builder_set_initial_max_slice_len(&m_handle, m_max_slice_len.value());
    } else {
        iox2_port_factory_client_builder_set_initial_max_slice_len(&m_handle, 1);
    }
    if (m_allocation_strategy.has_value()) {
        iox2_port_factory_client_builder_set_allocation_strategy(
            &m_handle, bb::into<iox2_allocation_strategy_e>(m_allocation_strategy.value()));
    }

    iox2_client_h client_handle {};
    auto result = iox2_port_factory_client_builder_create(m_handle, nullptr, &client_handle);

    if (result == IOX2_OK) {
        return iox2::legacy::ok(
            Client<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>(client_handle));
    }

    return iox2::legacy::err(iox2::bb::into<ClientCreateError>(result));
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
inline PortFactoryClient<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>::
    PortFactoryClient(iox2_port_factory_client_builder_h handle)
    : m_handle { handle } {
}
} // namespace iox2
#endif
