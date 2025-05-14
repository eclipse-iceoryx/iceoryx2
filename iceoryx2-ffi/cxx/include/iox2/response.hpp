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
#include "iox/slice.hpp"
#include "iox2/payload_info.hpp"
#include "iox2/service_type.hpp"

namespace iox2 {
/// It stores the payload and can be received by the [`PendingResponse`] after a
/// [`RequestMut`] was sent to a [`Server`] via the [`Client`].
template <ServiceType Service, typename ResponsePayload, typename ResponseUserHeader>
class Response {
    using ValueType = typename PayloadInfo<ResponsePayload>::ValueType;

  public:
    Response(Response&& rhs) noexcept;
    auto operator=(Response&& rhs) noexcept -> Response&;
    ~Response() noexcept;

    Response(const Response&) noexcept = delete;
    auto operator=(const Response&) noexcept -> Response& = delete;

    auto operator*() const -> const ResponsePayload&;
    auto operator->() const -> const ResponsePayload*;

    /// Returns a reference to the [`ResponseHeader`].
    auto header() const -> ResponseHeader;

    /// Returns a reference to the user header of the response.
    template <typename T = ResponseUserHeader,
              typename = std::enable_if_t<!std::is_same_v<void, ResponseUserHeader>, T>>
    auto user_header() const -> const T&;

    /// Returns a reference to the payload of the response.
    template <typename T = ResponsePayload, typename = std::enable_if_t<!iox::IsSlice<T>::VALUE, void>>
    auto payload() const -> const T&;

    /// Returns a reference to the payload of the response.
    template <typename T = ResponsePayload, typename = std::enable_if_t<iox::IsSlice<T>::VALUE, void>>
    auto payload() const -> iox::ImmutableSlice<ValueType>;

    /// Returns the [`UniqueServerId`] of the [`Server`] which sent
    /// the [`Response`].
    auto origin() const -> UniqueServerId;

  private:
    template <ServiceType, typename, typename, typename, typename>
    friend class PendingResponse;

    explicit Response(iox2_response_h handle) noexcept;

    void drop();

    iox2_response_h m_handle = nullptr;
};

template <ServiceType Service, typename ResponsePayload, typename ResponseUserHeader>
inline Response<Service, ResponsePayload, ResponseUserHeader>::Response(Response&& rhs) noexcept {
    *this = std::move(rhs);
}

template <ServiceType Service, typename ResponsePayload, typename ResponseUserHeader>
inline auto Response<Service, ResponsePayload, ResponseUserHeader>::operator=(Response&& rhs) noexcept -> Response& {
    if (this != &rhs) {
        drop();
        m_handle = std::move(rhs.m_handle);
        rhs.m_handle = nullptr;
    }

    return *this;
}

template <ServiceType Service, typename ResponsePayload, typename ResponseUserHeader>
inline Response<Service, ResponsePayload, ResponseUserHeader>::~Response() noexcept {
    drop();
}

template <ServiceType Service, typename ResponsePayload, typename ResponseUserHeader>
inline auto Response<Service, ResponsePayload, ResponseUserHeader>::operator*() const -> const ResponsePayload& {
    return payload();
}

template <ServiceType Service, typename ResponsePayload, typename ResponseUserHeader>
inline auto Response<Service, ResponsePayload, ResponseUserHeader>::operator->() const -> const ResponsePayload* {
    return &payload();
}

template <ServiceType Service, typename ResponsePayload, typename ResponseUserHeader>
inline auto Response<Service, ResponsePayload, ResponseUserHeader>::header() const -> ResponseHeader {
    iox2_response_header_h header_handle = nullptr;
    iox2_response_header(&m_handle, nullptr, &header_handle);
    return ResponseHeader { header_handle };
}

template <ServiceType Service, typename ResponsePayload, typename ResponseUserHeader>
template <typename T, typename>
inline auto Response<Service, ResponsePayload, ResponseUserHeader>::user_header() const -> const T& {
    const void* ptr = nullptr;
    iox2_response_user_header(&m_handle, &ptr);
    return *static_cast<const T*>(ptr);
}

template <ServiceType Service, typename ResponsePayload, typename ResponseUserHeader>
template <typename T, typename>
inline auto Response<Service, ResponsePayload, ResponseUserHeader>::payload() const -> const T& {
    const void* ptr = nullptr;
    iox2_response_payload(&m_handle, &ptr, nullptr);
    return *static_cast<const T*>(ptr);
}

template <ServiceType Service, typename ResponsePayload, typename ResponseUserHeader>
template <typename T, typename>
inline auto Response<Service, ResponsePayload, ResponseUserHeader>::payload() const -> iox::ImmutableSlice<ValueType> {
    const void* ptr = nullptr;
    size_t number_of_elements = 0;
    iox2_response_payload(&m_handle, &ptr, &number_of_elements);
    return iox::ImmutableSlice<ValueType>(static_cast<const ValueType*>(ptr), number_of_elements);
}

template <ServiceType Service, typename ResponsePayload, typename ResponseUserHeader>
inline auto Response<Service, ResponsePayload, ResponseUserHeader>::origin() const -> UniqueServerId {
    return header().server_port_id();
}

template <ServiceType Service, typename ResponsePayload, typename ResponseUserHeader>
inline Response<Service, ResponsePayload, ResponseUserHeader>::Response(iox2_response_h handle) noexcept
    : m_handle(handle) {
}

template <ServiceType Service, typename ResponsePayload, typename ResponseUserHeader>
inline void Response<Service, ResponsePayload, ResponseUserHeader>::drop() {
    if (m_handle != nullptr) {
        iox2_response_drop(m_handle);
        m_handle = nullptr;
    }
}
} // namespace iox2

#endif
