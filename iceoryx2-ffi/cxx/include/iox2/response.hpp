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

#ifndef IOX2_RESPONSE_HPP
#define IOX2_RESPONSE_HPP

#include "header_request_response.hpp"
#include "iox/assertions_addendum.hpp"
#include "iox2/service_type.hpp"

namespace iox2 {
/// It stores the payload and can be received by the
/// [`PendingResponse`](crate::pending_response::PendingResponse) after a
/// [`RequestMut`](crate::request_mut::RequestMut) was sent to a
/// [`Server`](crate::port::server::Server) via the [`Client`](crate::port::client::Client).
template <ServiceType Service, typename ResponsePayload, typename ResponseHeader>
class Response {
  public:
    Response(Response&& rhs) noexcept;
    auto operator=(Response&& rhs) noexcept -> Response&;
    ~Response() noexcept;

    Response(const Response&) noexcept = delete;
    auto operator=(const Response&) noexcept -> Response& = delete;

    auto operator*() const -> const ResponsePayload&;
    auto operator->() const -> const ResponsePayload*;

    /// Returns a reference to the
    /// [`ResponseHeader`](service::header::request_response::ResponseHeader).
    auto header() const -> ResponseHeaderRequestResponse&;

    /// Returns a reference to the user header of the response.
    template <typename T = ResponseHeader, typename = std::enable_if_t<!std::is_same_v<void, ResponseHeader>, T>>
    auto user_header() const -> const T&;

    /// Returns a reference to the payload of the response.
    auto payload() const -> const ResponsePayload&;

    /// Returns the [`UniqueServerId`] of the [`Server`](crate::port::server::Server) which sent
    /// the [`Response`].
    auto origin() const -> UniqueServerId;

  private:
    template <ServiceType, typename, typename, typename, typename>
    friend class PendingResponse;

    explicit Response(iox2_response_h handle) noexcept;

    void drop();

    iox2_response_h m_handle = nullptr;
};

template <ServiceType Service, typename ResponsePayload, typename ResponseHeader>
inline Response<Service, ResponsePayload, ResponseHeader>::Response(Response&& rhs) noexcept {
    *this = std::move(rhs);
}

template <ServiceType Service, typename ResponsePayload, typename ResponseHeader>
inline auto Response<Service, ResponsePayload, ResponseHeader>::operator=(Response&& rhs) noexcept -> Response& {
    if (this != &rhs) {
        drop();
        m_handle = std::move(rhs.m_handle);
        rhs.m_handle = nullptr;
    }

    return *this;
}

template <ServiceType Service, typename ResponsePayload, typename ResponseHeader>
inline Response<Service, ResponsePayload, ResponseHeader>::~Response() noexcept {
    drop();
}

template <ServiceType Service, typename ResponsePayload, typename ResponseHeader>
inline auto Response<Service, ResponsePayload, ResponseHeader>::operator*() const -> const ResponsePayload& {
    return payload();
}

template <ServiceType Service, typename ResponsePayload, typename ResponseHeader>
inline auto Response<Service, ResponsePayload, ResponseHeader>::operator->() const -> const ResponsePayload* {
    return &payload();
}

template <ServiceType Service, typename ResponsePayload, typename ResponseHeader>
inline auto Response<Service, ResponsePayload, ResponseHeader>::header() const -> ResponseHeaderRequestResponse& {
    IOX_TODO();
}

template <ServiceType Service, typename ResponsePayload, typename ResponseHeader>
template <typename T, typename>
inline auto Response<Service, ResponsePayload, ResponseHeader>::user_header() const -> const T& {
    IOX_TODO();
}

template <ServiceType Service, typename ResponsePayload, typename ResponseHeader>
inline auto Response<Service, ResponsePayload, ResponseHeader>::payload() const -> const ResponsePayload& {
    const void* ptr = nullptr;
    size_t number_of_elements = 0;
    iox2_response_payload(&m_handle, &ptr, &number_of_elements);
    return *static_cast<const ResponsePayload*>(ptr);
}

template <ServiceType Service, typename ResponsePayload, typename ResponseHeader>
inline auto Response<Service, ResponsePayload, ResponseHeader>::origin() const -> UniqueServerId {
    IOX_TODO();
}

template <ServiceType Service, typename ResponsePayload, typename ResponseHeader>
inline Response<Service, ResponsePayload, ResponseHeader>::Response(iox2_response_h handle) noexcept
    : m_handle(handle) {
}

template <ServiceType Service, typename ResponsePayload, typename ResponseHeader>
inline void Response<Service, ResponsePayload, ResponseHeader>::drop() {
    if (m_handle != nullptr) {
        iox2_response_drop(m_handle);
        m_handle = nullptr;
    }
}
} // namespace iox2

#endif
