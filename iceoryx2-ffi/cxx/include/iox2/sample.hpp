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

#ifndef IOX2_SAMPLE_HPP
#define IOX2_SAMPLE_HPP

#include "iox/assertions_addendum.hpp"
#include "iox2/header_publish_subscribe.hpp"
#include "iox2/internal/iceoryx2.hpp"
#include "iox2/service_type.hpp"
#include "iox2/unique_port_id.hpp"

namespace iox2 {
/// It stores the payload and is acquired by the [`Subscriber`] whenever
/// it receives new data from a [`Publisher`] via
/// [`Subscriber::receive()`].
/// # Notes
///
/// Does not implement [`Send`] since it releases unsent samples vie the [`Subscriber`] and the
/// [`Subscriber`] is not thread-safe!
///
/// # Important
///
/// DO NOT MOVE THE SAMPLE INTO ANOTHER THREAD!
template <ServiceType, typename Payload, typename UserHeader>
class Sample {
  public:
    Sample(Sample&& rhs) noexcept;
    auto operator=(Sample&& rhs) noexcept -> Sample&;
    ~Sample();

    Sample(const Sample&) = delete;
    auto operator=(const Sample&) -> Sample& = delete;

    /// Returns a reference to the payload of the [`Sample`]
    auto operator*() const -> const Payload&;

    /// Returns a pointer to the payload of the [`Sample`]
    auto operator->() const -> const Payload*;

    /// Returns a reference to the payload of the [`Sample`]
    auto payload() const -> const Payload&;

    /// Returns a reference to the user_header of the [`Sample`]
    template <typename T = UserHeader, typename = std::enable_if_t<!std::is_same_v<void, UserHeader>, T>>
    auto user_header() const -> const T&;

    /// Returns a reference to the [`Header`] of the [`Sample`].
    auto header() const -> const HeaderPublishSubscribe&;

    /// Returns the [`UniquePublisherId`] of the [`Publisher`](crate::port::publisher::Publisher)
    auto origin() const -> UniquePublisherId;

  private:
    template <ServiceType, typename, typename>
    friend class Subscriber;

    explicit Sample(iox2_sample_h handle);
    void drop();

    iox2_sample_h m_handle { nullptr };
};

template <ServiceType S, typename Payload, typename UserHeader>
inline Sample<S, Payload, UserHeader>::Sample(iox2_sample_h handle)
    : m_handle { handle } {
}

template <ServiceType S, typename Payload, typename UserHeader>
inline void Sample<S, Payload, UserHeader>::drop() {
    if (m_handle != nullptr) {
        iox2_sample_drop(m_handle);
        m_handle = nullptr;
    }
}

template <ServiceType S, typename Payload, typename UserHeader>
inline Sample<S, Payload, UserHeader>::Sample(Sample&& rhs) noexcept {
    *this = std::move(rhs);
}

template <ServiceType S, typename Payload, typename UserHeader>
inline auto Sample<S, Payload, UserHeader>::operator=(Sample&& rhs) noexcept -> Sample& {
    if (this != &rhs) {
        drop();
        m_handle = std::move(rhs.m_handle);
        rhs.m_handle = nullptr;
    }

    return *this;
}

template <ServiceType S, typename Payload, typename UserHeader>
inline Sample<S, Payload, UserHeader>::~Sample() {
    drop();
}

template <ServiceType S, typename Payload, typename UserHeader>
inline auto Sample<S, Payload, UserHeader>::operator*() const -> const Payload& {
    return payload();
}

template <ServiceType S, typename Payload, typename UserHeader>
inline auto Sample<S, Payload, UserHeader>::operator->() const -> const Payload* {
    return &payload();
}

template <ServiceType S, typename Payload, typename UserHeader>
inline auto Sample<S, Payload, UserHeader>::payload() const -> const Payload& {
    auto* ref_handle = iox2_cast_sample_ref_h(m_handle);
    const Payload* payload_ptr = nullptr;
    size_t payload_len = 0;

    // NOLINTBEGIN(cppcoreguidelines-pro-type-reinterpret-cast), no other way for type erasure
    iox2_sample_payload(ref_handle, reinterpret_cast<const void**>(&payload_ptr), &payload_len);
    // NOLINTEND(cppcoreguidelines-pro-type-reinterpret-cast)
    IOX_ASSERT(sizeof(Payload) <= payload_len, "");

    return *payload_ptr;
}

template <ServiceType S, typename Payload, typename UserHeader>
template <typename T, typename>
inline auto Sample<S, Payload, UserHeader>::user_header() const -> const T& {
    IOX_TODO();
}

template <ServiceType S, typename Payload, typename UserHeader>
inline auto Sample<S, Payload, UserHeader>::header() const -> const HeaderPublishSubscribe& {
    IOX_TODO();
}

template <ServiceType S, typename Payload, typename UserHeader>
inline auto Sample<S, Payload, UserHeader>::origin() const -> UniquePublisherId {
    IOX_TODO();
}


} // namespace iox2

#endif
