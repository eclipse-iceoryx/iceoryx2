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

#include "internal/helper.hpp"
#include "iox2/internal/helper.hpp"
#include "iox2/payload_info.hpp"
#include "iox2/response_mut_uninit.hpp"
#include "iox2/service_type.hpp"

namespace iox2 {
/// Represents a one-to-one connection to a [`Client`] holding the corresponding
/// [`PendingResponse`] that is coupled with the [`RequestMut`] the [`Client`]
/// sent to the [`Server`]. The [`Server`] will use it to send arbitrary many
/// [`Response`]s.
template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
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
    auto loan_uninit() -> iox::expected<ResponseMutUninit<Service, ResponsePayload, ResponseUserHeader>, LoanError>;

    /// Loans uninitialized memory for a [`ResponseMutUninit`] where the user can write its payload to.
    template <typename T = ResponsePayload, typename = std::enable_if_t<iox::IsSlice<T>::VALUE, void>>
    auto loan_slice_uninit(uint64_t number_of_elements)
        -> iox::expected<ResponseMutUninit<Service, ResponsePayload, ResponseUserHeader>, LoanError>;

    /// Sends a copy of the provided data to the [`PendingResponse`] of the corresponding
    /// [`Client`]. This is not a zero-copy API. Use [`ActiveRequest::loan_uninit()`] instead.
    template <typename T = RequestPayload, typename = std::enable_if_t<!iox::IsSlice<T>::VALUE, void>>
    auto send_copy(const ResponsePayload& payload) const -> iox::expected<void, SendError>;

    /// Sends a copy of the provided data to the [`PendingResponse`] of the corresponding
    /// [`Client`]. This is not a zero-copy API. Use [`ActiveRequest::loan_slice_uninit()`] instead.
    template <typename T = RequestPayload, typename = std::enable_if_t<iox::IsSlice<T>::VALUE, void>>
    auto send_slice_copy(const iox::ImmutableSlice<ValueType>& payload) const -> iox::expected<void, SendError>;

    /// Returns a reference to the payload of the received [`RequestMut`]
    template <typename T = RequestPayload, typename = std::enable_if_t<!iox::IsSlice<T>::VALUE, void>>
    auto payload() const -> const T&;

    /// Returns a reference to the payload of the received [`RequestMut`]
    template <typename T = RequestPayload, typename = std::enable_if_t<iox::IsSlice<T>::VALUE, void>>
    auto payload() const -> iox::ImmutableSlice<ValueType>;

    /// Returns a reference to the user_header of the received [`RequestMut`]
    template <typename T = RequestUserHeader, typename = std::enable_if_t<!std::is_same_v<void, RequestUserHeader>, T>>
    auto user_header() const -> const T&;

    /// Returns a reference to the [`RequestHeader`] of the received [`RequestMut`]
    auto header() const -> RequestHeader;

    /// Returns the [`UniqueClientId`] of the [`Client`]
    auto origin() const -> UniqueClientId;

    /// Returns [`true`] until the [`PendingResponse`] goes out of scope on the
    /// [`Client`] side indicating that the [`Client`] no longer receives the [`ResponseMut`].
    auto is_connected() const -> bool;

    /// Returns [`true`] if the [`Client`] wants to gracefully disconnect.
    /// This allows the [`Server`] to send its last response and then
    /// drop the [`ActiveRequest`] to signal the [`Client`] that no more
    /// [`ResponseMut`] will be sent.
    auto has_disconnect_hint() const -> bool;

    /// Loans default initialized memory for a [`ResponseMut`] where the user can write its
    /// payload to.
    template <typename T = ResponsePayload, typename = std::enable_if_t<!iox::IsSlice<T>::VALUE, void>>
    auto loan() -> iox::expected<ResponseMut<Service, ResponsePayload, ResponseUserHeader>, LoanError>;

    /// Loans default initialized memory for a [`ResponseMut`] where the user can write its
    /// payload to.
    template <typename T = ResponsePayload, typename = std::enable_if_t<iox::IsSlice<T>::VALUE, void>>
    auto loan_slice(uint64_t number_of_elements)
        -> iox::expected<ResponseMut<Service, ResponsePayload, ResponseUserHeader>, LoanError>;

  private:
    template <ServiceType, typename, typename, typename, typename>
    friend class Server;

    explicit ActiveRequest(iox2_active_request_h handle) noexcept;

    void drop();

    iox2_active_request_h m_handle = nullptr;
};

template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
inline ActiveRequest<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>::ActiveRequest(
    ActiveRequest&& rhs) noexcept {
    *this = std::move(rhs);
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
inline auto ActiveRequest<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>::operator=(
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
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
inline ActiveRequest<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>::
    ~ActiveRequest() noexcept {
    drop();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
inline auto
ActiveRequest<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>::operator*() const
    -> const RequestPayload& {
    return payload();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
inline auto
ActiveRequest<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>::operator->() const
    -> const RequestPayload* {
    return &payload();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
template <typename T, typename>
inline auto
ActiveRequest<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>::loan_uninit()
    -> iox::expected<ResponseMutUninit<Service, ResponsePayload, ResponseUserHeader>, LoanError> {
    constexpr uint64_t NUMBER_OF_ELEMENTS = 1;
    ResponseMutUninit<Service, ResponsePayload, ResponseUserHeader> response;
    auto result = iox2_active_request_loan_slice_uninit(
        &m_handle, &response.m_response.m_response, &response.m_response.m_handle, NUMBER_OF_ELEMENTS);
    internal::PlacementDefault<ResponseUserHeader>::placement_default(response);
    if (result == IOX2_OK) {
        return iox::ok(std::move(response));
    }
    return iox::err(iox::into<LoanError>(result));
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
template <typename T, typename>
inline auto
ActiveRequest<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>::loan_slice_uninit(
    uint64_t number_of_elements)
    -> iox::expected<ResponseMutUninit<Service, ResponsePayload, ResponseUserHeader>, LoanError> {
    ResponseMutUninit<Service, ResponsePayload, ResponseUserHeader> response;
    auto result = iox2_active_request_loan_slice_uninit(
        &m_handle, &response.m_response.m_response, &response.m_response.m_handle, number_of_elements);
    internal::PlacementDefault<ResponseUserHeader>::placement_default(response);
    if (result == IOX2_OK) {
        return iox::ok(std::move(response));
    }
    return iox::err(iox::into<LoanError>(result));
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
template <typename T, typename>
inline auto ActiveRequest<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>::send_copy(
    const ResponsePayload& payload) const -> iox::expected<void, SendError> {
    static_assert(std::is_trivially_copyable_v<ResponsePayload>);
    constexpr uint64_t NUMBER_OF_ELEMENTS = 1;

    auto result = iox2_active_request_send_copy(
        &m_handle, static_cast<const void*>(&payload), sizeof(ResponsePayload), NUMBER_OF_ELEMENTS);
    if (result == IOX2_OK) {
        return iox::ok();
    }
    return iox::err(iox::into<SendError>(result));
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
template <typename T, typename>
inline auto
ActiveRequest<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>::send_slice_copy(
    const iox::ImmutableSlice<ValueType>& payload) const -> iox::expected<void, SendError> {
    static_assert(std::is_trivially_copyable_v<ValueType>);

    auto result = iox2_active_request_send_copy(
        &m_handle, payload.data(), sizeof(typename ResponsePayload::ValueType), payload.number_of_elements());
    if (result == IOX2_OK) {
        return iox::ok();
    }
    return iox::err(iox::into<SendError>(result));
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
template <typename T, typename>
inline auto
ActiveRequest<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>::payload() const
    -> const T& {
    const void* ptr = nullptr;
    iox2_active_request_payload(&m_handle, &ptr, nullptr);
    return *static_cast<const T*>(ptr);
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
template <typename T, typename>
inline auto
ActiveRequest<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>::payload() const
    -> iox::ImmutableSlice<ValueType> {
    const void* ptr = nullptr;
    size_t number_of_elements = 0;

    iox2_active_request_payload(&m_handle, &ptr, &number_of_elements);

    return iox::ImmutableSlice<ValueType>(static_cast<const ValueType*>(ptr), number_of_elements);
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
template <typename T, typename>
inline auto
ActiveRequest<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>::user_header() const
    -> const T& {
    const void* ptr = nullptr;
    iox2_active_request_user_header(&m_handle, &ptr);
    return *static_cast<const T*>(ptr);
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
inline auto
ActiveRequest<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>::header() const
    -> RequestHeader {
    iox2_request_header_h header_handle = nullptr;
    iox2_active_request_header(&m_handle, nullptr, &header_handle);
    return RequestHeader { header_handle };
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
inline auto
ActiveRequest<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>::origin() const
    -> UniqueClientId {
    return header().client_port_id();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
inline auto
ActiveRequest<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>::is_connected() const
    -> bool {
    return iox2_active_request_is_connected(&m_handle);
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
inline auto
ActiveRequest<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>::has_disconnect_hint()
    const -> bool {
    return iox2_active_request_has_disconnect_hint(&m_handle);
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
template <typename T, typename>
inline auto ActiveRequest<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>::loan()
    -> iox::expected<ResponseMut<Service, ResponsePayload, ResponseUserHeader>, LoanError> {
    auto response = loan_uninit();
    if (response.has_error()) {
        return iox::err(response.error());
    }

    new (&response->payload_mut()) ResponsePayload();
    return iox::ok(assume_init(std::move(*response)));
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
template <typename T, typename>
inline auto ActiveRequest<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>::loan_slice(
    uint64_t number_of_elements)
    -> iox::expected<ResponseMut<Service, ResponsePayload, ResponseUserHeader>, LoanError> {
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
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
inline ActiveRequest<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>::ActiveRequest(
    iox2_active_request_h handle) noexcept
    : m_handle(handle) {
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestUserHeader,
          typename ResponsePayload,
          typename ResponseUserHeader>
inline void ActiveRequest<Service, RequestPayload, RequestUserHeader, ResponsePayload, ResponseUserHeader>::drop() {
    if (m_handle != nullptr) {
        iox2_active_request_drop(m_handle);
        m_handle = nullptr;
    }
}
} // namespace iox2

#endif
