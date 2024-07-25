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
template <ServiceType, typename Payload, typename UserHeader>
class Sample {
  public:
    Sample(Sample&&) noexcept;
    auto operator=(Sample&&) noexcept -> Sample&;
    ~Sample();

    Sample(const Sample&) = delete;
    auto operator=(const Sample&) -> Sample& = delete;

    auto operator*() const -> const Payload&;
    auto operator->() const -> const Payload*;

    auto payload() const -> const Payload&;
    template <typename T = UserHeader, typename = std::enable_if_t<!std::is_same_v<void, UserHeader>, T>>
    auto user_header() const -> const T&;
    auto header() const -> const HeaderPublishSubscribe&;
    auto origin() const -> UniquePublisherId;

  private:
    template <ServiceType, typename, typename>
    friend class Subscriber;

    explicit Sample(iox2_sample_h handle);
    void drop();

    iox2_sample_h m_handle;
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
inline Sample<S, Payload, UserHeader>::Sample(Sample&& rhs) noexcept
    : m_handle { nullptr } {
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

    iox2_sample_payload(ref_handle, reinterpret_cast<const void**>(&payload_ptr), &payload_len);
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
