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

#ifndef IOX2_REQUEST_MUT_HPP
#define IOX2_REQUEST_MUT_HPP

#include "iox/expected.hpp"
#include "iox/slice.hpp"
#include "iox2/header_request_response.hpp"
#include "iox2/internal/iceoryx2.hpp"
#include "iox2/payload_info.hpp"
#include "iox2/pending_response.hpp"
#include "iox2/port_error.hpp"
#include "iox2/service_type.hpp"

namespace iox2 {

/// The [`RequestMut`] represents the object that contains the payload that the
/// [`Client`] sends to the [`Server`].
template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
// NOLINTNEXTLINE(cppcoreguidelines-pro-type-member-init,hicpp-member-init) 'm_request' is not used directly but only via the initialized 'm_handle'; furthermore, it will be initialized on the call site
class RequestMut {
    using ValueType = typename PayloadInfo<RequestPayload>::ValueType;

  public:
    RequestMut(RequestMut&& rhs) noexcept;
    auto operator=(RequestMut&& rhs) noexcept -> RequestMut&;
    ~RequestMut() noexcept;

    RequestMut(const RequestMut&) = delete;
    auto operator=(const RequestMut&) -> RequestMut& = delete;

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

  private:
    template <ServiceType, typename, typename, typename, typename>
    friend class Client;
    template <ServiceType, typename, typename, typename, typename>
    friend class RequestMutUninit;

    /// Sends the [`RequestMut`] to all connected
    /// [`Server`](crate::port::server::Server)s of the
    /// [`Service`](crate::service::Service).
    template <ServiceType S,
              typename RequestPayloadT,
              typename RequestUserHeaderT,
              typename ResponsePayloadT,
              typename ResponseUserHeaderT>
    friend auto
    send(RequestMut<S, RequestPayloadT, RequestUserHeaderT, ResponsePayloadT, ResponseUserHeaderT>&& request)
        -> iox::expected<PendingResponse<S, RequestPayloadT, RequestUserHeaderT, ResponsePayloadT, ResponseUserHeaderT>,
                         RequestSendError>;

    explicit RequestMut() = default;
    void drop();

    iox2_request_mut_t m_request;
    iox2_request_mut_h m_handle = nullptr;
};

// NOLINTNEXTLINE(cppcoreguidelines-pro-type-member-init,hicpp-member-init) m_request will be initialized in the move assignment operator
template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
inline RequestMut<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>::RequestMut(
    RequestMut&& rhs) noexcept {
    *this = std::move(rhs);
}

namespace internal {
extern "C" {
void iox2_request_mut_move(iox2_request_mut_t*, iox2_request_mut_t*, iox2_request_mut_h*);
}
} // namespace internal

template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
inline auto RequestMut<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>::operator=(
    RequestMut&& rhs) noexcept -> RequestMut& {
    if (this != &rhs) {
        drop();

        internal::iox2_request_mut_move(&rhs.m_request, &m_request, &m_handle);
        rhs.m_handle = nullptr;
    }
    return *this;
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
inline RequestMut<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>::
    ~RequestMut() noexcept {
    drop();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
inline auto
RequestMut<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>::operator*() const
    -> const RequestPayload& {
    return payload();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
inline auto RequestMut<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>::operator*()
    -> RequestPayload& {
    return payload_mut();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
inline auto
RequestMut<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>::operator->() const
    -> const RequestPayload* {
    return &payload();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
inline auto RequestMut<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>::operator->()
    -> RequestPayload* {
    return &payload_mut();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
inline auto RequestMut<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>::header() const
    -> RequestHeader {
    iox2_request_header_h header_handle = nullptr;
    iox2_request_mut_header(&m_handle, nullptr, &header_handle);
    return RequestHeader { header_handle };
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
template <typename T, typename>
inline auto
RequestMut<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>::user_header() const
    -> const T& {
    const void* ptr = nullptr;
    iox2_request_mut_user_header(&m_handle, &ptr);
    return *static_cast<const T*>(ptr);
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
template <typename T, typename>
inline auto
RequestMut<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>::user_header_mut() -> T& {
    void* ptr = nullptr;
    iox2_request_mut_user_header_mut(&m_handle, &ptr);
    return *static_cast<T*>(ptr);
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
template <typename T, typename>
inline auto RequestMut<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>::payload() const
    -> const RequestPayload& {
    const void* ptr = nullptr;
    iox2_request_mut_payload(&m_handle, &ptr, nullptr);
    return *static_cast<const RequestPayload*>(ptr);
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
template <typename T, typename>
inline auto RequestMut<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>::payload() const
    -> iox::ImmutableSlice<ValueType> {
    const void* ptr = nullptr;
    size_t number_of_elements = 0;
    iox2_request_mut_payload(&m_handle, &ptr, &number_of_elements);
    return iox::ImmutableSlice<ValueType>(static_cast<const ValueType*>(ptr), number_of_elements);
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
template <typename T, typename>
inline auto RequestMut<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>::payload_mut()
    -> RequestPayload& {
    void* ptr = nullptr;
    iox2_request_mut_payload_mut(&m_handle, &ptr, nullptr);
    return *static_cast<RequestPayload*>(ptr);
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
template <typename T, typename>
inline auto RequestMut<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>::payload_mut()
    -> iox::MutableSlice<ValueType> {
    void* ptr = nullptr;
    size_t number_of_elements = 0;
    iox2_request_mut_payload_mut(&m_handle, &ptr, &number_of_elements);
    return iox::MutableSlice<ValueType>(static_cast<ValueType*>(ptr), number_of_elements);
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
inline auto send(RequestMut<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>&& request)
    -> iox::expected<PendingResponse<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>,
                     RequestSendError> {
    iox2_pending_response_h pending_response_handle {};
    auto result = iox2_request_mut_send(request.m_handle, nullptr, &pending_response_handle);
    request.m_handle = nullptr;

    if (result == IOX2_OK) {
        return iox::ok(PendingResponse<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>(
            pending_response_handle));
    }
    return iox::err(iox::into<RequestSendError>(result));
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
inline void RequestMut<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>::drop() {
    if (m_handle != nullptr) {
        iox2_request_mut_drop(m_handle);
        m_handle = nullptr;
    }
}

} // namespace iox2

#endif
