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

#ifndef IOX2_CLIENT_HPP
#define IOX2_CLIENT_HPP

#include "iox/expected.hpp"
#include "iox/slice.hpp"
#include "iox2/payload_info.hpp"
#include "iox2/request_mut_uninit.hpp"
#include "iox2/service_type.hpp"
#include "iox2/unique_port_id.hpp"

namespace iox2 {
/// Sends [`RequestMut`]s to a [`Server`](crate::port::server::Server) in a
/// request-response based communication.
template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
class Client {
    using ValueType = typename PayloadInfo<RequestPayload>::ValueType;

  public:
    Client(Client&& rhs) noexcept;
    auto operator=(Client&& rhs) noexcept -> Client&;
    ~Client() noexcept;

    Client(const Client&) noexcept = delete;
    auto operator=(const Client&) noexcept -> Client& = delete;

    /// Returns the [`UniqueClientId`] of the [`Client`]
    auto id() const -> UniqueClientId;

    /// Returns the strategy the [`Client`] follows when a [`RequestMut`] cannot be delivered
    /// if the [`Server`](crate::port::server::Server)s buffer is full.
    auto unable_to_deliver_strategy() const -> UnableToDeliverStrategy;

    /// Returns the maximum number of elements that can be loaned in a slice.
    template <typename T = RequestPayload, typename = std::enable_if_t<iox::IsSlice<T>::VALUE, void>>
    auto initial_max_slice_len() const -> uint64_t;

    /// Copies the input value into a [`RequestMut`] and sends it. On success it
    /// returns a [`PendingResponse`] that can be used to receive a stream of
    /// [`Response`](crate::response::Response)s from the
    /// [`Server`](crate::port::server::Server).
    template <typename T = RequestPayload, typename = std::enable_if_t<!iox::IsSlice<T>::VALUE, void>>
    auto send_copy(const RequestPayload& payload) const
        -> iox::expected<PendingResponse<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>,
                         RequestSendError>;

    /// Copies the input value into a [`RequestMut`] and sends it. On success it
    /// returns a [`PendingResponse`] that can be used to receive a stream of
    /// [`Response`](crate::response::Response)s from the
    /// [`Server`](crate::port::server::Server).
    template <typename T = RequestPayload, typename = std::enable_if_t<iox::IsSlice<T>::VALUE, void>>
    auto send_slice_copy(iox::ImmutableSlice<ValueType>& payload) const
        -> iox::expected<PendingResponse<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>,
                         RequestSendError>;

    /// Acquires an [`RequestMutUninit`] to store payload. This API shall be used
    /// by default to avoid unnecessary copies.
    template <typename T = RequestPayload, typename = std::enable_if_t<!iox::IsSlice<T>::VALUE, void>>
    auto loan_uninit()
        -> iox::expected<RequestMutUninit<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>,
                         LoanError>;

    /// Acquires an [`RequestMutUninit`] to store payload. This API shall be used
    /// by default to avoid unnecessary copies.
    template <typename T = RequestPayload, typename = std::enable_if_t<iox::IsSlice<T>::VALUE, void>>
    auto loan_slice_uninit(uint64_t number_of_elements)
        -> iox::expected<RequestMutUninit<Service, T, RequestHeader, ResponsePayload, ResponseHeader>, LoanError>;

    /// Acquires the payload for the request and initializes the underlying memory
    /// with default. This can be very expensive when the payload is large, therefore
    /// prefer [`Client::loan_uninit()`] when possible.
    template <typename T = RequestPayload, typename = std::enable_if_t<!iox::IsSlice<T>::VALUE, void>>
    auto loan() -> iox::expected<RequestMut<Service, T, RequestHeader, ResponsePayload, ResponseHeader>, LoanError>;

    /// Acquires the payload for the request and initializes the underlying memory
    /// with default. This can be very expensive when the payload is large, therefore
    /// prefer [`Client::loan_uninit()`] when possible.
    template <typename T = RequestPayload, typename = std::enable_if_t<iox::IsSlice<T>::VALUE, void>>
    auto loan_slice(uint64_t number_of_elements)
        -> iox::expected<RequestMut<Service, T, RequestHeader, ResponsePayload, ResponseHeader>, LoanError>;

  private:
    template <ServiceType, typename, typename, typename, typename>
    friend class PortFactoryClient;

    explicit Client(iox2_client_h handle) noexcept;

    void drop();

    iox2_client_h m_handle = nullptr;
};

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline Client<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::Client(Client&& rhs) noexcept {
    *this = std::move(rhs);
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline auto
Client<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::operator=(Client&& rhs) noexcept
    -> Client& {
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
inline Client<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::~Client() noexcept {
    drop();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline auto Client<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::id() const
    -> UniqueClientId {
    iox2_unique_client_id_h id_handle = nullptr;
    iox2_client_id(&m_handle, nullptr, &id_handle);
    return UniqueClientId { id_handle };
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline auto
Client<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::unable_to_deliver_strategy() const
    -> UnableToDeliverStrategy {
    return iox::into<UnableToDeliverStrategy>(static_cast<int>(iox2_client_unable_to_deliver_strategy(&m_handle)));
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
template <typename T, typename>
inline auto
Client<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::initial_max_slice_len() const
    -> uint64_t {
    return iox2_client_initial_max_slice_len(&m_handle);
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
template <typename T, typename>
inline auto Client<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::loan_uninit()
    -> iox::expected<RequestMutUninit<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>,
                     LoanError> {
    RequestMutUninit<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader> request;
    auto result =
        iox2_client_loan_slice_uninit(&m_handle, &request.m_request.m_request, &request.m_request.m_handle, 1);
    if (result == IOX2_OK) {
        return iox::ok(std::move(request));
    }
    return iox::err(iox::into<LoanError>(result));
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
template <typename T, typename>
inline auto Client<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::loan_slice_uninit(
    uint64_t number_of_elements)
    -> iox::expected<RequestMutUninit<Service, T, RequestHeader, ResponsePayload, ResponseHeader>, LoanError> {
    RequestMutUninit<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader> request;
    auto result = iox2_client_loan_slice_uninit(
        &m_handle, &request.m_request.m_request, &request.m_request.m_handle, number_of_elements);
    if (result == IOX2_OK) {
        return iox::ok(std::move(request));
    }
    return iox::err(iox::into<LoanError>(result));
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
template <typename T, typename>
inline auto Client<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::send_copy(
    const RequestPayload& payload) const
    -> iox::expected<PendingResponse<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>,
                     RequestSendError> {
    static_assert(std::is_trivially_copyable_v<RequestPayload>);

    iox2_pending_response_h pending_response_handle {};
    auto result = iox2_client_send_copy(
        &m_handle, static_cast<const void*>(&payload), sizeof(RequestPayload), 1, nullptr, &pending_response_handle);

    if (result == IOX2_OK) {
        return iox::ok(PendingResponse<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>(
            pending_response_handle));
    }

    return iox::err(iox::into<RequestSendError>(result));
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
template <typename T, typename>
inline auto Client<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::send_slice_copy(
    iox::ImmutableSlice<ValueType>& payload) const
    -> iox::expected<PendingResponse<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>,
                     RequestSendError> {
    iox2_pending_response_h pending_response_handle {};
    auto result = iox2_client_send_copy(&m_handle,
                                        payload.data(),
                                        sizeof(typename RequestPayload::ValueType),
                                        payload.number_of_elements(),
                                        nullptr,
                                        &pending_response_handle);

    if (result == IOX2_OK) {
        return iox::ok(PendingResponse<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>(
            pending_response_handle));
    }

    return iox::err(iox::into<RequestSendError>(result));
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
template <typename T, typename>
inline auto Client<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::loan()
    -> iox::expected<RequestMut<Service, T, RequestHeader, ResponsePayload, ResponseHeader>, LoanError> {
    auto request = loan_uninit();
    if (request.has_error()) {
        return iox::err(request.error());
    }

    new (&request->payload_mut()) RequestPayload();
    return iox::ok(assume_init(std::move(*request)));
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
template <typename T, typename>
inline auto
Client<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::loan_slice(uint64_t number_of_elements)
    -> iox::expected<RequestMut<Service, T, RequestHeader, ResponsePayload, ResponseHeader>, LoanError> {
    auto request_uninit = loan_slice_uninit(number_of_elements);
    if (request_uninit.has_error()) {
        return iox::err(request_uninit.error());
    }

    auto request_init = std::move(request_uninit.value());
    for (auto& item : request_init.payload_mut()) {
        new (&item) ValueType();
    }

    return iox::ok(assume_init(std::move(request_init)));
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline Client<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::Client(
    iox2_client_h handle) noexcept
    : m_handle { handle } {
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline void Client<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::drop() {
    if (m_handle != nullptr) {
        iox2_client_drop(m_handle);
        m_handle = nullptr;
    }
}
} // namespace iox2

#endif
