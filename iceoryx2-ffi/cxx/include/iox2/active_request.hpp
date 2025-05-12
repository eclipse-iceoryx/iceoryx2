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

#ifndef IOX2_ACTIVE_REQUEST_HPP
#define IOX2_ACTIVE_REQUEST_HPP

#include "iox2/payload_info.hpp"
#include "iox2/response_mut_uninit.hpp"
#include "iox2/service_type.hpp"

namespace iox2 {
/// Represents a one-to-one connection to a [`Client`](crate::port::client::Client)
/// holding the corresponding
/// [`PendingResponse`](crate::pending_response::PendingResponse) that is coupled
/// with the [`RequestMut`](crate::request_mut::RequestMut) the
/// [`Client`](crate::port::client::Client) sent to the
/// [`Server`](crate::port::server::Server).
/// The [`Server`](crate::port::server::Server) will use it to send arbitrary many
/// [`Response`](crate::response::Response)s.
template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
class ActiveRequest {
    using ValueType = typename PayloadInfo<RequestPayload>::ValueType;

  public:
    ActiveRequest(ActiveRequest&& rhs) noexcept;
    auto operator=(ActiveRequest&& rhs) noexcept -> ActiveRequest&;
    ~ActiveRequest() noexcept;

    ActiveRequest(const ActiveRequest&) = delete;
    auto operator=(const ActiveRequest&) noexcept -> ActiveRequest& = delete;

    auto operator*() const -> const RequestPayload&;
    auto operator->() const -> const RequestPayload*;

    /// Loans uninitialized memory for a [`ResponseMutUninit`] where the user can write its payload to.
    template <typename T = ResponsePayload, typename = std::enable_if_t<!iox::IsSlice<T>::VALUE, void>>
    auto loan_uninit() -> iox::expected<ResponseMutUninit<Service, ResponsePayload, ResponseHeader>, LoanError>;

    /// Loans uninitialized memory for a [`ResponseMutUninit`] where the user can write its payload to.
    template <typename T = ResponsePayload, typename = std::enable_if_t<iox::IsSlice<T>::VALUE, void>>
    auto loan_slice_uninit(uint64_t number_of_elements)
        -> iox::expected<ResponseMutUninit<Service, ResponsePayload, ResponseHeader>, LoanError>;

    /// Sends a copy of the provided data to the
    /// [`PendingResponse`](crate::pending_response::PendingResponse) of the corresponding
    /// [`Client`](crate::port::client::Client).
    /// This is not a zero-copy API. Use [`ActiveRequest::loan_uninit()`] instead.
    template <typename T = RequestPayload, typename = std::enable_if_t<!iox::IsSlice<T>::VALUE, void>>
    auto send_copy(const ResponsePayload& payload) const -> iox::expected<void, SendError>;

    /// Sends a copy of the provided data to the
    /// [`PendingResponse`](crate::pending_response::PendingResponse) of the corresponding
    /// [`Client`](crate::port::client::Client).
    /// This is not a zero-copy API. Use [`ActiveRequest::loan_slice_uninit()`] instead.
    template <typename T = RequestPayload, typename = std::enable_if_t<iox::IsSlice<T>::VALUE, void>>
    auto send_slice_copy(const iox::ImmutableSlice<ValueType>& payload) const -> iox::expected<void, SendError>;

    /// Returns a reference to the payload of the received RequestPayload
    /// [`RequestMut`](crate::request_mut::RequestMut)
    template <typename T = RequestPayload, typename = std::enable_if_t<!iox::IsSlice<T>::VALUE, void>>
    auto payload() const -> const T&;

    /// Returns a reference to the payload of the received RequestPayload
    /// [`RequestMut`](crate::request_mut::RequestMut)
    template <typename T = RequestPayload, typename = std::enable_if_t<iox::IsSlice<T>::VALUE, void>>
    auto payload() const -> iox::ImmutableSlice<ValueType>;

    /// Returns a reference to the user_header of the received
    /// [`RequestMut`](crate::request_mut::RequestMut)
    template <typename T = RequestHeader, typename = std::enable_if_t<!std::is_same_v<void, RequestHeader>, T>>
    auto user_header() const -> const T&;

    /// Returns a reference to the
    /// [`crate::service::header::request_response::RequestHeader`] of the received
    /// [`RequestMut`](crate::request_mut::RequestMut)
    auto header() const -> RequestHeaderRequestResponse;

    /// Returns the [`UniqueClientId`] of the [`Client`](crate::port::client::Client)
    auto origin() const -> UniqueClientId;

    /// Returns [`true`] until the [`PendingResponse`](crate::pending_response::PendingResponse)
    /// goes out of scope on the [`Client`](crate::port::client::Client)s side indicating that the
    /// [`Client`](crate::port::client::Client) no longer receives the [`ResponseMut`].
    auto is_connected() const -> bool;

    /// Loans default initialized memory for a [`ResponseMut`] where the user can write its
    /// payload to.
    template <typename T = ResponsePayload, typename = std::enable_if_t<!iox::IsSlice<T>::VALUE, void>>
    auto loan() -> iox::expected<ResponseMut<Service, ResponsePayload, ResponseHeader>, LoanError>;

    /// Loans default initialized memory for a [`ResponseMut`] where the user can write its
    /// payload to.
    template <typename T = ResponsePayload, typename = std::enable_if_t<iox::IsSlice<T>::VALUE, void>>
    auto loan_slice(uint64_t number_of_elements)
        -> iox::expected<ResponseMut<Service, ResponsePayload, ResponseHeader>, LoanError>;

  private:
    template <ServiceType, typename, typename, typename, typename>
    friend class Server;

    explicit ActiveRequest(iox2_active_request_h handle) noexcept;

    void drop();

    iox2_active_request_h m_handle = nullptr;
};

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline ActiveRequest<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::ActiveRequest(
    ActiveRequest&& rhs) noexcept {
    *this = std::move(rhs);
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline auto ActiveRequest<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::operator=(
    ActiveRequest&& rhs) noexcept -> ActiveRequest& {
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
inline ActiveRequest<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::
    ~ActiveRequest() noexcept {
    drop();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline auto ActiveRequest<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::operator*() const
    -> const RequestPayload& {
    return payload();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline auto ActiveRequest<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::operator->() const
    -> const RequestPayload* {
    return &payload();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
template <typename T, typename>
inline auto ActiveRequest<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::loan_uninit()
    -> iox::expected<ResponseMutUninit<Service, ResponsePayload, ResponseHeader>, LoanError> {
    ResponseMutUninit<Service, ResponsePayload, ResponseHeader> response;
    auto result = iox2_active_request_loan_slice_uninit(
        &m_handle, &response.m_response.m_response, &response.m_response.m_handle, 1);
    if (result == IOX2_OK) {
        return iox::ok(std::move(response));
    }
    return iox::err(iox::into<LoanError>(result));
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
template <typename T, typename>
inline auto ActiveRequest<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::loan_slice_uninit(
    uint64_t number_of_elements)
    -> iox::expected<ResponseMutUninit<Service, ResponsePayload, ResponseHeader>, LoanError> {
    ResponseMutUninit<Service, ResponsePayload, ResponseHeader> response;
    auto result = iox2_active_request_loan_slice_uninit(
        &m_handle, &response.m_response.m_response, &response.m_response.m_handle, number_of_elements);
    if (result == IOX2_OK) {
        return iox::ok(std::move(response));
    }
    return iox::err(iox::into<LoanError>(result));
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
template <typename T, typename>
inline auto ActiveRequest<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::send_copy(
    const ResponsePayload& payload) const -> iox::expected<void, SendError> {
    static_assert(std::is_trivially_copyable_v<ResponsePayload>);

    auto result =
        iox2_active_request_send_copy(&m_handle, static_cast<const void*>(&payload), sizeof(ResponsePayload), 1);
    if (result == IOX2_OK) {
        return iox::ok();
    }
    return iox::err(iox::into<SendError>(result));
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
template <typename T, typename>
inline auto ActiveRequest<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::send_slice_copy(
    const iox::ImmutableSlice<ValueType>& payload) const -> iox::expected<void, SendError> {
    auto result = iox2_active_request_send_copy(
        &m_handle, payload.data(), sizeof(typename ResponsePayload::ValueType), payload.number_of_elements());
    if (result == IOX2_OK) {
        return iox::ok();
    }
    return iox::err(iox::into<SendError>(result));
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
template <typename T, typename>
inline auto ActiveRequest<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::payload() const
    -> const T& {
    const void* ptr = nullptr;
    size_t number_of_elements = 0;

    iox2_active_request_payload(&m_handle, &ptr, &number_of_elements);
    return *static_cast<const T*>(ptr);
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
template <typename T, typename>
inline auto ActiveRequest<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::payload() const
    -> iox::ImmutableSlice<ValueType> {
    const void* ptr = nullptr;
    size_t number_of_elements = 0;

    iox2_active_request_payload(&m_handle, &ptr, &number_of_elements);

    return iox::ImmutableSlice<ValueType>(static_cast<const ValueType*>(ptr), number_of_elements);
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
template <typename T, typename>
inline auto ActiveRequest<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::user_header() const
    -> const T& {
    const void* ptr = nullptr;
    iox2_active_request_user_header(&m_handle, &ptr);
    return *static_cast<const T*>(ptr);
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline auto ActiveRequest<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::header() const
    -> RequestHeaderRequestResponse {
    iox2_request_header_h header_handle = nullptr;
    iox2_active_request_header(&m_handle, nullptr, &header_handle);
    return RequestHeaderRequestResponse { header_handle };
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline auto ActiveRequest<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::origin() const
    -> UniqueClientId {
    return header().client_port_id();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline auto ActiveRequest<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::is_connected() const
    -> bool {
    return iox2_active_request_is_connected(&m_handle);
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
template <typename T, typename>
inline auto ActiveRequest<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::loan()
    -> iox::expected<ResponseMut<Service, ResponsePayload, ResponseHeader>, LoanError> {
    auto response = loan_uninit();
    if (response.has_error()) {
        return iox::err(response.error());
    }

    new (&response->payload_mut()) ResponsePayload();
    return iox::ok(assume_init(std::move(*response)));
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
template <typename T, typename>
inline auto ActiveRequest<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::loan_slice(
    uint64_t number_of_elements) -> iox::expected<ResponseMut<Service, ResponsePayload, ResponseHeader>, LoanError> {
    auto response_uninit = loan_slice_uninit(number_of_elements);
    if (response_uninit.has_error()) {
        return iox::err(response_uninit.error());
    }

    auto response_init = std::move(response_uninit.value());
    for (auto& item : response_init.payload_mut()) {
        new (&item) ValueType();
    }

    return iox::ok(assume_init(std::move(response_init)));
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline ActiveRequest<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::ActiveRequest(
    iox2_active_request_h handle) noexcept
    : m_handle(handle) {
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline void ActiveRequest<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::drop() {
    if (m_handle != nullptr) {
        iox2_active_request_drop(m_handle);
        m_handle = nullptr;
    }
}
} // namespace iox2

#endif
