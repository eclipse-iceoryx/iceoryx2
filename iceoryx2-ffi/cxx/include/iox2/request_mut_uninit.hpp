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

#include "iox/assertions_addendum.hpp"
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
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
class RequestMutUninit {
    using ValueType = typename PayloadInfo<RequestPayload>::ValueType;

  public:
    RequestMutUninit(RequestMutUninit&& rhs) noexcept = default;
    auto operator=(RequestMutUninit&& rhs) noexcept -> RequestMutUninit& = default;
    ~RequestMutUninit() noexcept = default;

    RequestMutUninit(const RequestMutUninit&) = delete;
    auto operator=(const RequestMutUninit&) -> RequestMutUninit& = delete;

    /// Returns a reference to the iceoryx2 internal
    /// [`service::header::request_response::RequestHeader`]
    auto header() const -> RequestHeaderRequestResponse&;

    /// Returns a reference to the user defined request header.
    template <typename T = RequestHeader, typename = std::enable_if_t<!std::is_same_v<void, RequestHeader>, T>>
    auto user_header() const -> const T&;

    /// Returns a mutable reference to the user defined request header.
    template <typename T = RequestHeader, typename = std::enable_if_t<!std::is_same_v<void, RequestHeader>, T>>
    auto user_header_mut() -> T&;

    /// Returns a reference to the user defined request payload.
    template <typename T = RequestPayload, typename = std::enable_if_t<!iox::IsSlice<T>::VALUE, void>>
    auto payload() const -> const RequestPayload&;

    template <typename T = RequestPayload, typename = std::enable_if_t<iox::IsSlice<T>::VALUE, void>>
    auto payload() const -> iox::ImmutableSlice<ValueType>;

    /// Returns a mutable reference to the user defined request payload.
    template <typename T = RequestPayload, typename = std::enable_if_t<!iox::IsSlice<T>::VALUE, void>>
    auto payload_mut() -> RequestPayload&;

    template <typename T = RequestPayload, typename = std::enable_if_t<iox::IsSlice<T>::VALUE, void>>
    auto payload_mut() -> iox::MutableSlice<ValueType>;

    /// Copies the provided payload into the uninitialized request and returns
    /// an initialized [`RequestMut`].
    template <typename T = RequestPayload, typename = std::enable_if_t<!iox::IsSlice<T>::VALUE, T>>
    void write_payload(RequestPayload&& payload);

    template <typename T = RequestPayload, typename = std::enable_if_t<iox::IsSlice<T>::VALUE, T>>
    void write_from_slice(iox::ImmutableSlice<ValueType>& value);

    template <typename T = RequestPayload, typename = std::enable_if_t<iox::IsSlice<T>::VALUE, T>>
    void write_from_fn(const iox::function<typename T::ValueType(uint64_t)>& initializer);

  private:
    template <ServiceType, typename, typename, typename, typename>
    friend class Client;

    explicit RequestMutUninit() = default;

    template <ServiceType S,
              typename RequestPayloadT,
              typename RequestHeaderT,
              typename ResponsePayloadT,
              typename ResponseHeaderT>
    friend auto
    assume_init(RequestMutUninit<S, RequestPayloadT, RequestHeaderT, ResponsePayloadT, ResponseHeaderT>&& self)
        -> RequestMut<S, RequestPayloadT, RequestHeaderT, ResponsePayloadT, ResponseHeaderT>;

    RequestMut<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader> m_request;
};

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline auto RequestMutUninit<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::header() const
    -> RequestHeaderRequestResponse& {
    IOX_TODO();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
template <typename T, typename>
inline auto
RequestMutUninit<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::user_header() const
    -> const T& {
    IOX_TODO();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
template <typename T, typename>
inline auto RequestMutUninit<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::user_header_mut()
    -> T& {
    IOX_TODO();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
template <typename T, typename>
inline auto RequestMutUninit<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::payload() const
    -> const RequestPayload& {
    IOX_TODO();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
template <typename T, typename>
inline auto RequestMutUninit<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::payload() const
    -> iox::ImmutableSlice<ValueType> {
    return m_request.payload();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
template <typename T, typename>
inline auto RequestMutUninit<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::payload_mut()
    -> RequestPayload& {
    return m_request.payload_mut();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
template <typename T, typename>
inline auto RequestMutUninit<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::payload_mut()
    -> iox::MutableSlice<ValueType> {
    return m_request.payload_mut();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
template <typename T, typename>
inline void RequestMutUninit<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::write_payload(
    RequestPayload&& payload) {
    new (&payload_mut()) RequestPayload(std::forward<T>(payload));
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
template <typename T, typename>
inline void RequestMutUninit<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::write_from_slice(
    iox::ImmutableSlice<ValueType>& value) {
    auto dest = payload_mut();
    IOX_ASSERT(dest.number_of_bytes() >= value.number_of_bytes(),
               "Destination payload size is smaller than source slice size");
    std::memcpy(dest.begin(), value.begin(), value.number_of_bytes());
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
template <typename T, typename>
inline void RequestMutUninit<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::write_from_fn(
    const iox::function<typename T::ValueType(uint64_t)>& initializer) {
    auto slice = payload_mut();
    for (uint64_t i = 0; i < slice.number_of_elements(); ++i) {
        new (&slice[i]) typename T::ValueType(initializer(i));
    }
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline auto
assume_init(RequestMutUninit<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>&& self)
    -> RequestMut<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader> {
    return std::move(self.m_request);
}

} // namespace iox2

#endif
