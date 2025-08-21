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

#ifndef IOX2_SERVER_HPP
#define IOX2_SERVER_HPP

#include "iox/expected.hpp"
#include "iox/slice.hpp"
#include "iox2/active_request.hpp"
#include "iox2/service_type.hpp"
#include "iox2/unique_port_id.hpp"

namespace iox2 {
/// Receives [`RequestMut`] from a [`Client`] and responds with
/// [`Response`] by using an [`ActiveRequest`].
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

    /// Receives a [`RequestMut`] that was sent by a [`Client`] and returns an
    /// [`ActiveRequest`] which can be used to respond.
    /// If no [`RequestMut`]s were received it returns [`None`].
    auto receive() -> iox::expected<
        iox::optional<ActiveRequest<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>>,
        ReceiveError>;

    /// Returns the maximum initial slice length configured for this [`Server`].
    template <typename T = ResponsePayload, typename = std::enable_if_t<iox::IsSlice<T>::VALUE, void>>
    auto initial_max_slice_len() const -> uint64_t;

    /// Returns the [`UniqueServerId`] of the [`Server`]
    auto id() const -> UniqueServerId;

    /// Returns true if the [`Server`] has [`RequestMut`]s in its buffer.
    auto has_requests() const -> iox::expected<bool, ConnectionFailure>;

  private:
    template <ServiceType, typename, typename, typename, typename>
    friend class PortFactoryServer;

    explicit Server(iox2_server_h handle) noexcept;

    void drop();

    iox2_server_h m_handle = nullptr;
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
    if (this != &rhs) {
        drop();
        m_handle = std::move(rhs.m_handle);
        rhs.m_handle = nullptr;
    }

    return *this;
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
    iox2_active_request_h active_request_handle {};
    auto result = iox2_server_receive(&m_handle, nullptr, &active_request_handle);

    if (result == IOX2_OK) {
        if (active_request_handle != nullptr) {
            ActiveRequest<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader> active_request(
                active_request_handle);
            return iox::ok(
                iox::optional<ActiveRequest<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>>(
                    std::move(active_request)));
        }
        return iox::ok(
            iox::optional<ActiveRequest<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>>(
                iox::nullopt));
    }
    return iox::err(iox::into<ReceiveError>(result));
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
template <typename T, typename>
inline auto
Server<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::initial_max_slice_len() const
    -> uint64_t {
    return iox2_server_initial_max_slice_len(&m_handle);
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline auto Server<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::id() const
    -> UniqueServerId {
    iox2_unique_server_id_h id_handle = nullptr;
    iox2_server_id(&m_handle, nullptr, &id_handle);
    return UniqueServerId { id_handle };
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline auto Server<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::has_requests() const
    -> iox::expected<bool, ConnectionFailure> {
    bool has_requests_result = false;
    auto result = iox2_server_has_requests(&m_handle, &has_requests_result);

    if (result == IOX2_OK) {
        return iox::ok(has_requests_result);
    }

    return iox::err(iox::into<ConnectionFailure>(result));
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline Server<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::Server(
    iox2_server_h handle) noexcept
    : m_handle { handle } {
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline void Server<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::drop() {
    if (m_handle != nullptr) {
        iox2_server_drop(m_handle);
        m_handle = nullptr;
    }
}
} // namespace iox2
#endif
