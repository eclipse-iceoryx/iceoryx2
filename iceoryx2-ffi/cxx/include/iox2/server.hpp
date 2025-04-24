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

#ifndef SERVER_HPP
#define SERVER_HPP

#include "iox/expected.hpp"
#include "iox2/active_request.hpp"
#include "iox2/service_type.hpp"
#include "iox2/unique_port_id.hpp"

namespace iox2 {
/// Receives [`RequestMut`](crate::request_mut::RequestMut) from a
/// [`Client`](crate::port::client::Client) and responds with
/// [`Response`](crate::response::Response) by using an
/// [`ActiveRequest`].
template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
class Server {
  public:
    Server(Server&& rhs) noexcept;
    auto operator=(Server&& rhs) noexcept -> Server&;
    ~Server() noexcept;

    Server(const Server&) noexcept = delete;
    auto operator=(const Server&) noexcept -> Server& = delete;

    /// Receives a [`RequestMut`](crate::request_mut::RequestMut) that was sent by a
    /// [`Client`](crate::port::client::Client) and returns an [`ActiveRequest`] which
    /// can be used to respond.
    /// If no [`RequestMut`](crate::request_mut::RequestMut)s were received it
    /// returns [`None`].
    auto receive() -> iox::expected<
        iox::optional<ActiveRequest<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>>,
        ReceiveError>;

    /// Returns the [`UniqueServerId`] of the [`Server`]
    auto id() const -> UniqueServerId;

    /// Returns true if the [`Server`] has [`RequestMut`](crate::request_mut::RequestMut)s in its buffer.
    auto has_requests() const -> iox::expected<bool, ConnectionFailure>;

  private:
    explicit Server() noexcept;

    void drop();
};

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline Server<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::Server(Server&& rhs) noexcept {
    *this = std::move(rhs);
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline auto
Server<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::operator=(Server&& rhs) noexcept
    -> Server& {
    IOX_TODO();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline Server<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::~Server() noexcept {
    drop();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline auto Server<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::receive() -> iox::expected<
    iox::optional<ActiveRequest<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>>,
    ReceiveError> {
    IOX_TODO();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline auto Server<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::id() const
    -> UniqueServerId {
    IOX_TODO();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline auto Server<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::has_requests() const
    -> iox::expected<bool, ConnectionFailure> {
    IOX_TODO();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline Server<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::Server() noexcept {
    IOX_TODO();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline void Server<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::drop() {
    IOX_TODO();
}
} // namespace iox2
#endif

