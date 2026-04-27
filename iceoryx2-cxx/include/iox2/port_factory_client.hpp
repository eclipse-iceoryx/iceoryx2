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

#include "iox2/bb/detail/builder.hpp"
#include "iox2/bb/expected.hpp"
#include "iox2/bb/optional.hpp"
#include "iox2/client.hpp"
#include "iox2/client_error.hpp"
#include "iox2/degradation_handler.hpp"
#include "iox2/internal/callback_context.hpp"
#include "iox2/internal/iceoryx2.hpp"
#include "iox2/service_type.hpp"
#include "iox2/unable_to_deliver_handler.hpp"
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
    template <typename T = RequestPayload, typename = std::enable_if_t<bb::IsSlice<T>::VALUE, void>>
    auto initial_max_slice_len(uint64_t value) && -> PortFactoryClient&&;

    /// Defines the allocation strategy that is used when the provided
    /// [`PortFactoryClient::initial_max_slice_len()`] is exhausted. This happens when the user
    /// acquires more than max slice len in [`Client::loan_slice()`] or
    /// [`Client::loan_slice_uninit()`].
    template <typename T = RequestPayload, typename = std::enable_if_t<bb::IsSlice<T>::VALUE, void>>
    auto allocation_strategy(AllocationStrategy value) && -> PortFactoryClient&&;

    /// Defines a callback to reduce the number of preallocated [`RequestMut`]s.
    /// The input argument is the worst case number of preallocated [`RequestMut`]s required
    /// to guarantee that the [`Client`] never runs out of [`RequestMut`]s to loan
    /// and send.
    /// The return value is clamped between `1` and the worst case number of
    /// preallocated [`RequestMut`]s.
    ///
    /// # Important
    ///
    /// If the user reduces the number of preallocated [`RequestMut`]s, iceoryx2 can
    /// no longer guarantee, that the [`Client`] can always loan a [`RequestMut`]
    /// to send.
    auto override_request_preallocation(const OverridePreallocationCallback& callback) && -> PortFactoryClient&&;

    /// Sets the [`DegradationHandler`] for sending [`RequestMut`] from the [`Client`]. Whenever a request connection
    /// to a [`Server`] is corrupted, this handler is called and depending on the returned [`DegradationAction`]
    /// measures will be taken.
    /// @attention The handler function needs to live as long as the generated client. If the [`Client`], including
    /// the send and receive functions, is accessed from multiple threads, the handler must be thread-safe if it
    /// captures data
    auto set_request_degradation_handler(DegradationHandler* handler) && -> PortFactoryClient&&;

    /// Sets the [`DegradationHandler`] for receiving [`Response`]s from a [`Server`]. Whenever a response connection
    /// to a [`Server`] is corrupted, this handler is called and depending on the returned [`DegradationAction`]
    /// measures will be taken.
    /// @attention The handler function needs to live as long as the generated client. If the [`Client`], including
    /// the send and receive functions, is accessed from multiple threads, the handler must be thread-safe if it
    /// captures data
    auto set_response_degradation_handler(DegradationHandler* handler) && -> PortFactoryClient&&;

    /// Sets the [`UnableToDeliverHandler`] of the [`Client`]. Whenever a [`RequestMut`] cannot be sent to a [`Server`],
    /// this handler is called and depending on the returned [`UnableToDeliverAction`], measures will be taken.
    /// If no handler is set, the measures will be determined by the value set in [`UnableToDeliverStrategy`].
    /// @attention The handler function needs to live as long as the generated client. If the [`Client`], including
    /// the send and receive functions, is accessed from multiple threads, the handler must be thread-safe if it
    /// captures data
    auto set_unable_to_deliver_handler(UnableToDeliverHandler* handler) && -> PortFactoryClient&&;

    /// Creates a new [`Client`] or returns a [`ClientCreateError`] on failure.
    auto
    create() && -> bb::Expected<Client<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>,
                                ClientCreateError>;

  private:
    template <ServiceType, typename, typename, typename, typename>
    friend class PortFactoryRequestResponse;

    explicit PortFactoryClient(iox2_port_factory_client_builder_h handle);

    iox2_port_factory_client_builder_h m_handle = nullptr;
    bb::Optional<uint64_t> m_max_slice_len;
    bb::Optional<AllocationStrategy> m_allocation_strategy;
    bb::Optional<OverridePreallocationCallback> m_override_preallocation_callback;
    bb::Optional<DegradationHandler* const> m_request_degradation_handler;
    bb::Optional<DegradationHandler* const> m_response_degradation_handler;
    bb::Optional<UnableToDeliverHandler* const> m_unable_to_deliver_handler;
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
inline auto PortFactoryClient<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>::
    override_request_preallocation(const OverridePreallocationCallback& callback) && -> PortFactoryClient&& {
    m_override_preallocation_callback.emplace(callback);
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
    set_request_degradation_handler(DegradationHandler* handler) && -> PortFactoryClient&& {
    m_request_degradation_handler.emplace(handler);
    return std::move(*this);
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
inline auto PortFactoryClient<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>::
    set_response_degradation_handler(DegradationHandler* handler) && -> PortFactoryClient&& {
    m_response_degradation_handler.emplace(handler);
    return std::move(*this);
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
inline auto PortFactoryClient<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>::
    set_unable_to_deliver_handler(UnableToDeliverHandler* handler) && -> PortFactoryClient&& {
    m_unable_to_deliver_handler.emplace(handler);
    return std::move(*this);
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
inline auto PortFactoryClient<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>::
    create() && -> bb::Expected<Client<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>,
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
    if (m_override_preallocation_callback.has_value()) {
        // NOLINTNEXTLINE(cppcoreguidelines-owning-memory) must be a raw pointer - crosses FFI boundary
        auto* callback = new OverridePreallocationCallback(m_override_preallocation_callback.value());
        // NOLINTNEXTLINE(cppcoreguidelines-owning-memory) must be a raw pointer - crosses FFI boundary
        auto* ctx = new internal::CallbackContext<OverridePreallocationCallback>(*callback);
        iox2_port_factory_client_builder_override_requests_preallocation(
            &m_handle, internal::override_callback, static_cast<void*>(ctx));
    }

    if (m_request_degradation_handler.has_value()) {
        iox2_port_factory_client_builder_set_request_degradation_handler(
            &m_handle, detail::degradation_handler_delegate, static_cast<void*>(m_request_degradation_handler.value()));
    }

    if (m_response_degradation_handler.has_value()) {
        iox2_port_factory_client_builder_set_response_degradation_handler(
            &m_handle,
            detail::degradation_handler_delegate,
            static_cast<void*>(m_response_degradation_handler.value()));
    }

    if (m_unable_to_deliver_handler.has_value()) {
        iox2_port_factory_client_builder_set_unable_to_deliver_handler(
            &m_handle,
            detail::unable_to_deliver_handler_delegate,
            static_cast<void*>(m_unable_to_deliver_handler.value()));
    }

    iox2_client_h client_handle {};
    auto result = iox2_port_factory_client_builder_create(m_handle, nullptr, &client_handle);

    if (result == IOX2_OK) {
        return Client<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>(client_handle);
    }

    return bb::err(bb::into<ClientCreateError>(result));
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
