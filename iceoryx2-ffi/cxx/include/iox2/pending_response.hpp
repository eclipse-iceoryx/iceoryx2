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

#ifndef PENDING_RESPONSE_HPP
#define PENDING_RESPONSE_HPP

#include "iox/assertions_addendum.hpp"
#include "iox/expected.hpp"
#include "iox/optional.hpp"
#include "iox2/header_request_response.hpp"
#include "iox2/response.hpp"
#include "iox2/service_type.hpp"

namespace iox2 {
/// Represents an active connection to all [`Server`](crate::port::server::Server)
/// that received the [`RequestMut`]. The
/// [`Client`](crate::port::client::Client) can use it to receive the corresponding
/// [`Response`]s.
///
/// As soon as it goes out of scope, the connections are closed and the
/// [`Server`](crate::port::server::Server)s are informed.
template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
class PendingResponse {
  public:
    PendingResponse(PendingResponse&& rhs) noexcept;
    auto operator=(PendingResponse&& rhs) noexcept -> PendingResponse&;
    ~PendingResponse() noexcept;

    PendingResponse(const PendingResponse&) = delete;
    auto operator=(const PendingResponse&) -> PendingResponse& = delete;

    auto receive() -> iox::expected<iox::optional<Response<Service, ResponsePayload, ResponseHeader>>, ReceiveError>;

    auto header() -> RequestHeaderRequestResponse&;

    template <typename T = RequestHeader, typename = std::enable_if_t<!std::is_same_v<void, RequestHeader>, T>>
    auto user_header() -> const T&;

    auto payload() -> const RequestPayload&;
    auto number_of_server_connections() const -> size_t;
    auto has_response() -> iox::expected<bool, ConnectionFailure>;
    auto is_connected() const -> bool;

  private:
    explicit PendingResponse() noexcept;

    void drop();
    void close();
};

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline PendingResponse<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::PendingResponse(
    PendingResponse&& rhs) noexcept {
    *this = std::move(rhs);
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline auto PendingResponse<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::operator=(
    PendingResponse&& rhs) noexcept -> PendingResponse& {
    IOX_TODO();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline PendingResponse<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::
    ~PendingResponse() noexcept {
    drop();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline auto PendingResponse<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::receive()
    -> iox::expected<iox::optional<Response<Service, ResponsePayload, ResponseHeader>>, ReceiveError> {
    IOX_TODO();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline auto PendingResponse<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::header()
    -> RequestHeaderRequestResponse& {
    IOX_TODO();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
template <typename T, typename>
inline auto PendingResponse<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::user_header()
    -> const T& {
    IOX_TODO();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline auto PendingResponse<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::payload()
    -> const RequestPayload& {
    IOX_TODO();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline auto
PendingResponse<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::number_of_server_connections()
    const -> size_t {
    IOX_TODO();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline auto PendingResponse<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::has_response()
    -> iox::expected<bool, ConnectionFailure> {
    IOX_TODO();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline auto
PendingResponse<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::is_connected() const -> bool {
    IOX_TODO();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline PendingResponse<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::
    PendingResponse() noexcept {
    IOX_TODO();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline void PendingResponse<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::drop() {
    IOX_TODO();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline void PendingResponse<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::close() {
    IOX_TODO();
}

} // namespace iox2
#endif
