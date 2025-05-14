// Copyright (c) 2025 Contributors to the Eclipse Foundation
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

#ifndef IOX2_RESPONSE_MUT_UNINIT_HPP
#define IOX2_RESPONSE_MUT_UNINIT_HPP

#include "iox/function.hpp"
#include "iox/slice.hpp"
#include "iox2/payload_info.hpp"
#include "iox2/response_mut.hpp"
#include "iox2/service_type.hpp"

namespace iox2 {

/// Acquired by a [`ActiveRequest`] with
///  * [`ActiveRequest::loan_uninit()`]
///
/// It stores the payload of the response that will be sent to the corresponding
/// [`PendingResponse`] of the [`Client`].
///
/// If the [`ResponseMutUninit`] is not sent it will reelase the loaned memory when going out of
/// scope.
template <ServiceType Service, typename ResponsePayload, typename ResponseUserHeader>
class ResponseMutUninit {
    using ValueType = typename PayloadInfo<ResponsePayload>::ValueType;

  public:
    ResponseMutUninit(ResponseMutUninit&& rhs) noexcept = default;
    auto operator=(ResponseMutUninit&& rhs) noexcept -> ResponseMutUninit& = default;
    ~ResponseMutUninit() noexcept = default;

    ResponseMutUninit(const ResponseMutUninit&) = delete;
    auto operator=(const ResponseMutUninit&) -> ResponseMutUninit& = delete;

    /// Returns a reference to the [`ResponseHeader`].
    auto header() const -> ResponseHeader;

    /// Returns a reference to the user header of the response.
    template <typename T = ResponseUserHeader,
              typename = std::enable_if_t<!std::is_same_v<void, ResponseUserHeader>, T>>
    auto user_header() const -> const T&;

    /// Returns a mutable reference to the user header of the response.
    template <typename T = ResponseUserHeader,
              typename = std::enable_if_t<!std::is_same_v<void, ResponseUserHeader>, T>>
    auto user_header_mut() -> T&;

    /// Returns a reference to the payload of the response.
    template <typename T = ResponsePayload, typename = std::enable_if_t<!iox::IsSlice<T>::VALUE, void>>
    auto payload() const -> const T&;

    /// Returns a reference to the payload of the response.
    template <typename T = ResponsePayload, typename = std::enable_if_t<iox::IsSlice<T>::VALUE, void>>
    auto payload() const -> iox::ImmutableSlice<ValueType>;

    /// Returns a mutable reference to the payload of the response.
    template <typename T = ResponsePayload, typename = std::enable_if_t<!iox::IsSlice<T>::VALUE, void>>
    auto payload_mut() -> T&;

    /// Returns a mutable reference to the payload of the response.
    template <typename T = ResponsePayload, typename = std::enable_if_t<iox::IsSlice<T>::VALUE, void>>
    auto payload_mut() -> iox::MutableSlice<ValueType>;

    /// Writes the provided payload into the [`ResponseMutUninit`] and returns an initialized
    /// [`ResponseMut`] that is ready to be sent.
    template <typename T = ResponsePayload, typename = std::enable_if_t<!iox::IsSlice<T>::VALUE, T>>
    auto write_payload(ResponsePayload&& payload) -> ResponseMut<Service, T, ResponseUserHeader>;

    /// Writes the provided payload into the [`ResponseMutUninit`] and returns an initialized
    /// [`ResponseMut`] that is ready to be sent.
    template <typename T = ResponsePayload, typename = std::enable_if_t<iox::IsSlice<T>::VALUE, T>>
    auto write_from_slice(iox::ImmutableSlice<ValueType>& value) -> ResponseMut<Service, T, ResponseUserHeader>;

    /// Writes the provided payload into the [`ResponseMutUninit`] and returns an initialized
    /// [`ResponseMut`] that is ready to be sent.
    template <typename T = ResponsePayload, typename = std::enable_if_t<iox::IsSlice<T>::VALUE, T>>
    auto write_from_fn(const iox::function<typename T::ValueType(uint64_t)>& initializer)
        -> ResponseMut<Service, T, ResponseUserHeader>;

  private:
    template <ServiceType, typename, typename, typename, typename>
    friend class ActiveRequest;

    template <ServiceType S, typename ResponsePayloadT, typename ResponseUserHeaderT>
    friend auto assume_init(ResponseMutUninit<S, ResponsePayloadT, ResponseUserHeaderT>&& self)
        -> ResponseMut<S, ResponsePayloadT, ResponseUserHeaderT>;

    explicit ResponseMutUninit() = default;

    ResponseMut<Service, ResponsePayload, ResponseUserHeader> m_response;
};

template <ServiceType Service, typename ResponsePayload, typename ResponseUserHeader>
inline auto ResponseMutUninit<Service, ResponsePayload, ResponseUserHeader>::header() const -> ResponseHeader {
    return m_response.header();
}

template <ServiceType Service, typename ResponsePayload, typename ResponseUserHeader>
template <typename T, typename>
inline auto ResponseMutUninit<Service, ResponsePayload, ResponseUserHeader>::user_header() const -> const T& {
    return m_response.user_header();
}

template <ServiceType Service, typename ResponsePayload, typename ResponseUserHeader>
template <typename T, typename>
inline auto ResponseMutUninit<Service, ResponsePayload, ResponseUserHeader>::user_header_mut() -> T& {
    return m_response.user_header_mut();
}

template <ServiceType Service, typename ResponsePayload, typename ResponseUserHeader>
template <typename T, typename>
inline auto ResponseMutUninit<Service, ResponsePayload, ResponseUserHeader>::payload() const -> const T& {
    return m_response.payload();
}

template <ServiceType Service, typename ResponsePayload, typename ResponseUserHeader>
template <typename T, typename>
inline auto ResponseMutUninit<Service, ResponsePayload, ResponseUserHeader>::payload() const
    -> iox::ImmutableSlice<ValueType> {
    return m_response.payload();
}

template <ServiceType Service, typename ResponsePayload, typename ResponseUserHeader>
template <typename T, typename>
inline auto ResponseMutUninit<Service, ResponsePayload, ResponseUserHeader>::payload_mut() -> T& {
    return m_response.payload_mut();
}

template <ServiceType Service, typename ResponsePayload, typename ResponseUserHeader>
template <typename T, typename>
inline auto ResponseMutUninit<Service, ResponsePayload, ResponseUserHeader>::payload_mut()
    -> iox::MutableSlice<ValueType> {
    return m_response.payload_mut();
}

template <ServiceType Service, typename ResponsePayload, typename ResponseUserHeader>
template <typename T, typename>
inline auto ResponseMutUninit<Service, ResponsePayload, ResponseUserHeader>::write_payload(ResponsePayload&& payload)
    -> ResponseMut<Service, T, ResponseUserHeader> {
    new (&payload_mut()) ResponsePayload(std::forward<T>(payload));
    return std::move(m_response);
}

template <ServiceType Service, typename ResponsePayload, typename ResponseUserHeader>
template <typename T, typename>
inline auto
ResponseMutUninit<Service, ResponsePayload, ResponseUserHeader>::write_from_slice(iox::ImmutableSlice<ValueType>& value)
    -> ResponseMut<Service, T, ResponseUserHeader> {
    auto dest = payload_mut();
    IOX_ASSERT(dest.number_of_bytes() >= value.number_of_bytes(),
               "Destination payload size is smaller than source slice size");
    std::memcpy(dest.begin(), value.begin(), value.number_of_bytes());
    return std::move(m_response);
}

template <ServiceType Service, typename ResponsePayload, typename ResponseUserHeader>
template <typename T, typename>
inline auto ResponseMutUninit<Service, ResponsePayload, ResponseUserHeader>::write_from_fn(
    const iox::function<typename T::ValueType(uint64_t)>& initializer) -> ResponseMut<Service, T, ResponseUserHeader> {
    auto slice = payload_mut();
    for (uint64_t i = 0; i < slice.number_of_elements(); ++i) {
        new (&slice[i]) typename T::ValueType(initializer(i));
    }
    return std::move(m_response);
}

template <ServiceType Service, typename ResponsePayload, typename ResponseUserHeader>
inline auto assume_init(ResponseMutUninit<Service, ResponsePayload, ResponseUserHeader>&& self)
    -> ResponseMut<Service, ResponsePayload, ResponseUserHeader> {
    return std::move(self.m_response);
}

} // namespace iox2

#endif
