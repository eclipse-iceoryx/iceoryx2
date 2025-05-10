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

#include "iox/assertions_addendum.hpp"
#include "iox/expected.hpp"
#include "iox/slice.hpp"
#include "iox2/header_request_response.hpp"
#include "iox2/payload_info.hpp"
#include "iox2/port_error.hpp"
#include "iox2/service_type.hpp"

namespace iox2 {

/// Acquired by a [`ActiveRequest`](crate::active_request::ActiveRequest) with
///  * [`ActiveRequest::loan()`](crate::active_request::ActiveRequest::loan())
///
/// It stores the payload of the response that will be sent to the corresponding
/// [`PendingResponse`](crate::pending_response::PendingResponse) of the
/// [`Client`](crate::port::client::Client).
///
/// If the [`ResponseMut`] is not sent it will reelase the loaned memory when going out of
/// scope.
template <ServiceType Service, typename ResponsePayload, typename ResponseHeader>
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

    /// Returns a reference to the
    /// [`ResponseHeader`](service::header::request_response::ResponseHeader).
    auto header() const -> ResponseHeaderRequestResponse&;

    /// Returns a reference to the user header of the response.
    template <typename T = ResponseHeader, typename = std::enable_if_t<!std::is_same_v<void, ResponseHeader>, T>>
    auto user_header() const -> const T&;

    /// Returns a mutable reference to the user header of the response.
    template <typename T = ResponseHeader, typename = std::enable_if_t<!std::is_same_v<void, ResponseHeader>, T>>
    auto user_header_mut() -> T&;

    /// Returns a reference to the payload of the response.
    template <typename T = ResponsePayload, typename = std::enable_if_t<!iox::IsSlice<T>::VALUE, void>>
    auto payload() const -> const ResponsePayload&;

    template <typename T = ResponsePayload, typename = std::enable_if_t<iox::IsSlice<T>::VALUE, void>>
    auto payload() const -> iox::ImmutableSlice<ValueType>;

    /// Returns a mutable reference to the payload of the response.
    template <typename T = ResponsePayload, typename = std::enable_if_t<!iox::IsSlice<T>::VALUE, void>>
    auto payload_mut() -> ResponsePayload&;

    template <typename T = ResponsePayload, typename = std::enable_if_t<iox::IsSlice<T>::VALUE, void>>
    auto payload_mut() -> iox::MutableSlice<ValueType>;

  private:
    template <ServiceType, typename, typename>
    friend class ResponseMutUninit;
    template <ServiceType, typename, typename, typename, typename>
    friend class ActiveRequest;

    /// Sends a [`ResponseMut`] to the corresponding
    /// [`PendingResponse`](crate::pending_response::PendingResponse) of the
    /// [`Client`](crate::port::client::Client).
    template <ServiceType S, typename ResponsePayloadT, typename ResponseHeaderT>
    friend auto send(ResponseMut<S, ResponsePayloadT, ResponseHeaderT>&& response) -> iox::expected<void, SendError>;

    explicit ResponseMut() = default;
    void drop();

    iox2_response_mut_t m_response;
    iox2_response_mut_h m_handle = nullptr;
};

// NOLINTNEXTLINE(cppcoreguidelines-pro-type-member-init,hicpp-member-init) m_response will be initialized in the move assignment operator
template <ServiceType Service, typename ResponsePayload, typename ResponseHeader>
inline ResponseMut<Service, ResponsePayload, ResponseHeader>::ResponseMut(ResponseMut&& rhs) noexcept {
    *this = std::move(rhs);
}

namespace internal {
extern "C" {
void iox2_response_mut_move(iox2_response_mut_t*, iox2_response_mut_t*, iox2_response_mut_h*);
}
} // namespace internal

template <ServiceType Service, typename ResponsePayload, typename ResponseHeader>
inline auto ResponseMut<Service, ResponsePayload, ResponseHeader>::operator=(ResponseMut&& rhs) noexcept
    -> ResponseMut& {
    if (this != &rhs) {
        drop();

        internal::iox2_response_mut_move(&rhs.m_response, &m_response, &m_handle);
        rhs.m_handle = nullptr;
    }

    return *this;
}

template <ServiceType Service, typename ResponsePayload, typename ResponseHeader>
inline ResponseMut<Service, ResponsePayload, ResponseHeader>::~ResponseMut() noexcept {
    drop();
}

template <ServiceType Service, typename ResponsePayload, typename ResponseHeader>
inline auto ResponseMut<Service, ResponsePayload, ResponseHeader>::operator*() const -> const ResponsePayload& {
    return payload();
}

template <ServiceType Service, typename ResponsePayload, typename ResponseHeader>
inline auto ResponseMut<Service, ResponsePayload, ResponseHeader>::operator*() -> ResponsePayload& {
    return payload_mut();
}

template <ServiceType Service, typename ResponsePayload, typename ResponseHeader>
inline auto ResponseMut<Service, ResponsePayload, ResponseHeader>::operator->() const -> const ResponsePayload* {
    return &payload();
}

template <ServiceType Service, typename ResponsePayload, typename ResponseHeader>
inline auto ResponseMut<Service, ResponsePayload, ResponseHeader>::operator->() -> ResponsePayload* {
    return &payload_mut();
}

template <ServiceType Service, typename ResponsePayload, typename ResponseHeader>
inline auto ResponseMut<Service, ResponsePayload, ResponseHeader>::header() const -> ResponseHeaderRequestResponse& {
    IOX_TODO();
}

template <ServiceType Service, typename ResponsePayload, typename ResponseHeader>
template <typename T, typename>
inline auto ResponseMut<Service, ResponsePayload, ResponseHeader>::user_header() const -> const T& {
    IOX_TODO();
}

template <ServiceType Service, typename ResponsePayload, typename ResponseHeader>
template <typename T, typename>
inline auto ResponseMut<Service, ResponsePayload, ResponseHeader>::user_header_mut() -> T& {
    void* ptr = nullptr;
    iox2_response_mut_user_header_mut(&m_handle, &ptr);
    return *static_cast<T*>(ptr);
}

template <ServiceType Service, typename ResponsePayload, typename ResponseHeader>
template <typename T, typename>
inline auto ResponseMut<Service, ResponsePayload, ResponseHeader>::payload() const -> const ResponsePayload& {
    IOX_TODO();
}

template <ServiceType Service, typename ResponsePayload, typename ResponseHeader>
template <typename T, typename>
inline auto ResponseMut<Service, ResponsePayload, ResponseHeader>::payload() const -> iox::ImmutableSlice<ValueType> {
    IOX_TODO();
}

template <ServiceType Service, typename ResponsePayload, typename ResponseHeader>
template <typename T, typename>
inline auto ResponseMut<Service, ResponsePayload, ResponseHeader>::payload_mut() -> ResponsePayload& {
    void* ptr = nullptr;
    iox2_response_mut_payload_mut(&m_handle, &ptr, nullptr);
    return *static_cast<ResponsePayload*>(ptr);
}

template <ServiceType Service, typename ResponsePayload, typename ResponseHeader>
template <typename T, typename>
inline auto ResponseMut<Service, ResponsePayload, ResponseHeader>::payload_mut() -> iox::MutableSlice<ValueType> {
    void* ptr = nullptr;
    size_t number_of_elements = 0;
    iox2_response_mut_payload_mut(&m_handle, &ptr, &number_of_elements);

    return iox::MutableSlice<ValueType>(static_cast<ValueType*>(ptr), number_of_elements);
}


template <ServiceType Service, typename ResponsePayload, typename ResponseHeader>
inline auto send(ResponseMut<Service, ResponsePayload, ResponseHeader>&& response) -> iox::expected<void, SendError> {
    auto result = iox2_response_mut_send(response.m_handle);
    response.m_handle = nullptr;

    if (result == IOX2_OK) {
        return iox::ok();
    }
    return iox::err(iox::into<SendError>(result));
}

template <ServiceType Service, typename ResponsePayload, typename ResponseHeader>
inline void ResponseMut<Service, ResponsePayload, ResponseHeader>::drop() {
    if (m_handle != nullptr) {
        iox2_response_mut_drop(m_handle);
        m_handle = nullptr;
    }
}

} // namespace iox2

#endif
