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

#ifndef IOX2_PORTFACTORY_SUBSCRIBER_HPP
#define IOX2_PORTFACTORY_SUBSCRIBER_HPP

#include "iox/builder_addendum.hpp"
#include "iox/expected.hpp"
#include "iox2/internal/iceoryx2.hpp"
#include "iox2/service_type.hpp"
#include "iox2/subscriber.hpp"

#include <cstdint>

namespace iox2 {

/// Factory to create a new [`Subscriber`] port/endpoint for
/// [`MessagingPattern::PublishSubscribe`] based communication.
template <ServiceType S, typename Payload, typename UserHeader>
class PortFactorySubscriber {
  public:
    /// Defines the required buffer size of the [`Subscriber`]. Smallest possible value is `1`.
#ifdef DOXYGEN_MACRO_FIX
    auto buffer_size(const uint64_t value) -> decltype(auto);
#else
    IOX_BUILDER_OPTIONAL(uint64_t, buffer_size);
#endif

  public:
    PortFactorySubscriber(const PortFactorySubscriber&) = delete;
    PortFactorySubscriber(PortFactorySubscriber&&) = default;
    auto operator=(const PortFactorySubscriber&) -> PortFactorySubscriber& = delete;
    auto operator=(PortFactorySubscriber&&) -> PortFactorySubscriber& = default;
    ~PortFactorySubscriber() = default;

    /// Creates a new [`Subscriber`] or returns a [`SubscriberCreateError`] on failure.
    auto create() && -> iox::expected<Subscriber<S, Payload, UserHeader>, SubscriberCreateError>;

  private:
    template <ServiceType, typename, typename>
    friend class PortFactoryPublishSubscribe;

    explicit PortFactorySubscriber(iox2_port_factory_subscriber_builder_h handle);

    iox2_port_factory_subscriber_builder_h m_handle = nullptr;
};

template <ServiceType S, typename Payload, typename UserHeader>
inline PortFactorySubscriber<S, Payload, UserHeader>::PortFactorySubscriber(
    iox2_port_factory_subscriber_builder_h handle)
    : m_handle { handle } {
}

template <ServiceType S, typename Payload, typename UserHeader>
inline auto
PortFactorySubscriber<S, Payload, UserHeader>::create() && -> iox::expected<Subscriber<S, Payload, UserHeader>,
                                                                            SubscriberCreateError> {
    m_buffer_size.and_then([&](auto value) { iox2_port_factory_subscriber_builder_set_buffer_size(&m_handle, value); });

    iox2_subscriber_h sub_handle {};
    auto result = iox2_port_factory_subscriber_builder_create(m_handle, nullptr, &sub_handle);

    if (result == IOX2_OK) {
        return iox::ok(Subscriber<S, Payload, UserHeader>(sub_handle));
    }

    return iox::err(iox::into<SubscriberCreateError>(result));
}
} // namespace iox2

#endif
