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

#ifndef IOX2_SUBSCRIBER_HPP
#define IOX2_SUBSCRIBER_HPP

#include "iox/assertions_addendum.hpp"
#include "iox/expected.hpp"
#include "iox/optional.hpp"
#include "iox2/connection_failure.hpp"
#include "iox2/internal/iceoryx2.hpp"
#include "iox2/sample.hpp"
#include "iox2/service_type.hpp"
#include "iox2/subscriber_error.hpp"
#include "iox2/unique_port_id.hpp"

namespace iox2 {
template <ServiceType S, typename Payload, typename UserHeader>
class Subscriber {
  public:
    Subscriber(Subscriber&&) noexcept;
    auto operator=(Subscriber&&) noexcept -> Subscriber&;
    ~Subscriber();

    Subscriber(const Subscriber&) = delete;
    auto operator=(const Subscriber&) -> Subscriber& = delete;

    auto id() const -> UniqueSubscriberId;
    auto buffer_size() const -> uint64_t;
    auto receive() const -> iox::expected<iox::optional<Sample<S, Payload, UserHeader>>, SubscriberReceiveError>;
    auto update_connections() const -> iox::expected<void, ConnectionFailure>;

  private:
    template <ServiceType, typename, typename>
    friend class PortFactorySubscriber;

    explicit Subscriber(iox2_subscriber_h handle);
    void drop();

    iox2_subscriber_h m_handle;
};
template <ServiceType S, typename Payload, typename UserHeader>
inline Subscriber<S, Payload, UserHeader>::Subscriber(iox2_subscriber_h handle)
    : m_handle { handle } {
}

template <ServiceType S, typename Payload, typename UserHeader>
inline Subscriber<S, Payload, UserHeader>::Subscriber(Subscriber&& rhs) noexcept
    : m_handle { nullptr } {
    *this = std::move(rhs);
}

template <ServiceType S, typename Payload, typename UserHeader>
inline auto Subscriber<S, Payload, UserHeader>::operator=(Subscriber&& rhs) noexcept -> Subscriber& {
    if (this != &rhs) {
        drop();
        m_handle = std::move(rhs.m_handle);
        rhs.m_handle = nullptr;
    }

    return *this;
}

template <ServiceType S, typename Payload, typename UserHeader>
inline Subscriber<S, Payload, UserHeader>::~Subscriber() {
    drop();
}

template <ServiceType S, typename Payload, typename UserHeader>
inline void Subscriber<S, Payload, UserHeader>::drop() {
    if (m_handle != nullptr) {
        iox2_subscriber_drop(m_handle);
        m_handle = nullptr;
    }
}

template <ServiceType S, typename Payload, typename UserHeader>
inline auto Subscriber<S, Payload, UserHeader>::id() const -> UniqueSubscriberId {
    IOX_TODO();
}

template <ServiceType S, typename Payload, typename UserHeader>
inline auto Subscriber<S, Payload, UserHeader>::buffer_size() const -> uint64_t {
    IOX_TODO();
}

template <ServiceType S, typename Payload, typename UserHeader>
inline auto Subscriber<S, Payload, UserHeader>::receive() const
    -> iox::expected<iox::optional<Sample<S, Payload, UserHeader>>, SubscriberReceiveError> {
    IOX_TODO();
}

template <ServiceType S, typename Payload, typename UserHeader>
inline auto Subscriber<S, Payload, UserHeader>::update_connections() const -> iox::expected<void, ConnectionFailure> {
    IOX_TODO();
}

} // namespace iox2

#endif
