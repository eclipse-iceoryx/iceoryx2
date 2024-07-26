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

#ifndef IOX2_PUBLISHER_HPP
#define IOX2_PUBLISHER_HPP

#include "iox/assertions_addendum.hpp"
#include "iox/expected.hpp"
#include "iox2/connection_failure.hpp"
#include "iox2/iceoryx2.h"
#include "iox2/internal/iceoryx2.hpp"
#include "iox2/publisher_error.hpp"
#include "iox2/sample_mut.hpp"
#include "iox2/service_type.hpp"
#include "iox2/unique_port_id.hpp"

#include <cstdint>

namespace iox2 {
template <ServiceType S, typename Payload, typename UserHeader>
class Publisher {
  public:
    Publisher(Publisher&&) noexcept;
    auto operator=(Publisher&&) noexcept -> Publisher&;
    ~Publisher();

    Publisher(const Publisher&) = delete;
    auto operator=(const Publisher&) -> Publisher& = delete;

    auto id() const -> UniquePublisherId;
    auto send_copy(const Payload& payload) const -> iox::expected<uint64_t, PublisherSendError>;
    auto loan_uninit() -> iox::expected<SampleMut<S, Payload, UserHeader>, PublisherLoanError>;
    auto loan() -> iox::expected<SampleMut<S, Payload, UserHeader>, PublisherLoanError>;

    auto loan_slice(const uint64_t number_of_elements)
        -> iox::expected<SampleMut<S, Payload, UserHeader>, PublisherLoanError>;
    auto loan_slice_uninit(const uint64_t number_of_elements)
        -> iox::expected<SampleMut<S, Payload, UserHeader>, PublisherLoanError>;

    auto update_connections() -> iox::expected<void, ConnectionFailure>;

  private:
    template <ServiceType, typename, typename>
    friend class PortFactoryPublisher;

    explicit Publisher(iox2_publisher_h handle);
    void drop();

    iox2_publisher_h m_handle;
};

template <ServiceType S, typename Payload, typename UserHeader>
inline Publisher<S, Payload, UserHeader>::Publisher(iox2_publisher_h handle)
    : m_handle { handle } {
}

template <ServiceType S, typename Payload, typename UserHeader>
inline void Publisher<S, Payload, UserHeader>::drop() {
    if (m_handle != nullptr) {
        iox2_publisher_drop(m_handle);
        m_handle = nullptr;
    }
}

template <ServiceType S, typename Payload, typename UserHeader>
inline Publisher<S, Payload, UserHeader>::Publisher(Publisher&& rhs) noexcept
    : m_handle { nullptr } {
    *this = std::move(rhs);
}

template <ServiceType S, typename Payload, typename UserHeader>
inline auto Publisher<S, Payload, UserHeader>::operator=(Publisher&& rhs) noexcept -> Publisher& {
    if (this != &rhs) {
        drop();
        m_handle = std::move(rhs.m_handle);
        rhs.m_handle = nullptr;
    }

    return *this;
}

template <ServiceType S, typename Payload, typename UserHeader>
inline Publisher<S, Payload, UserHeader>::~Publisher() {
    drop();
}

template <ServiceType S, typename Payload, typename UserHeader>
inline auto Publisher<S, Payload, UserHeader>::id() const -> UniquePublisherId {
    IOX_TODO();
}

template <ServiceType S, typename Payload, typename UserHeader>
inline auto Publisher<S, Payload, UserHeader>::send_copy(const Payload& payload) const
    -> iox::expected<uint64_t, PublisherSendError> {
    auto* ref_handle = iox2_cast_publisher_ref_h(m_handle);

    size_t number_of_recipients = 0;
    auto result = iox2_publisher_send_copy(
        ref_handle, static_cast<const void*>(&payload), sizeof(Payload), &number_of_recipients);

    if (result == IOX2_OK) {
        return iox::ok(number_of_recipients);
    }

    return iox::err(iox::into<PublisherSendError>(result));
}

template <ServiceType S, typename Payload, typename UserHeader>
inline auto Publisher<S, Payload, UserHeader>::loan_uninit()
    -> iox::expected<SampleMut<S, Payload, UserHeader>, PublisherLoanError> {
    auto* ref_handle = iox2_cast_publisher_ref_h(m_handle);
    iox2_sample_mut_h sample_handle {};

    auto result = iox2_publisher_loan(ref_handle, 1, nullptr, &sample_handle);

    if (result == IOX2_OK) {
        return iox::ok(SampleMut<S, Payload, UserHeader>(sample_handle));
    }

    return iox::err(iox::into<PublisherLoanError>(result));
}

template <ServiceType S, typename Payload, typename UserHeader>
inline auto
Publisher<S, Payload, UserHeader>::loan() -> iox::expected<SampleMut<S, Payload, UserHeader>, PublisherLoanError> {
    IOX_TODO();
}

template <ServiceType S, typename Payload, typename UserHeader>
inline auto Publisher<S, Payload, UserHeader>::loan_slice(const uint64_t number_of_elements)
    -> iox::expected<SampleMut<S, Payload, UserHeader>, PublisherLoanError> {
    IOX_TODO();
}

template <ServiceType S, typename Payload, typename UserHeader>
inline auto Publisher<S, Payload, UserHeader>::loan_slice_uninit(const uint64_t number_of_elements)
    -> iox::expected<SampleMut<S, Payload, UserHeader>, PublisherLoanError> {
    IOX_TODO();
}

template <ServiceType S, typename Payload, typename UserHeader>
inline auto Publisher<S, Payload, UserHeader>::update_connections() -> iox::expected<void, ConnectionFailure> {
    IOX_TODO();
}
} // namespace iox2

#endif
