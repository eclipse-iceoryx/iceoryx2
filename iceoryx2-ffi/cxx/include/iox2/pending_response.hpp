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

#ifndef IOX2_PENDING_RESPONSE_HPP
#define IOX2_PENDING_RESPONSE_HPP

#include "iox/assertions_addendum.hpp"
#include "iox/expected.hpp"
#include "iox/optional.hpp"
#include "iox/slice.hpp"
#include "iox2/header_request_response.hpp"
#include "iox2/payload_info.hpp"
#include "iox2/response.hpp"
#include "iox2/service_type.hpp"

namespace iox2 {
template <ServiceType, typename, typename, typename, typename>
class RequestMut;

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
    using ValueType = typename PayloadInfo<RequestPayload>::ValueType;

  public:
    PendingResponse(PendingResponse&& rhs) noexcept;
    auto operator=(PendingResponse&& rhs) noexcept -> PendingResponse&;
    ~PendingResponse() noexcept;

    PendingResponse(const PendingResponse&) = delete;
    auto operator=(const PendingResponse&) -> PendingResponse& = delete;

    auto operator*() const -> const RequestPayload&;
    auto operator->() const -> const RequestPayload*;

    /// Receives a [`Response`] from one of the [`Server`](crate::port::server::Server)s that
    /// received the [`RequestMut`].
    auto receive() -> iox::expected<iox::optional<Response<Service, ResponsePayload, ResponseHeader>>, ReceiveError>;

    /// Returns a reference to the iceoryx2 internal
    /// [`service::header::request_response::RequestHeader`] of the corresponding
    /// [`RequestMut`]
    auto header() -> RequestHeaderRequestResponse&;

    /// Returns a reference to the user defined request header of the corresponding
    /// [`RequestMut`]
    template <typename T = RequestHeader, typename = std::enable_if_t<!std::is_same_v<void, RequestHeader>, T>>
    auto user_header() -> const T&;

    /// Returns a reference to the request payload of the corresponding
    /// [`RequestMut`]
    template <typename T = RequestPayload, typename = std::enable_if_t<!iox::IsSlice<T>::VALUE, void>>
    auto payload() -> const T&;

    template <typename T = RequestPayload, typename = std::enable_if_t<iox::IsSlice<T>::VALUE, void>>
    auto payload() const -> iox::ImmutableSlice<ValueType>;

    /// Returns how many [`Server`](crate::port::server::Server)s received the corresponding
    /// [`RequestMut`] initially.
    auto number_of_server_connections() const -> size_t;

    /// Returns [`true`] when a [`Server`](crate::port::server::Server) has sent a [`Response`]
    /// otherwise [`false`].
    auto has_response() -> iox::expected<bool, ConnectionFailure>;

    /// Returns [`true`] until the [`ActiveRequest`](crate::active_request::ActiveRequest)
    /// goes out of scope on the [`Server`](crate::port::server::Server)s side indicating that the
    /// [`Server`](crate::port::server::Server) will no longer send [`Response`]s.
    /// It also returns [`false`] when there are no [`Server`](crate::port::server::Server)s.
    auto is_connected() const -> bool;

  private:
    template <ServiceType, typename, typename, typename, typename>
    friend class Client;
    template <ServiceType S,
              typename RequestPayloadT,
              typename RequestHeaderT,
              typename ResponsePayloadT,
              typename ResponseHeaderT>
    friend auto send(RequestMut<S, RequestPayloadT, RequestHeaderT, ResponsePayloadT, ResponseHeaderT>&& request)
        -> iox::expected<PendingResponse<S, RequestPayloadT, RequestHeaderT, ResponsePayloadT, ResponseHeaderT>,
                         RequestSendError>;

    explicit PendingResponse(iox2_pending_response_h handle) noexcept;

    void drop();
    void close();

    iox2_pending_response_h m_handle = nullptr;
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
inline PendingResponse<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::
    ~PendingResponse() noexcept {
    drop();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline auto PendingResponse<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::operator*() const
    -> const RequestPayload& {
    IOX_TODO();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline auto PendingResponse<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::operator->() const
    -> const RequestPayload* {
    IOX_TODO();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline auto PendingResponse<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::receive()
    -> iox::expected<iox::optional<Response<Service, ResponsePayload, ResponseHeader>>, ReceiveError> {
    iox2_response_h response_handle {};
    auto result = iox2_pending_response_receive(&m_handle, nullptr, &response_handle);

    if (result == IOX2_OK) {
        if (response_handle != nullptr) {
            Response<Service, ResponsePayload, ResponseHeader> response(response_handle);
            return iox::ok(iox::optional<Response<Service, ResponsePayload, ResponseHeader>>(std::move(response)));
        }
        return iox::ok(iox::optional<Response<Service, ResponsePayload, ResponseHeader>>(iox::nullopt));
    }
    return iox::err(iox::into<ReceiveError>(result));
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
template <typename T, typename>
inline auto PendingResponse<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::payload()
    -> const T& {
    const void* ptr = nullptr;
    size_t number_of_elements = 0;

    iox2_pending_response_payload(&m_handle, &ptr, &number_of_elements);
    return *static_cast<const T*>(ptr);
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
template <typename T, typename>
inline auto PendingResponse<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::payload() const
    -> iox::ImmutableSlice<ValueType> {
    const void* ptr = nullptr;
    size_t number_of_elements = 0;

    iox2_pending_response_payload(&m_handle, &ptr, &number_of_elements);

    return iox::ImmutableSlice<ValueType>(static_cast<const ValueType*>(ptr), number_of_elements);
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
inline PendingResponse<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::PendingResponse(
    iox2_pending_response_h handle) noexcept
    : m_handle { handle } {
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline void PendingResponse<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::drop() {
    if (m_handle != nullptr) {
        iox2_pending_response_drop(m_handle);
        m_handle = nullptr;
    }
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
