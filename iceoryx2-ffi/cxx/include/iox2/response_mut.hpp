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

#ifndef IOX2_RESPONSE_MUT_HPP
#define IOX2_RESPONSE_MUT_HPP

#include "iox/expected.hpp"
#include "iox/slice.hpp"
#include "iox2/header_request_response.hpp"
#include "iox2/payload_info.hpp"
#include "iox2/port_error.hpp"
#include "iox2/service_type.hpp"

namespace iox2 {

/// Acquired by a [`ActiveRequest`] with
///  * [`ActiveRequest::loan()`]
///
/// It stores the payload of the response that will be sent to the corresponding
/// [`PendingResponse`] of the [`Client`].
///
/// If the [`ResponseMut`] is not sent it will reelase the loaned memory when going out of
/// scope.
template <ServiceType Service, typename ResponsePayload, typename ResponseUserHeader>
// NOLINTNEXTLINE(cppcoreguidelines-pro-type-member-init,hicpp-member-init) 'm_response' is not used directly but only via the initialized 'm_handle'; furthermore, it will be initialized on the call site
class ResponseMut {
    using ValueType = typename PayloadInfo<ResponsePayload>::ValueType;

  public:
    ResponseMut(ResponseMut&& rhs) noexcept;
    auto operator=(ResponseMut&& rhs) noexcept -> ResponseMut&;
    ~ResponseMut() noexcept;

    ResponseMut(const ResponseMut&) = delete;
    auto operator=(const ResponseMut&) -> ResponseMut& = delete;

    auto operator*() const -> const ResponsePayload&;
    auto operator*() -> ResponsePayload&;
    auto operator->() const -> const ResponsePayload*;
    auto operator->() -> ResponsePayload*;

    /// Returns a reference to the [`ResponseHeader`].
    auto header() const -> ResponseHeader;

    /// Returns a reference to the user header of the response.
    template <typename T = ResponseUserHeader,
              typename = std::enable_if_t<!std::is_same_v<void, ResponseUserHeader>, T>>
    auto user_header() const -> const T&;

    /// Returns a mutable reference to the user header of the response.
    template <typename T = ResponseUserHeader,
              typename = std::enable_if_t<!std::is_same_v<void, ResponseUserHeader>, T>>
    auto user_header_mut() -> T&;

    /// Returns a reference to the payload of the response.
    template <typename T = ResponsePayload, typename = std::enable_if_t<!iox::IsSlice<T>::VALUE, void>>
    auto payload() const -> const ResponsePayload&;

    /// Returns a reference to the payload of the response.
    template <typename T = ResponsePayload, typename = std::enable_if_t<iox::IsSlice<T>::VALUE, void>>
    auto payload() const -> iox::ImmutableSlice<ValueType>;

    /// Returns a mutable reference to the payload of the response.
    template <typename T = ResponsePayload, typename = std::enable_if_t<!iox::IsSlice<T>::VALUE, void>>
    auto payload_mut() -> ResponsePayload&;

    /// Returns a mutable reference to the payload of the response.
    template <typename T = ResponsePayload, typename = std::enable_if_t<iox::IsSlice<T>::VALUE, void>>
    auto payload_mut() -> iox::MutableSlice<ValueType>;

  private:
    template <ServiceType, typename, typename>
    friend class ResponseMutUninit;
    template <ServiceType, typename, typename, typename, typename>
    friend class ActiveRequest;

    /// Sends a [`ResponseMut`] to the corresponding [`PendingResponse`] of the
    /// [`Client`].
    template <ServiceType S, typename ResponsePayloadT, typename ResponseUserHeaderT>
    friend auto send(ResponseMut<S, ResponsePayloadT, ResponseUserHeaderT>&& response)
        -> iox::expected<void, SendError>;

    explicit ResponseMut() = default;
    void drop();

    iox2_response_mut_t m_response;
    iox2_response_mut_h m_handle = nullptr;
};

// NOLINTNEXTLINE(cppcoreguidelines-pro-type-member-init,hicpp-member-init) m_response will be initialized in the move assignment operator
template <ServiceType Service, typename ResponsePayload, typename ResponseUserHeader>
inline ResponseMut<Service, ResponsePayload, ResponseUserHeader>::ResponseMut(ResponseMut&& rhs) noexcept {
    *this = std::move(rhs);
}

namespace internal {
extern "C" {
void iox2_response_mut_move(iox2_response_mut_t*, iox2_response_mut_t*, iox2_response_mut_h*);
}
} // namespace internal

template <ServiceType Service, typename ResponsePayload, typename ResponseUserHeader>
inline auto ResponseMut<Service, ResponsePayload, ResponseUserHeader>::operator=(ResponseMut&& rhs) noexcept
    -> ResponseMut& {
    if (this != &rhs) {
        drop();

        internal::iox2_response_mut_move(&rhs.m_response, &m_response, &m_handle);
        rhs.m_handle = nullptr;
    }

    return *this;
}

template <ServiceType Service, typename ResponsePayload, typename ResponseUserHeader>
inline ResponseMut<Service, ResponsePayload, ResponseUserHeader>::~ResponseMut() noexcept {
    drop();
}

template <ServiceType Service, typename ResponsePayload, typename ResponseUserHeader>
inline auto ResponseMut<Service, ResponsePayload, ResponseUserHeader>::operator*() const -> const ResponsePayload& {
    return payload();
}

template <ServiceType Service, typename ResponsePayload, typename ResponseUserHeader>
inline auto ResponseMut<Service, ResponsePayload, ResponseUserHeader>::operator*() -> ResponsePayload& {
    return payload_mut();
}

template <ServiceType Service, typename ResponsePayload, typename ResponseUserHeader>
inline auto ResponseMut<Service, ResponsePayload, ResponseUserHeader>::operator->() const -> const ResponsePayload* {
    return &payload();
}

template <ServiceType Service, typename ResponsePayload, typename ResponseUserHeader>
inline auto ResponseMut<Service, ResponsePayload, ResponseUserHeader>::operator->() -> ResponsePayload* {
    return &payload_mut();
}

template <ServiceType Service, typename ResponsePayload, typename ResponseUserHeader>
inline auto ResponseMut<Service, ResponsePayload, ResponseUserHeader>::header() const -> ResponseHeader {
    iox2_response_header_h header_handle = nullptr;
    iox2_response_mut_header(&m_handle, nullptr, &header_handle);
    return ResponseHeader { header_handle };
}

template <ServiceType Service, typename ResponsePayload, typename ResponseUserHeader>
template <typename T, typename>
inline auto ResponseMut<Service, ResponsePayload, ResponseUserHeader>::user_header() const -> const T& {
    const void* ptr = nullptr;
    iox2_response_mut_user_header(&m_handle, &ptr);
    return *static_cast<const T*>(ptr);
}

template <ServiceType Service, typename ResponsePayload, typename ResponseUserHeader>
template <typename T, typename>
inline auto ResponseMut<Service, ResponsePayload, ResponseUserHeader>::user_header_mut() -> T& {
    void* ptr = nullptr;
    iox2_response_mut_user_header_mut(&m_handle, &ptr);
    return *static_cast<T*>(ptr);
}

template <ServiceType Service, typename ResponsePayload, typename ResponseUserHeader>
template <typename T, typename>
inline auto ResponseMut<Service, ResponsePayload, ResponseUserHeader>::payload() const -> const ResponsePayload& {
    const void* ptr = nullptr;
    iox2_response_mut_payload(&m_handle, &ptr, nullptr);
    return *static_cast<const T*>(ptr);
}

template <ServiceType Service, typename ResponsePayload, typename ResponseUserHeader>
template <typename T, typename>
inline auto ResponseMut<Service, ResponsePayload, ResponseUserHeader>::payload() const
    -> iox::ImmutableSlice<ValueType> {
    const void* ptr = nullptr;
    size_t number_of_elements = 0;
    iox2_response_mut_payload(&m_handle, &ptr, &number_of_elements);
    return iox::ImmutableSlice<ValueType>(static_cast<const ValueType*>(ptr), number_of_elements);
}

template <ServiceType Service, typename ResponsePayload, typename ResponseUserHeader>
template <typename T, typename>
inline auto ResponseMut<Service, ResponsePayload, ResponseUserHeader>::payload_mut() -> ResponsePayload& {
    void* ptr = nullptr;
    iox2_response_mut_payload_mut(&m_handle, &ptr, nullptr);
    return *static_cast<ResponsePayload*>(ptr);
}

template <ServiceType Service, typename ResponsePayload, typename ResponseUserHeader>
template <typename T, typename>
inline auto ResponseMut<Service, ResponsePayload, ResponseUserHeader>::payload_mut() -> iox::MutableSlice<ValueType> {
    void* ptr = nullptr;
    size_t number_of_elements = 0;
    iox2_response_mut_payload_mut(&m_handle, &ptr, &number_of_elements);

    return iox::MutableSlice<ValueType>(static_cast<ValueType*>(ptr), number_of_elements);
}


template <ServiceType Service, typename ResponsePayload, typename ResponseUserHeader>
inline auto send(ResponseMut<Service, ResponsePayload, ResponseUserHeader>&& response)
    -> iox::expected<void, SendError> {
    auto result = iox2_response_mut_send(response.m_handle);
    response.m_handle = nullptr;

    if (result == IOX2_OK) {
        return iox::ok();
    }
    return iox::err(iox::into<SendError>(result));
}

template <ServiceType Service, typename ResponsePayload, typename ResponseUserHeader>
inline void ResponseMut<Service, ResponsePayload, ResponseUserHeader>::drop() {
    if (m_handle != nullptr) {
        iox2_response_mut_drop(m_handle);
        m_handle = nullptr;
    }
}

} // namespace iox2

#endif
