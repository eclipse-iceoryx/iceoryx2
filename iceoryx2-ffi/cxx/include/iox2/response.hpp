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

#ifndef RESPONSE_HPP
#define RESPONSE_HPP

#include "header_request_response.hpp"
#include "iox/assertions_addendum.hpp"
#include "iox2/service_type.hpp"

namespace iox2 {
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

    auto header() const -> ResponseHeaderRequestResponse&;

    template <typename T = ResponseHeader, typename = std::enable_if_t<!std::is_same_v<void, ResponseHeader>, T>>
    auto user_header() const -> const T&;

    auto payload() const -> const ResponsePayload&;

    auto origin() const -> UniqueServerId;

  private:
    explicit Response() noexcept;

    void drop();
};

template <ServiceType Service, typename ResponsePayload, typename ResponseHeader>
inline Response<Service, ResponsePayload, ResponseHeader>::Response(Response&& rhs) noexcept {
    *this = std::move(rhs);
}

template <ServiceType Service, typename ResponsePayload, typename ResponseHeader>
inline auto Response<Service, ResponsePayload, ResponseHeader>::operator=(Response&& rhs) noexcept -> Response& {
    IOX_TODO();
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
    IOX_TODO();
}

template <ServiceType Service, typename ResponsePayload, typename ResponseHeader>
inline auto Response<Service, ResponsePayload, ResponseHeader>::origin() const -> UniqueServerId {
    IOX_TODO();
}

template <ServiceType Service, typename ResponsePayload, typename ResponseHeader>
inline Response<Service, ResponsePayload, ResponseHeader>::Response() noexcept {
    IOX_TODO();
}

template <ServiceType Service, typename ResponsePayload, typename ResponseHeader>
inline void Response<Service, ResponsePayload, ResponseHeader>::drop() {
    IOX_TODO();
}
} // namespace iox2

#endif
