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

#include "iox/assertions_addendum.hpp"
#include "iox/builder_addendum.hpp"
#include "iox/expected.hpp"
#include "iox2/internal/iceoryx2.hpp"
#include "iox2/payload_info.hpp"
#include "iox2/publisher.hpp"
#include "iox2/service_type.hpp"
#include "iox2/unable_to_deliver_strategy.hpp"

#include <cstdint>

namespace iox2 {
/// Factory to create a new [`Publisher`] port/endpoint for
/// [`MessagingPattern::PublishSubscribe`] based communication.
template <ServiceType S, typename Payload, typename UserHeader>
class PortFactoryPublisher {
    /// Sets the [`UnableToDeliverStrategy`].
    IOX_BUILDER_OPTIONAL(UnableToDeliverStrategy, unable_to_deliver_strategy);

    /// Defines how many [`SampleMut`] the [`Publisher`] can loan with
    /// [`Publisher::loan()`] or [`Publisher::loan_uninit()`] in parallel.
    IOX_BUILDER_OPTIONAL(uint64_t, max_loaned_samples);

    /// Sets the maximum slice length that a user can allocate with
    /// [`Publisher::loan_slice()`] or [`Publisher::loan_slice_uninit()`].
    IOX_BUILDER_OPTIONAL(uint64_t, max_slice_len);

  public:
    PortFactoryPublisher(const PortFactoryPublisher&) = delete;
    PortFactoryPublisher(PortFactoryPublisher&&) = default;
    auto operator=(const PortFactoryPublisher&) -> PortFactoryPublisher& = delete;
    auto operator=(PortFactoryPublisher&&) -> PortFactoryPublisher& = default;
    ~PortFactoryPublisher() = default;

    /// Creates a new [`Publisher`] or returns a [`PublisherCreateError`] on failure.
    auto create() && -> iox::expected<Publisher<S, Payload, UserHeader>, PublisherCreateError>;

  private:
    template <ServiceType, typename, typename>
    friend class PortFactoryPublishSubscribe;

    explicit PortFactoryPublisher(iox2_port_factory_publisher_builder_h handle);

    iox2_port_factory_publisher_builder_h m_handle;
};

template <ServiceType S, typename Payload, typename UserHeader>
inline PortFactoryPublisher<S, Payload, UserHeader>::PortFactoryPublisher(iox2_port_factory_publisher_builder_h handle)
    : m_handle { handle } {
}

template <ServiceType S, typename Payload, typename UserHeader>
inline auto
PortFactoryPublisher<S, Payload, UserHeader>::create() && -> iox::expected<Publisher<S, Payload, UserHeader>,
                                                                           PublisherCreateError> {
    m_unable_to_deliver_strategy.and_then([&](auto value) {
        iox2_port_factory_publisher_builder_unable_to_deliver_strategy(
            &m_handle, static_cast<iox2_unable_to_deliver_strategy_e>(iox::into<int>(value)));
    });
    m_max_slice_len
        .and_then([&](auto value) { iox2_port_factory_publisher_builder_set_max_slice_len(&m_handle, value); })
        .or_else([&]() { iox2_port_factory_publisher_builder_set_max_slice_len(&m_handle, 1); });
    m_max_loaned_samples.and_then(
        [&](auto value) { iox2_port_factory_publisher_builder_set_max_loaned_samples(&m_handle, value); });

    iox2_publisher_h pub_handle {};

    auto result = iox2_port_factory_publisher_builder_create(m_handle, nullptr, &pub_handle);

    if (result == IOX2_OK) {
        return iox::ok(Publisher<S, Payload, UserHeader>(pub_handle));
    }

    return iox::err(iox::into<PublisherCreateError>(result));
}
} // namespace iox2

#endif
