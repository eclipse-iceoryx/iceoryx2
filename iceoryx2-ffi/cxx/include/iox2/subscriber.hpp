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
#include "iox2/iceoryx2.h"
#include "iox2/internal/iceoryx2.hpp"
#include "iox2/sample.hpp"
#include "iox2/service_type.hpp"
#include "iox2/subscriber_error.hpp"
#include "iox2/unique_port_id.hpp"

namespace iox2 {
/// The receiving endpoint of a publish-subscribe communication.
template <ServiceType S, typename Payload, typename UserHeader>
class Subscriber {
  public:
    Subscriber(Subscriber&& rhs) noexcept;
    auto operator=(Subscriber&& rhs) noexcept -> Subscriber&;
    ~Subscriber();

    Subscriber(const Subscriber&) = delete;
    auto operator=(const Subscriber&) -> Subscriber& = delete;

    /// Returns the [`UniqueSubscriberId`] of the [`Subscriber`]
    auto id() const -> UniqueSubscriberId;

    /// Returns the internal buffer size of the [`Subscriber`].
    auto buffer_size() const -> uint64_t;

    /// Receives a [`Sample`] from [`Publisher`]. If no sample could be
    /// received [`None`] is returned. If a failure occurs [`ReceiveError`] is returned.
    auto receive() const -> iox::expected<iox::optional<Sample<S, Payload, UserHeader>>, ReceiveError>;

    /// Returns true when the [`Subscriber`] has [`Sample`]s that can be
    /// acquired via [`Subscriber::receive()`], otherwise false.
    auto has_samples() const -> iox::expected<bool, ConnectionFailure>;

  private:
    template <ServiceType, typename, typename>
    friend class PortFactorySubscriber;

    explicit Subscriber(iox2_subscriber_h handle);
    void drop();

    iox2_subscriber_h m_handle = nullptr;
};
template <ServiceType S, typename Payload, typename UserHeader>
inline Subscriber<S, Payload, UserHeader>::Subscriber(iox2_subscriber_h handle)
    : m_handle { handle } {
}

template <ServiceType S, typename Payload, typename UserHeader>
inline Subscriber<S, Payload, UserHeader>::Subscriber(Subscriber&& rhs) noexcept {
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
inline auto Subscriber<S, Payload, UserHeader>::has_samples() const -> iox::expected<bool, ConnectionFailure> {
    bool has_samples_result = false;
    auto result = iox2_subscriber_has_samples(&m_handle, &has_samples_result);

    if (result == IOX2_OK) {
        return iox::ok(has_samples_result);
    }

    return iox::err(iox::into<ConnectionFailure>(result));
}

template <ServiceType S, typename Payload, typename UserHeader>
inline auto Subscriber<S, Payload, UserHeader>::id() const -> UniqueSubscriberId {
    iox2_unique_subscriber_id_h id_handle = nullptr;

    iox2_subscriber_id(&m_handle, nullptr, &id_handle);
    return UniqueSubscriberId { id_handle };
}

template <ServiceType S, typename Payload, typename UserHeader>
inline auto Subscriber<S, Payload, UserHeader>::buffer_size() const -> uint64_t {
    return iox2_subscriber_buffer_size(&m_handle);
}

template <ServiceType S, typename Payload, typename UserHeader>
inline auto Subscriber<S, Payload, UserHeader>::receive() const
    -> iox::expected<iox::optional<Sample<S, Payload, UserHeader>>, ReceiveError> {
    Sample<S, Payload, UserHeader> sample;
    auto result = iox2_subscriber_receive(&m_handle, &sample.m_sample, &sample.m_handle);

    if (result == IOX2_OK) {
        if (sample.m_handle != nullptr) {
            return iox::ok(iox::optional<Sample<S, Payload, UserHeader>>(std::move(sample)));
        }
        return iox::ok(iox::optional<Sample<S, Payload, UserHeader>>(iox::nullopt));
    }

    return iox::err(iox::into<ReceiveError>(result));
}
} // namespace iox2

#endif
