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
#include "iox2/bb/slice.hpp"
#include "iox2/marker.hpp"
#include "iox2/payload_info.hpp"
#include "iox2/service_type.hpp"

#if IOX2_FEATURE_FLATBUFFERS
#include <flatbuffers/flatbuffers.h>
#endif // IOX2_FEATURE_FLATBUFFERS

#include <type_traits>

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

    /// Returns a reference to the [`ResponseHeader`].
    auto header() const -> ResponseHeader;

    /// Returns a reference to the user header of the response.
    template <typename T = ResponseUserHeader,
              typename = std::enable_if_t<!std::is_same<void, ResponseUserHeader>::value, T>>
    auto user_header() const -> const T&;

    /// Returns a reference to the payload of the response.
    template <typename T = ResponsePayload,
              typename = std::enable_if_t<!bb::IsSlice<T>::VALUE && !has_flatbuffer_marker<T>(), void>>
    auto payload() const -> const T&;

    /// Returns a reference to the payload of the response.
    template <typename T = ResponsePayload, typename = std::enable_if_t<bb::IsSlice<T>::VALUE, void>>
    auto payload() const -> bb::ImmutableSlice<ValueType>;

    /// Returns the [`UniqueServerId`] of the [`Server`] which sent
    /// the [`Response`].
    auto origin() const -> UniqueServerId;

#if IOX2_FEATURE_FLATBUFFERS
    /// Returns the serialized flatbuffer data as bytes.
    template <typename T = ResponsePayload, typename = std::enable_if_t<has_flatbuffer_marker<T>(), void>>
    auto payload_bytes() const -> bb::ImmutableSlice<uint8_t>;

    /// Returns the root of the flatbuffer.
    template <typename T = ResponsePayload, typename = std::enable_if_t<has_flatbuffer_marker<T>(), void>>
    auto payload_root() const -> const typename T::ValueType*;
#endif // IOX2_FEATURE_FLATBUFFERS

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
        m_handle = rhs.m_handle;
        rhs.m_handle = nullptr;
    }

    return *this;
}

template <ServiceType Service, typename ResponsePayload, typename ResponseUserHeader>
inline Response<Service, ResponsePayload, ResponseUserHeader>::~Response() noexcept {
    drop();
}

#if IOX2_FEATURE_FLATBUFFERS
template <ServiceType Service, typename ResponsePayload, typename ResponseUserHeader>
template <typename T, typename>
inline auto Response<Service, ResponsePayload, ResponseUserHeader>::payload_bytes() const
    -> bb::ImmutableSlice<uint8_t> {
    const void* ptr = nullptr;
    size_t number_of_elements = 0;

    iox2_response_payload(&m_handle, &ptr, &number_of_elements);
    auto payload_offset = header().payload_offset();
    auto payload_len = header().number_of_elements();

    return bb::ImmutableSlice<uint8_t>(static_cast<const uint8_t*>(ptr) + payload_offset, payload_len - payload_offset);
}

template <ServiceType Service, typename ResponsePayload, typename ResponseUserHeader>
template <typename T, typename>
inline auto Response<Service, ResponsePayload, ResponseUserHeader>::payload_root() const -> const
    typename T::ValueType* {
    return flatbuffers::GetRoot<typename T::ValueType>(payload_bytes().data());
}
#endif // IOX2_FEATURE_FLATBUFFERS

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
inline auto Response<Service, ResponsePayload, ResponseUserHeader>::payload() const -> bb::ImmutableSlice<ValueType> {
    const void* ptr = nullptr;
    size_t number_of_elements = 0;
    iox2_response_payload(&m_handle, &ptr, &number_of_elements);

    // for the custom payload marker, the slice length is the
    // runtime payload byte size
    auto length = number_of_elements;
    if (std::is_same<ValueType, CustomPayloadMarker>::value) {
        length = iox2_response_payload_number_of_bytes(&m_handle);
    }

    return bb::ImmutableSlice<ValueType>(static_cast<const ValueType*>(ptr), length);
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
