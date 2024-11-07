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

#ifndef IOX2_SAMPLE_MUT_HPP
#define IOX2_SAMPLE_MUT_HPP

#include "iox/assertions.hpp"
#include "iox/expected.hpp"
#include "iox/slice.hpp"
#include "iox2/header_publish_subscribe.hpp"
#include "iox2/iceoryx2.h"
#include "iox2/internal/iceoryx2.hpp"
#include "iox2/payload_info.hpp"
#include "iox2/publisher_error.hpp"
#include "iox2/service_type.hpp"

namespace iox2 {

/// Acquired by a [`Publisher`] via
///  * [`Publisher::loan()`],
///  * [`Publisher::loan_uninit()`]
///  * [`Publisher::loan_slice()`]
///  * [`Publisher::loan_slice_uninit()`]
///
/// It stores the payload that will be sent
/// to all connected [`Subscriber`]s. If the [`SampleMut`] is not sent
/// it will release the loaned memory when going out of scope.
///
/// # Notes
///
/// Does not implement [`Send`] since it releases unsent samples in the [`Publisher`] and the
/// [`Publisher`] is not thread-safe!
///
/// # Important
///
/// DO NOT MOVE THE SAMPLE INTO ANOTHER THREAD!
template <ServiceType S, typename Payload, typename UserHeader>
// NOLINTNEXTLINE(cppcoreguidelines-pro-type-member-init,hicpp-member-init) 'm_sample' is not used directly but only via the initialized 'm_handle'; furthermore, it will be initialized on the call site
class SampleMut {
    using ValueType = typename PayloadInfo<Payload>::ValueType;

  public:
    SampleMut(SampleMut&& rhs) noexcept;
    auto operator=(SampleMut&& rhs) noexcept -> SampleMut&;
    ~SampleMut() noexcept;

    SampleMut(const SampleMut&) = delete;
    auto operator=(const SampleMut&) -> SampleMut& = delete;

    /// Returns a const reference to the payload of the [`Sample`]
    auto operator*() const -> const Payload&;

    /// Returns a reference to the payload of the [`Sample`]
    auto operator*() -> Payload&;

    /// Returns a const pointer to the payload of the [`Sample`]
    auto operator->() const -> const Payload*;

    /// Returns a pointer to the payload of the [`Sample`]
    auto operator->() -> Payload*;

    /// Returns a reference to the [`Header`] of the [`Sample`].
    auto header() const -> HeaderPublishSubscribe;

    /// Returns a reference to the user_header of the [`Sample`]
    template <typename T = UserHeader, typename = std::enable_if_t<!std::is_same_v<void, UserHeader>, T>>
    auto user_header() const -> const T&;

    /// Returns a mutable reference to the user_header of the [`Sample`].
    template <typename T = UserHeader, typename = std::enable_if_t<!std::is_same_v<void, UserHeader>, T>>
    auto user_header_mut() -> T&;

    /// Returns a reference to the const payload of the sample.
    template <typename T = Payload, typename = std::enable_if_t<!iox::IsSlice<T>::VALUE, void>>
    auto payload() const -> const ValueType&;

    /// Returns a reference to the payload of the sample.
    template <typename T = Payload, typename = std::enable_if_t<!iox::IsSlice<T>::VALUE, void>>
    auto payload_mut() -> ValueType&;

    template <typename T = Payload, typename = std::enable_if_t<iox::IsSlice<T>::VALUE, void>>
    auto payload() const -> iox::ImmutableSlice<ValueType>;

    template <typename T = Payload, typename = std::enable_if_t<iox::IsSlice<T>::VALUE, void>>
    auto payload_mut() -> iox::MutableSlice<ValueType>;

  private:
    template <ServiceType, typename, typename>
    friend class Publisher;
    template <ServiceType, typename, typename>
    friend class SampleMutUninit;

    template <ServiceType ST, typename PayloadT, typename UserHeaderT>
    friend auto send(SampleMut<ST, PayloadT, UserHeaderT>&& sample) -> iox::expected<size_t, PublisherSendError>;

    // The sample is defaulted since both members are initialized in Publisher::loan() or
    // Publisher::loan_slice()
    explicit SampleMut() = default;
    void drop();

    // NOLINTNEXTLINE(cppcoreguidelines-pro-type-member-init,hicpp-member-init) will not be accessed directly but only via m_handle and will be set together with m_handle
    iox2_sample_mut_t m_sample;
    iox2_sample_mut_h m_handle { nullptr };
};

template <ServiceType S, typename Payload, typename UserHeader>
inline void SampleMut<S, Payload, UserHeader>::drop() {
    if (m_handle != nullptr) {
        iox2_sample_mut_drop(m_handle);
        m_handle = nullptr;
    }
}

// NOLINTNEXTLINE(cppcoreguidelines-pro-type-member-init,hicpp-member-init) m_sample will be initialized in the move assignment operator
template <ServiceType S, typename Payload, typename UserHeader>
inline SampleMut<S, Payload, UserHeader>::SampleMut(SampleMut&& rhs) noexcept {
    *this = std::move(rhs);
}

namespace internal {
extern "C" {
void iox2_sample_mut_move(iox2_sample_mut_t*, iox2_sample_mut_t*, iox2_sample_mut_h*);
}
} // namespace internal

template <ServiceType S, typename Payload, typename UserHeader>
inline auto SampleMut<S, Payload, UserHeader>::operator=(SampleMut&& rhs) noexcept -> SampleMut& {
    if (this != &rhs) {
        drop();

        internal::iox2_sample_mut_move(&rhs.m_sample, &m_sample, &m_handle);
        rhs.m_handle = nullptr;
    }

    return *this;
}

template <ServiceType S, typename Payload, typename UserHeader>
inline SampleMut<S, Payload, UserHeader>::~SampleMut() noexcept {
    drop();
}

template <ServiceType S, typename Payload, typename UserHeader>
inline auto SampleMut<S, Payload, UserHeader>::operator*() const -> const Payload& {
    return payload();
}

template <ServiceType S, typename Payload, typename UserHeader>
inline auto SampleMut<S, Payload, UserHeader>::operator*() -> Payload& {
    return payload_mut();
}

template <ServiceType S, typename Payload, typename UserHeader>
inline auto SampleMut<S, Payload, UserHeader>::operator->() const -> const Payload* {
    return &payload();
}

template <ServiceType S, typename Payload, typename UserHeader>
inline auto SampleMut<S, Payload, UserHeader>::operator->() -> Payload* {
    return &payload_mut();
}

template <ServiceType S, typename Payload, typename UserHeader>
inline auto SampleMut<S, Payload, UserHeader>::header() const -> HeaderPublishSubscribe {
    iox2_publish_subscribe_header_h header_handle = nullptr;
    iox2_sample_mut_header(&m_handle, nullptr, &header_handle);

    return HeaderPublishSubscribe { header_handle };
}

template <ServiceType S, typename Payload, typename UserHeader>
template <typename T, typename>
inline auto SampleMut<S, Payload, UserHeader>::user_header() const -> const T& {
    const void* ptr = nullptr;

    iox2_sample_mut_user_header(&m_handle, &ptr);

    return *static_cast<const T*>(ptr);
}

template <ServiceType S, typename Payload, typename UserHeader>
template <typename T, typename>
inline auto SampleMut<S, Payload, UserHeader>::user_header_mut() -> T& {
    void* ptr = nullptr;

    iox2_sample_mut_user_header_mut(&m_handle, &ptr);

    return *static_cast<T*>(ptr);
}

template <ServiceType S, typename Payload, typename UserHeader>
template <typename T, typename>
inline auto SampleMut<S, Payload, UserHeader>::payload() const -> const ValueType& {
    const void* ptr = nullptr;

    iox2_sample_mut_payload(&m_handle, &ptr, nullptr);

    return *static_cast<const ValueType*>(ptr);
}

template <ServiceType S, typename Payload, typename UserHeader>
template <typename T, typename>
inline auto SampleMut<S, Payload, UserHeader>::payload_mut() -> ValueType& {
    void* ptr = nullptr;

    iox2_sample_mut_payload_mut(&m_handle, &ptr, nullptr);

    return *static_cast<ValueType*>(ptr);
}

template <ServiceType S, typename Payload, typename UserHeader>
template <typename T, typename>
inline auto SampleMut<S, Payload, UserHeader>::payload() const -> iox::ImmutableSlice<ValueType> {
    const void* ptr = nullptr;
    size_t number_of_elements = 0;

    iox2_sample_mut_payload(&m_handle, &ptr, &number_of_elements);

    return iox::ImmutableSlice<ValueType>(static_cast<ValueType*>(ptr), number_of_elements);
}

template <ServiceType S, typename Payload, typename UserHeader>
template <typename T, typename>
inline auto SampleMut<S, Payload, UserHeader>::payload_mut() -> iox::MutableSlice<ValueType> {
    void* ptr = nullptr;
    size_t number_of_elements = 0;

    iox2_sample_mut_payload_mut(&m_handle, &ptr, &number_of_elements);

    return iox::MutableSlice<ValueType>(static_cast<ValueType*>(ptr), number_of_elements);
}

template <ServiceType S, typename Payload, typename UserHeader>
inline auto send(SampleMut<S, Payload, UserHeader>&& sample) -> iox::expected<size_t, PublisherSendError> {
    size_t number_of_recipients = 0;
    auto result = iox2_sample_mut_send(sample.m_handle, &number_of_recipients);
    sample.m_handle = nullptr;

    if (result == IOX2_OK) {
        return iox::ok(number_of_recipients);
    }

    return iox::err(iox::into<PublisherSendError>(result));
}

} // namespace iox2

#endif
