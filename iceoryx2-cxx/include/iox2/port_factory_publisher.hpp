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

#ifndef IOX2_PORTFACTORY_PUBLISHER_HPP
#define IOX2_PORTFACTORY_PUBLISHER_HPP

#include "iox2/allocation_strategy.hpp"
#include "iox2/bb/detail/builder.hpp"
#include "iox2/bb/expected.hpp"
#include "iox2/bb/optional.hpp"
#include "iox2/internal/callback_context.hpp"
#include "iox2/internal/iceoryx2.hpp"
#include "iox2/publisher.hpp"
#include "iox2/service_type.hpp"
#include "iox2/unable_to_deliver_strategy.hpp"

#include <cstdint>

namespace iox2 {
/// Factory to create a new [`Publisher`] port/endpoint for
/// [`MessagingPattern::PublishSubscribe`] based communication.
template <ServiceType S, typename Payload, typename UserHeader>
class PortFactoryPublisher {
  public:
    /// Sets the [`UnableToDeliverStrategy`].
#ifdef DOXYGEN_MACRO_FIX
    auto unable_to_deliver_strategy(const UnableToDeliverStrategy value) -> decltype(auto);
#else
    IOX2_BUILDER_OPTIONAL(UnableToDeliverStrategy, unable_to_deliver_strategy);
#endif

    /// Defines how many [`SampleMut`] the [`Publisher`] can loan with
    /// [`Publisher::loan()`] or [`Publisher::loan_uninit()`] in parallel.
#ifdef DOXYGEN_MACRO_FIX
    auto max_loaned_samples(const uint64_t value) -> decltype(auto);
#else
    IOX2_BUILDER_OPTIONAL(uint64_t, max_loaned_samples);
#endif

  public:
    PortFactoryPublisher(const PortFactoryPublisher&) = delete;
    PortFactoryPublisher(PortFactoryPublisher&&) = default;
    auto operator=(const PortFactoryPublisher&) -> PortFactoryPublisher& = delete;
    auto operator=(PortFactoryPublisher&&) -> PortFactoryPublisher& = default;
    ~PortFactoryPublisher() = default;

    /// Defines a callback to reduce the number of preallocated [`SampleMut`]s.
    /// The input argument is the worst case number of preallocated [`SampleMut`]s required
    /// to guarantee that the [`Publisher`] never runs out of [`SampleMut`]s to loan
    /// and send.
    /// The return value is clamped between `1` and the worst case number of
    /// preallocated [`SampleMut`]s.
    ///
    /// # Important
    ///
    /// If the user reduces the number of preallocated [`SampleMut`]s, iceoryx2 can
    /// no longer guarantee, that the [`Publisher`] can always loan a [`SampleMut`]
    /// to send.
    auto override_sample_preallocation(
        const iox2::bb::StaticFunction<size_t(size_t)>& callback) && -> PortFactoryPublisher&&;

    /// Sets the maximum slice length that a user can allocate with
    /// [`Publisher::loan_slice()`] or [`Publisher::loan_slice_uninit()`].
    template <typename T = Payload, typename = std::enable_if_t<bb::IsSlice<T>::VALUE, void>>
    auto initial_max_slice_len(uint64_t value) && -> PortFactoryPublisher&&;

    /// Defines the allocation strategy that is used when the provided
    /// [`PortFactoryPublisher::initial_max_slice_len()`] is exhausted. This happens when the user
    /// acquires a more than max slice len in [`Publisher::loan_slice()`] or
    /// [`Publisher::loan_slice_uninit()`].
    template <typename T = Payload, typename = std::enable_if_t<bb::IsSlice<T>::VALUE, void>>
    auto allocation_strategy(AllocationStrategy value) && -> PortFactoryPublisher&&;

    /// Creates a new [`Publisher`] or returns a [`PublisherCreateError`] on failure.
    auto create() && -> bb::Expected<Publisher<S, Payload, UserHeader>, PublisherCreateError>;

  private:
    template <ServiceType, typename, typename>
    friend class PortFactoryPublishSubscribe;

    explicit PortFactoryPublisher(iox2_port_factory_publisher_builder_h handle);

    iox2_port_factory_publisher_builder_h m_handle = nullptr;
    bb::Optional<uint64_t> m_max_slice_len;
    bb::Optional<AllocationStrategy> m_allocation_strategy;
    bb::Optional<iox2::bb::StaticFunction<size_t(size_t)>> m_override_preallocation_callback;
};

template <ServiceType S, typename Payload, typename UserHeader>
inline PortFactoryPublisher<S, Payload, UserHeader>::PortFactoryPublisher(iox2_port_factory_publisher_builder_h handle)
    : m_handle { handle } {
}

template <ServiceType S, typename Payload, typename UserHeader>
template <typename T, typename>
inline auto
PortFactoryPublisher<S, Payload, UserHeader>::initial_max_slice_len(uint64_t value) && -> PortFactoryPublisher&& {
    m_max_slice_len.emplace(value);
    return std::move(*this);
}

template <ServiceType S, typename Payload, typename UserHeader>
inline auto PortFactoryPublisher<S, Payload, UserHeader>::override_sample_preallocation(
    const iox2::bb::StaticFunction<size_t(size_t)>& callback) && -> PortFactoryPublisher&& {
    m_override_preallocation_callback.emplace(callback);
    return std::move(*this);
}

template <ServiceType S, typename Payload, typename UserHeader>
template <typename T, typename>
inline auto PortFactoryPublisher<S, Payload, UserHeader>::allocation_strategy(
    AllocationStrategy value) && -> PortFactoryPublisher&& {
    m_allocation_strategy.emplace(value);
    return std::move(*this);
}

template <ServiceType S, typename Payload, typename UserHeader>
inline auto PortFactoryPublisher<S, Payload, UserHeader>::create() && -> bb::Expected<Publisher<S, Payload, UserHeader>,
                                                                                      PublisherCreateError> {
    if (m_unable_to_deliver_strategy.has_value()) {
        iox2_port_factory_publisher_builder_unable_to_deliver_strategy(
            &m_handle,
            static_cast<iox2_unable_to_deliver_strategy_e>(bb::into<int>(m_unable_to_deliver_strategy.value())));
    }
    if (m_max_slice_len.has_value()) {
        iox2_port_factory_publisher_builder_set_initial_max_slice_len(&m_handle, m_max_slice_len.value());
    } else {
        iox2_port_factory_publisher_builder_set_initial_max_slice_len(&m_handle, 1);
    }
    if (m_max_loaned_samples.has_value()) {
        iox2_port_factory_publisher_builder_set_max_loaned_samples(&m_handle, m_max_loaned_samples.value());
    }
    if (m_allocation_strategy.has_value()) {
        iox2_port_factory_publisher_builder_set_allocation_strategy(
            &m_handle, bb::into<iox2_allocation_strategy_e>(m_allocation_strategy.value()));
    }
    if (m_override_preallocation_callback.has_value()) {
        // NOLINTNEXTLINE(cppcoreguidelines-owning-memory) must be a raw pointer - crosses FFI boundary
        auto* callback = new iox2::bb::StaticFunction<size_t(size_t)>(m_override_preallocation_callback.value());
        // NOLINTNEXTLINE(cppcoreguidelines-owning-memory) must be a raw pointer - crosses FFI boundary
        auto* ctx = new internal::CallbackContext<iox2::bb::StaticFunction<size_t(size_t)>*>(callback);
        iox2_port_factory_publisher_builder_override_samples_preallocation(
            &m_handle, internal::override_callback, static_cast<void*>(ctx));
    }


    iox2_publisher_h pub_handle {};

    auto result = iox2_port_factory_publisher_builder_create(m_handle, nullptr, &pub_handle);

    if (result == IOX2_OK) {
        return Publisher<S, Payload, UserHeader>(pub_handle);
    }

    return bb::err(bb::into<PublisherCreateError>(result));
}
} // namespace iox2

#endif
