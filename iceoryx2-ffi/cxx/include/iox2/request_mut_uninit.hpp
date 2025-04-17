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

#ifndef REQUEST_MUT_UNINIT_HPP
#define REQUEST_MUT_UNINIT_HPP

#include "iox/assertions_addendum.hpp"
#include "iox2/header_request_response.hpp"
#include "iox2/request_mut.hpp"
#include "iox2/service_type.hpp"

namespace iox2 {

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
class RequestMutUninit {
  public:
    RequestMutUninit(RequestMutUninit&& rhs) noexcept = default;
    auto operator=(RequestMutUninit&& rhs) noexcept -> RequestMutUninit& = default;
    ~RequestMutUninit() noexcept = default;

    RequestMutUninit(const RequestMutUninit&) = delete;
    auto operator=(const RequestMutUninit&) -> RequestMutUninit& = delete;

    auto header() const -> RequestHeaderRequestResponse&;

    template <typename T = RequestHeader, typename = std::enable_if_t<!std::is_same_v<void, RequestHeader>, T>>
    auto user_header() const -> const T&;

    template <typename T = RequestHeader, typename = std::enable_if_t<!std::is_same_v<void, RequestHeader>, T>>
    auto user_header_mut() -> T&;

    auto payload() const -> const RequestPayload&;
    auto payload_mut() -> RequestPayload&;

    void write_payload(RequestPayload& value);

  private:
    explicit RequestMutUninit();

    auto assume_init() -> RequestMut<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>;

    RequestMut<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader> m_request;
};

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline auto RequestMutUninit<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::header() const
    -> RequestHeaderRequestResponse& {
    IOX_TODO();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
template <typename T, typename>
inline auto
RequestMutUninit<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::user_header() const
    -> const T& {
    IOX_TODO();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
template <typename T, typename>
inline auto RequestMutUninit<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::user_header_mut()
    -> T& {
    IOX_TODO();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline auto RequestMutUninit<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::payload() const
    -> const RequestPayload& {
    IOX_TODO();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline auto RequestMutUninit<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::payload_mut()
    -> RequestPayload& {
    IOX_TODO();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline void RequestMutUninit<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::write_payload(
    RequestPayload& value) {
    IOX_TODO();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline RequestMutUninit<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::RequestMutUninit() {
    IOX_TODO();
}

template <ServiceType Service,
          typename RequestPayload,
          typename RequestHeader,
          typename ResponsePayload,
          typename ResponseHeader>
inline auto RequestMutUninit<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader>::assume_init()
    -> RequestMut<Service, RequestPayload, RequestHeader, ResponsePayload, ResponseHeader> {
    IOX_TODO();
}

} // namespace iox2

#endif

