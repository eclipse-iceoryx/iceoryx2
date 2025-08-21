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

#include "iox/slice.hpp"
#include "iox2/header_publish_subscribe.hpp"
#include "iox2/internal/iceoryx2.hpp"
#include "iox2/payload_info.hpp"
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
// NOLINTNEXTLINE(cppcoreguidelines-pro-type-member-init,hicpp-member-init) 'm_sample' is not used directly but only via the initialized 'm_handle'; furthermore, it will be initialized on the call site
class Sample {
    using ValueType = typename PayloadInfo<Payload>::ValueType;

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
    template <typename T = Payload, typename = std::enable_if_t<!iox::IsSlice<T>::VALUE, void>>
    auto payload() const -> const ValueType&;

    /// Returns a slice to navigate the payload of the [`Sample`]
    template <typename T = Payload, typename = std::enable_if_t<iox::IsSlice<T>::VALUE, void>>
    auto payload() const -> iox::ImmutableSlice<ValueType>;

    /// Returns a reference to the user_header of the [`Sample`]
    template <typename T = UserHeader, typename = std::enable_if_t<!std::is_same_v<void, UserHeader>, T>>
    auto user_header() const -> const T&;

    /// Returns a reference to the [`Header`] of the [`Sample`].
    auto header() const -> HeaderPublishSubscribe;

    /// Returns the [`UniquePublisherId`] of the [`Publisher`](crate::port::publisher::Publisher)
    auto origin() const -> UniquePublisherId;

  private:
    template <ServiceType, typename, typename>
    friend class Subscriber;

    // The sample is defaulted since both members are initialized in Subscriber::receive
    explicit Sample() = default;
    void drop();

    iox2_sample_t m_sample;
    iox2_sample_h m_handle = nullptr;
};

template <ServiceType S, typename Payload, typename UserHeader>
inline void Sample<S, Payload, UserHeader>::drop() {
    if (m_handle != nullptr) {
        iox2_sample_drop(m_handle);
        m_handle = nullptr;
    }
}

// NOLINTNEXTLINE(cppcoreguidelines-pro-type-member-init,hicpp-member-init) m_sample will be initialized in the move assignment operator
template <ServiceType S, typename Payload, typename UserHeader>
inline Sample<S, Payload, UserHeader>::Sample(Sample&& rhs) noexcept {
    *this = std::move(rhs);
}

namespace internal {
extern "C" {
void iox2_sample_move(iox2_sample_t*, iox2_sample_t*, iox2_sample_h*);
}
} // namespace internal

template <ServiceType S, typename Payload, typename UserHeader>
inline auto Sample<S, Payload, UserHeader>::operator=(Sample&& rhs) noexcept -> Sample& {
    if (this != &rhs) {
        drop();

        internal::iox2_sample_move(&rhs.m_sample, &m_sample, &m_handle);
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
template <typename T, typename>
inline auto Sample<S, Payload, UserHeader>::payload() const -> const ValueType& {
    const void* ptr = nullptr;

    iox2_sample_payload(&m_handle, &ptr, nullptr);

    return *static_cast<const ValueType*>(ptr);
}

template <ServiceType S, typename Payload, typename UserHeader>
template <typename T, typename>
inline auto Sample<S, Payload, UserHeader>::payload() const -> iox::ImmutableSlice<ValueType> {
    const void* ptr = nullptr;
    size_t number_of_elements = 0;

    iox2_sample_payload(&m_handle, &ptr, &number_of_elements);

    return iox::ImmutableSlice<ValueType>(static_cast<const ValueType*>(ptr), number_of_elements);
}

template <ServiceType S, typename Payload, typename UserHeader>
template <typename T, typename>
inline auto Sample<S, Payload, UserHeader>::user_header() const -> const T& {
    const void* header_ptr = nullptr;

    iox2_sample_user_header(&m_handle, &header_ptr);

    return *static_cast<const T*>(header_ptr);
}

template <ServiceType S, typename Payload, typename UserHeader>
inline auto Sample<S, Payload, UserHeader>::header() const -> HeaderPublishSubscribe {
    iox2_publish_subscribe_header_h header_handle = nullptr;
    iox2_sample_header(&m_handle, nullptr, &header_handle);

    return HeaderPublishSubscribe { header_handle };
}

template <ServiceType S, typename Payload, typename UserHeader>
inline auto Sample<S, Payload, UserHeader>::origin() const -> UniquePublisherId {
    return header().publisher_id();
}

} // namespace iox2

#endif
