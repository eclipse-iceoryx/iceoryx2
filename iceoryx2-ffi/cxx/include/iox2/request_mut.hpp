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

#include "iox/assertions_addendum.hpp"
#include "iox/expected.hpp"
#include "iox2/header_request_response.hpp"
#include "iox2/pending_response.hpp"
#include "iox2/port_error.hpp"
#include "iox2/service_type.hpp"

namespace iox2 {

/// The [`RequestMut`] represents the object that contains the payload that the
/// [`Client`](crate::port::client::Client) sends to the
/// [`Server`](crate::port::server::Server).
template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
class RequestMut {
  public:
    RequestMut(RequestMut&& rhs) noexcept;
    auto operator=(RequestMut&& rhs) noexcept -> RequestMut&;
    ~RequestMut() noexcept;

    RequestMut(const RequestMut&) = delete;
    auto operator=(const RequestMut&) -> RequestMut& = delete;

    auto operator*() const -> const RequestPayload&;
    auto operator*() -> RequestPayload&;
    auto operator->() const -> const RequestPayload*;
    auto operator->() -> RequestPayload*;

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
    auto payload() const -> const RequestPayload&;

    /// Returns a mutable reference to the user defined request payload.
    auto payload_mut() -> RequestPayload&;

    /// Sends the [`RequestMut`] to all connected
    /// [`Server`](crate::port::server::Server)s of the
    /// [`Service`](crate::service::Service).
    auto send()
        -> iox::expected<PendingResponse<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>,
                         RequestSendError>;

  private:
    template <ServiceType, typename, typename, typename, typename>
    friend class RequestMutUninit;

    explicit RequestMut();
    void drop();

    // iox2_request_mut_t m_response;
    // iox2_request_mut_h m_handle = nullptr;
};

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline RequestMut<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::RequestMut(
    RequestMut&& rhs) noexcept {
    *this = std::move(rhs);
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline auto RequestMut<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::operator=(
    [[maybe_unused]] RequestMut&& rhs) noexcept -> RequestMut& {
    IOX_TODO();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline RequestMut<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::~RequestMut() noexcept {
    drop();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline auto RequestMut<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::operator*() const
    -> const RequestPayload& {
    return payload();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline auto RequestMut<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::operator*()
    -> RequestPayload& {
    return payload_mut();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline auto RequestMut<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::operator->() const
    -> const RequestPayload* {
    return &payload();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline auto RequestMut<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::operator->()
    -> RequestPayload* {
    return &payload_mut();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline auto RequestMut<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::header() const
    -> RequestHeaderRequestResponse& {
    IOX_TODO();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
template <typename T, typename>
inline auto RequestMut<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::user_header() const
    -> const T& {
    IOX_TODO();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
template <typename T, typename>
inline auto RequestMut<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::user_header_mut()
    -> T& {
    IOX_TODO();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline auto RequestMut<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::payload() const
    -> const RequestPayload& {
    IOX_TODO();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline auto RequestMut<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::payload_mut()
    -> RequestPayload& {
    IOX_TODO();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline auto RequestMut<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::send()
    -> iox::expected<PendingResponse<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>,
                     RequestSendError> {
    IOX_TODO();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline RequestMut<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::RequestMut() {
    IOX_TODO();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline void RequestMut<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::drop() {
    IOX_TODO();
}

} // namespace iox2

#endif
