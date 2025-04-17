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

#ifndef ACTIVE_REQUEST_HPP
#define ACTIVE_REQUEST_HPP

#include "iox2/response_mut_uninit.hpp"
#include "iox2/service_type.hpp"

namespace iox2 {
/// Represents a one-to-one connection to a [`Client`](crate::port::client::Client)
/// holding the corresponding
/// [`PendingResponse`](crate::pending_response::PendingResponse) that is coupled
/// with the [`RequestMut`](crate::request_mut::RequestMut) the
/// [`Client`](crate::port::client::Client) sent to the
/// [`Server`](crate::port::server::Server).
/// The [`Server`](crate::port::server::Server) will use it to send arbitrary many
/// [`Response`](crate::response::Response)s.
template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
class ActiveRequest {
  public:
    ActiveRequest(ActiveRequest&& rhs) noexcept;
    auto operator=(ActiveRequest&& rhs) noexcept -> ActiveRequest&;
    ~ActiveRequest() noexcept;

    ActiveRequest(const ActiveRequest&) = delete;
    auto operator=(const ActiveRequest&) noexcept -> ActiveRequest& = delete;

    auto operator*() const -> const RequestPayload&;
    auto operator->() const -> const RequestPayload*;

    auto loan_uninit() -> iox::expected<ResponseMutUninit<Service, ResponsePayload, ResponseHeader>, LoanError>;

    auto send_copy(const ResponsePayload& value) const -> iox::expected<void, SendError>;

    auto payload() const -> const RequestPayload&;

    template <typename T = RequestHeader, typename = std::enable_if_t<!std::is_same_v<void, RequestHeader>, T>>
    auto user_header() const -> const T&;

    auto header() const -> RequestHeaderRequestResponse&;

    auto origin() const -> UniqueClientId;

    auto is_connected() const -> bool;

    auto loan() -> iox::expected<ResponseMut<Service, ResponsePayload, ResponseHeader>, LoanError>;

  private:
    explicit ActiveRequest() noexcept;

    void drop();
    void finish();
};

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline ActiveRequest<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::ActiveRequest(
    ActiveRequest&& rhs) noexcept {
    *this = std::move(rhs);
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline auto ActiveRequest<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::operator=(
    ActiveRequest&& rhs) noexcept -> ActiveRequest& {
    IOX_TODO();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline ActiveRequest<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::
    ~ActiveRequest() noexcept {
    drop();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline auto ActiveRequest<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::operator*() const
    -> const RequestPayload& {
    return payload();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline auto ActiveRequest<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::operator->() const
    -> const RequestPayload* {
    return &payload();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline auto ActiveRequest<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::loan_uninit()
    -> iox::expected<ResponseMutUninit<Service, ResponsePayload, ResponseHeader>, LoanError> {
    IOX_TODO();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline auto ActiveRequest<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::send_copy(
    const ResponsePayload& value) const -> iox::expected<void, SendError> {
    IOX_TODO();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline auto ActiveRequest<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::payload() const
    -> const RequestPayload& {
    IOX_TODO();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
template <typename T, typename>
inline auto ActiveRequest<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::user_header() const
    -> const T& {
    IOX_TODO();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline auto ActiveRequest<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::header() const
    -> RequestHeaderRequestResponse& {
    IOX_TODO();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline auto ActiveRequest<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::origin() const
    -> UniqueClientId {
    IOX_TODO();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline auto ActiveRequest<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::is_connected() const
    -> bool {
    IOX_TODO();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline auto ActiveRequest<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::loan()
    -> iox::expected<ResponseMut<Service, ResponsePayload, ResponseHeader>, LoanError> {
    IOX_TODO();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline ActiveRequest<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::
    ActiveRequest() noexcept {
    IOX_TODO();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline void ActiveRequest<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::drop() {
    IOX_TODO();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline void ActiveRequest<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::finish() {
    IOX_TODO();
}
} // namespace iox2

#endif
