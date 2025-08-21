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
    IOX_BUILDER_OPTIONAL(UnableToDeliverStrategy, unable_to_deliver_strategy);
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
    auto create() && -> iox::expected<
        Client<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>,
        ClientCreateError>;

  private:
    template <ServiceType, typename, typename, typename, typename>
    friend class PortFactoryRequestResponse;

    explicit PortFactoryClient(iox2_port_factory_client_builder_h handle);

    iox2_port_factory_client_builder_h m_handle = nullptr;
    iox::optional<uint64_t> m_max_slice_len;
    iox::optional<AllocationStrategy> m_allocation_strategy;
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
inline auto
PortFactoryClient<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>::create() && -> iox::
    expected<Client<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>,
             ClientCreateError> {
    m_unable_to_deliver_strategy.and_then([&](auto value) {
        iox2_port_factory_client_builder_unable_to_deliver_strategy(
            &m_handle, static_cast<iox2_unable_to_deliver_strategy_e>(iox::into<int>(value)));
    });
    m_max_slice_len
        .and_then([&](auto value) { iox2_port_factory_client_builder_set_initial_max_slice_len(&m_handle, value); })
        .or_else([&]() { iox2_port_factory_client_builder_set_initial_max_slice_len(&m_handle, 1); });
    m_allocation_strategy.and_then([&](auto value) {
        iox2_port_factory_client_builder_set_allocation_strategy(&m_handle,
                                                                 iox::into<iox2_allocation_strategy_e>(value));
    });

    iox2_client_h client_handle {};
    auto result = iox2_port_factory_client_builder_create(m_handle, nullptr, &client_handle);

    if (result == IOX2_OK) {
        return iox::ok(
            Client<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>(client_handle));
    }

    return iox::err(iox::into<ClientCreateError>(result));
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
