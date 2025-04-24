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

#ifndef CLIENT_HPP
#define CLIENT_HPP

#include "iox/expected.hpp"
#include "iox2/request_mut_uninit.hpp"
#include "iox2/service_type.hpp"
#include "iox2/unique_port_id.hpp"

namespace iox2 {
/// Sends [`RequestMut`]s to a [`Server`](crate::port::server::Server) in a
/// request-response based communication.
template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
class Client {
  public:
    Client(Client&& rhs) noexcept;
    auto operator=(Client&& rhs) noexcept -> Client&;
    ~Client() noexcept;

    Client(const Client&) noexcept = delete;
    auto operator=(const Client&) noexcept -> Client& = delete;

    /// Returns the [`UniqueClientId`] of the [`Client`]
    auto id() const -> UniqueClientId;

    /// Returns the strategy the [`Client`] follows when a [`RequestMut`] cannot be delivered
    /// if the [`Server`](crate::port::server::Server)s buffer is full.
    auto unable_to_deliver_strategy() const -> UnableToDeliverStrategy;

    /// Acquires an [`RequestMutUninit`] to store payload. This API shall be used
    /// by default to avoid unnecessary copies.
    auto loan_uninit()
        -> iox::expected<RequestMutUninit<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>,
                         LoanError>;

    /// Copies the input value into a [`RequestMut`] and sends it. On success it
    /// returns a [`PendingResponse`] that can be used to receive a stream of
    /// [`Response`](crate::response::Response)s from the
    /// [`Server`](crate::port::server::Server).
    auto send_copy(const RequestPayload& value) const
        -> iox::expected<PendingResponse<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>,
                         RequestSendError>;

    /// Acquires the payload for the request and initializes the underlying memory
    /// with default. This can be very expensive when the payload is large, therefore
    /// prefer [`Client::loan_uninit()`] when possible.
    auto loan() -> iox::expected<RequestMut<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>,
                                 LoanError>;

  private:
    explicit Client() noexcept;

    void drop();

    // iox2_client_h m_handle = nullptr;
};

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline Client<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::Client(Client&& rhs) noexcept {
    *this = std::move(rhs);
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline auto
Client<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::operator=(Client&& rhs) noexcept
    -> Client& {
    IOX_TODO();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline Client<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::~Client() noexcept {
    drop();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline auto Client<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::id() const
    -> UniqueClientId {
    IOX_TODO();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline auto
Client<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::unable_to_deliver_strategy() const
    -> UnableToDeliverStrategy {
    IOX_TODO();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline auto Client<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::loan_uninit()
    -> iox::expected<RequestMutUninit<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>,
                     LoanError> {
    IOX_TODO();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline auto Client<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::send_copy(
    const RequestPayload& value) const
    -> iox::expected<PendingResponse<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>,
                     RequestSendError> {
    IOX_TODO();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline auto Client<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::loan()
    -> iox::expected<RequestMut<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>, LoanError> {
    IOX_TODO();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline Client<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::Client() noexcept {
    IOX_TODO();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline void Client<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::drop() {
    IOX_TODO();
}
} // namespace iox2

#endif
