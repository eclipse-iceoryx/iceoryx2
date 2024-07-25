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
#include "iox2/publisher.hpp"
#include "iox2/service_type.hpp"

#include <cstdint>

namespace iox2 {
enum class UnableToDeliverStrategy : uint8_t {
    Block,
    DiscardSample
};

template <ServiceType S, typename Payload, typename UserHeader>
class PortFactoryPublisher {
    IOX_BUILDER_OPTIONAL(UnableToDeliverStrategy, unable_to_deliver_strategy);
    IOX_BUILDER_OPTIONAL(uint64_t, max_loaned_samples);
    IOX_BUILDER_OPTIONAL(uint64_t, max_slice_len);

  public:
    PortFactoryPublisher(const PortFactoryPublisher&) = delete;
    PortFactoryPublisher(PortFactoryPublisher&&) = default;
    auto operator=(const PortFactoryPublisher&) -> PortFactoryPublisher& = delete;
    auto operator=(PortFactoryPublisher&&) -> PortFactoryPublisher& = default;
    ~PortFactoryPublisher() = default;

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
    auto* ref_handle = iox2_cast_port_factory_publisher_builder_ref_h(m_handle);

    m_unable_to_deliver_strategy.and_then([](auto) { IOX_TODO(); });
    m_max_slice_len.and_then([](auto) { IOX_TODO(); });
    m_max_loaned_samples.and_then(
        [&](auto value) { iox2_port_factory_publisher_builder_set_max_loaned_samples(ref_handle, value); });

    IOX_TODO();
}
} // namespace iox2

#endif
