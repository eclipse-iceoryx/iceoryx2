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

/// Represents an active connection to all [`Server`] that received the
/// [`RequestMut`]. The [`Client`] can use it to receive the corresponding
/// [`Response`]s.
///
/// As soon as it goes out of scope, the connections are closed and the
/// [`Server`]s are informed.
template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
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

    /// Receives a [`Response`] from one of the [`Server`]s that
    /// received the [`RequestMut`].
    auto receive()
        -> iox::expected<iox::optional<Response<Service, ResponsePayload, ResponseUserHeader>>, ReceiveError>;

    /// Returns a reference to the iceoryx2 internal [`RequestHeader`] of
    /// the corresponding [`RequestMut`]
    auto header() -> RequestHeader;

    /// Returns a reference to the user defined request header of the corresponding
    /// [`RequestMut`]
    template <typename T = RequestUserHeader, typename = std::enable_if_t<!std::is_same_v<void, RequestUserHeader>, T>>
    auto user_header() -> const T&;

    /// Returns a reference to the request payload of the corresponding
    /// [`RequestMut`]
    template <typename T = RequestPayload, typename = std::enable_if_t<!iox::IsSlice<T>::VALUE, void>>
    auto payload() const -> const T&;

    /// Returns a reference to the request payload of the corresponding
    /// [`RequestMut`]
    template <typename T = RequestPayload, typename = std::enable_if_t<iox::IsSlice<T>::VALUE, void>>
    auto payload() const -> iox::ImmutableSlice<ValueType>;

    /// Returns how many [`Server`]s received the corresponding
    /// [`RequestMut`] initially.
    auto number_of_server_connections() const -> size_t;

    /// Returns [`true`] when a [`Server`] has sent a [`Response`]
    /// otherwise [`false`].
    auto has_response() -> bool;

    /// Returns [`true`] until the [`ActiveRequest`] goes out of scope on the
    /// [`Server`]s side indicating that the [`Server`] will no longer send [`Response`]s.
    /// It also returns [`false`] when there are no [`Server`]s.
    auto is_connected() const -> bool;

    /// Marks the connection state that the [`Client`] wants to gracefully
    /// disconnect. When the [`Server`] reads this, it can send the last [`Response`] and drop the
    /// corresponding [`ActiveRequest`] to terminate the
    /// connection ensuring that no [`Response`] is lost on the [`Client`]
    /// side.
    void set_disconnect_hint();

  private:
    template <ServiceType, typename, typename, typename, typename>
    friend class Client;
    template <ServiceType S,
              typename RequestPayloadT,
              typename RequestUserHeaderT,
              typename ResponsePayloadT,
              typename ResponseUserHeaderT>
    friend auto
    send(RequestMut<S, RequestPayloadT, RequestUserHeaderT, ResponsePayloadT, ResponseUserHeaderT>&& request)
        -> iox::expected<PendingResponse<S, RequestPayloadT, RequestUserHeaderT, ResponsePayloadT, ResponseUserHeaderT>,
                         RequestSendError>;

    explicit PendingResponse(iox2_pending_response_h handle) noexcept;

    void drop();

    iox2_pending_response_h m_handle = nullptr;
};

template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
inline PendingResponse<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>::
    PendingResponse(PendingResponse&& rhs) noexcept {
    *this = std::move(rhs);
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
inline auto PendingResponse<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>::operator=(
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
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
inline PendingResponse<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>::
    ~PendingResponse() noexcept {
    drop();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
inline auto
PendingResponse<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>::operator*() const
    -> const RequestPayload& {
    return payload();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
inline auto
PendingResponse<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>::operator->() const
    -> const RequestPayload* {
    return &payload();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
inline auto PendingResponse<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>::receive()
    -> iox::expected<iox::optional<Response<Service, ResponsePayload, ResponseUserHeader>>, ReceiveError> {
    iox2_response_h response_handle {};
    auto result = iox2_pending_response_receive(&m_handle, nullptr, &response_handle);

    if (result == IOX2_OK) {
        if (response_handle != nullptr) {
            Response<Service, ResponsePayload, ResponseUserHeader> response(response_handle);
            return iox::ok(iox::optional<Response<Service, ResponsePayload, ResponseUserHeader>>(std::move(response)));
        }
        return iox::ok(iox::optional<Response<Service, ResponsePayload, ResponseUserHeader>>(iox::nullopt));
    }
    return iox::err(iox::into<ReceiveError>(result));
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
inline auto PendingResponse<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>::header()
    -> RequestHeader {
    iox2_request_header_h header_handle = nullptr;
    iox2_pending_response_header(&m_handle, nullptr, &header_handle);
    return RequestHeader { header_handle };
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
template <typename T, typename>
inline auto
PendingResponse<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>::user_header()
    -> const T& {
    const void* ptr = nullptr;
    iox2_pending_response_user_header(&m_handle, &ptr);
    return *static_cast<const T*>(ptr);
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
template <typename T, typename>
inline auto
PendingResponse<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>::payload() const
    -> const T& {
    const void* ptr = nullptr;
    iox2_pending_response_payload(&m_handle, &ptr, nullptr);
    return *static_cast<const T*>(ptr);
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
template <typename T, typename>
inline auto
PendingResponse<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>::payload() const
    -> iox::ImmutableSlice<ValueType> {
    const void* ptr = nullptr;
    size_t number_of_elements = 0;

    iox2_pending_response_payload(&m_handle, &ptr, &number_of_elements);

    return iox::ImmutableSlice<ValueType>(static_cast<const ValueType*>(ptr), number_of_elements);
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
inline auto PendingResponse<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>::
    number_of_server_connections() const -> size_t {
    return iox2_pending_response_number_of_server_connections(&m_handle);
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
inline auto
PendingResponse<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>::has_response()
    -> bool {
    return iox2_pending_response_has_response(&m_handle);
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
inline auto
PendingResponse<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>::is_connected() const
    -> bool {
    return iox2_pending_response_is_connected(&m_handle);
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
inline void PendingResponse<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>::
    set_disconnect_hint() {
    iox2_pending_response_set_disconnect_hint(&m_handle);
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
inline PendingResponse<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>::
    PendingResponse(iox2_pending_response_h handle) noexcept
    : m_handle { handle } {
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
inline void PendingResponse<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>::drop() {
    if (m_handle != nullptr) {
        iox2_pending_response_drop(m_handle);
        m_handle = nullptr;
    }
}
} // namespace iox2
#endif
