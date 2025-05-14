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

#ifndef IOX2_REQUEST_MUT_UNINIT_HPP
#define IOX2_REQUEST_MUT_UNINIT_HPP

#include "iox/function.hpp"
#include "iox2/header_request_response.hpp"
#include "iox2/request_mut.hpp"
#include "iox2/service_type.hpp"

namespace iox2 {

/// A version of the [`RequestMut`] where the payload is not initialized which allows
/// true zero copy usage. To send a [`RequestMutUninit`] it must be first initialized
/// and converted into [`RequestMut`] with [`RequestMutUninit::assume_init()`].
template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
class RequestMutUninit {
    using ValueType = typename PayloadInfo<RequestPayload>::ValueType;

  public:
    RequestMutUninit(RequestMutUninit&& rhs) noexcept = default;
    auto operator=(RequestMutUninit&& rhs) noexcept -> RequestMutUninit& = default;
    ~RequestMutUninit() noexcept = default;

    RequestMutUninit(const RequestMutUninit&) = delete;
    auto operator=(const RequestMutUninit&) -> RequestMutUninit& = delete;

    /// Returns a const reference to the request payload
    auto operator*() const -> const RequestPayload&;

    /// Returns a reference to the request payload
    auto operator*() -> RequestPayload&;

    /// Returns a const pointer to the request payload
    auto operator->() const -> const RequestPayload*;

    /// Returns a pointer to the request payload
    auto operator->() -> RequestPayload*;

    /// Returns a reference to the iceoryx2 internal [`RequestHeader`]
    auto header() const -> RequestHeader;

    /// Returns a reference to the user defined request header.
    template <typename T = RequestUserHeader, typename = std::enable_if_t<!std::is_same_v<void, RequestUserHeader>, T>>
    auto user_header() const -> const T&;

    /// Returns a mutable reference to the user defined request header.
    template <typename T = RequestUserHeader, typename = std::enable_if_t<!std::is_same_v<void, RequestUserHeader>, T>>
    auto user_header_mut() -> T&;

    /// Returns a reference to the user defined request payload.
    template <typename T = RequestPayload, typename = std::enable_if_t<!iox::IsSlice<T>::VALUE, void>>
    auto payload() const -> const RequestPayload&;

    /// Returns a reference to the user defined request payload.
    template <typename T = RequestPayload, typename = std::enable_if_t<iox::IsSlice<T>::VALUE, void>>
    auto payload() const -> iox::ImmutableSlice<ValueType>;

    /// Returns a mutable reference to the user defined request payload.
    template <typename T = RequestPayload, typename = std::enable_if_t<!iox::IsSlice<T>::VALUE, void>>
    auto payload_mut() -> RequestPayload&;

    /// Returns a mutable reference to the user defined request payload.
    template <typename T = RequestPayload, typename = std::enable_if_t<iox::IsSlice<T>::VALUE, void>>
    auto payload_mut() -> iox::MutableSlice<ValueType>;

    /// Copies the provided payload into the uninitialized request and returns
    /// an initialized [`RequestMut`].
    template <typename T = RequestPayload, typename = std::enable_if_t<!iox::IsSlice<T>::VALUE, T>>
    auto write_payload(RequestPayload&& payload)
        -> RequestMut<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>;

    /// Copies the provided payload into the uninitialized request and returns
    /// an initialized [`RequestMut`].
    template <typename T = RequestPayload, typename = std::enable_if_t<iox::IsSlice<T>::VALUE, T>>
    auto write_from_slice(iox::ImmutableSlice<ValueType>& value)
        -> RequestMut<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>;

    /// Copies the provided payload into the uninitialized request and returns
    /// an initialized [`RequestMut`].
    template <typename T = RequestPayload, typename = std::enable_if_t<iox::IsSlice<T>::VALUE, T>>
    auto write_from_fn(const iox::function<typename T::ValueType(uint64_t)>& initializer)
        -> RequestMut<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>;

  private:
    template <ServiceType, typename, typename, typename, typename>
    friend class Client;

    explicit RequestMutUninit() = default;

    template <ServiceType S,
              typename RequestPayloadT,
              typename RequestUserHeaderT,
              typename ResponsePayloadT,
              typename ResponseUserHeaderT>
    friend auto
    assume_init(RequestMutUninit<S, RequestPayloadT, RequestUserHeaderT, ResponsePayloadT, ResponseUserHeaderT>&& self)
        -> RequestMut<S, RequestPayloadT, RequestUserHeaderT, ResponsePayloadT, ResponseUserHeaderT>;

    RequestMut<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader> m_request;
};

template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
inline auto
RequestMutUninit<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>::operator*() const
    -> const RequestPayload& {
    return payload();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
inline auto
RequestMutUninit<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>::operator*()
    -> RequestPayload& {
    return payload_mut();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
inline auto
RequestMutUninit<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>::operator->() const
    -> const RequestPayload* {
    return &payload();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
inline auto
RequestMutUninit<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>::operator->()
    -> RequestPayload* {
    return &payload_mut();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
inline auto
RequestMutUninit<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>::header() const
    -> RequestHeader {
    return m_request.header();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
template <typename T, typename>
inline auto
RequestMutUninit<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>::user_header() const
    -> const T& {
    return m_request.user_header();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
template <typename T, typename>
inline auto
RequestMutUninit<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>::user_header_mut()
    -> T& {
    return m_request.user_header_mut();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
template <typename T, typename>
inline auto
RequestMutUninit<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>::payload() const
    -> const RequestPayload& {
    return m_request.payload();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
template <typename T, typename>
inline auto
RequestMutUninit<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>::payload() const
    -> iox::ImmutableSlice<ValueType> {
    return m_request.payload();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
template <typename T, typename>
inline auto
RequestMutUninit<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>::payload_mut()
    -> RequestPayload& {
    return m_request.payload_mut();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
template <typename T, typename>
inline auto
RequestMutUninit<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>::payload_mut()
    -> iox::MutableSlice<ValueType> {
    return m_request.payload_mut();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
template <typename T, typename>
inline auto
RequestMutUninit<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>::write_payload(
    RequestPayload&& payload)
    -> RequestMut<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader> {
    new (&payload_mut()) RequestPayload(std::forward<T>(payload));
    return std::move(m_request);
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
template <typename T, typename>
inline auto
RequestMutUninit<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>::write_from_slice(
    iox::ImmutableSlice<ValueType>& value)
    -> RequestMut<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader> {
    auto dest = payload_mut();
    IOX_ASSERT(dest.number_of_bytes() >= value.number_of_bytes(),
               "Destination payload size is smaller than source slice size");
    std::memcpy(dest.begin(), value.begin(), value.number_of_bytes());
    return std::move(m_request);
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
template <typename T, typename>
inline auto
RequestMutUninit<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>::write_from_fn(
    const iox::function<typename T::ValueType(uint64_t)>& initializer)
    -> RequestMut<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader> {
    auto slice = payload_mut();
    for (uint64_t i = 0; i < slice.number_of_elements(); ++i) {
        new (&slice[i]) typename T::ValueType(initializer(i));
    }
    return std::move(m_request);
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
inline auto
assume_init(RequestMutUninit<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>&& self)
    -> RequestMut<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader> {
    return std::move(self.m_request);
}

} // namespace iox2

#endif
