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

#ifndef RESPONSE_MUT_HPP
#define RESPONSE_MUT_HPP

#include "iox/assertions_addendum.hpp"
#include "iox/expected.hpp"
#include "iox2/header_request_response.hpp"
#include "iox2/port_error.hpp"
#include "iox2/service_type.hpp"

namespace iox2 {

template <ServiceType Service, typename ResponsePayload, typename ResponseHeader>
class ResponseMut {
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

    auto header() const -> ResponseHeaderRequestResponse&;

    template <typename T = ResponseHeader, typename = std::enable_if_t<!std::is_same_v<void, ResponseHeader>, T>>
    auto user_header() const -> const T&;

    template <typename T = ResponseHeader, typename = std::enable_if_t<!std::is_same_v<void, ResponseHeader>, T>>
    auto user_header_mut() -> T&;

    auto payload() const -> const ResponsePayload&;
    auto payload_mut() -> ResponsePayload&;

    auto send() -> iox::expected<void, SendError>;

  private:
    template <ServiceType, typename, typename>
    friend class ResponseMutUninit;

    explicit ResponseMut();
    void drop();

    // iox2_response_mut_t m_response;
    // iox2_response_mut_h m_handle = nullptr;
};

template <ServiceType Service, typename ResponsePayload, typename ResponseHeader>
inline ResponseMut<Service, ResponsePayload, ResponseHeader>::ResponseMut(ResponseMut&& rhs) noexcept {
    *this = std::move(rhs);
}

template <ServiceType Service, typename ResponsePayload, typename ResponseHeader>
inline auto ResponseMut<Service, ResponsePayload, ResponseHeader>::operator=(ResponseMut&& rhs) noexcept
    -> ResponseMut& {
    IOX_TODO();
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
    IOX_TODO();
}

template <ServiceType Service, typename ResponsePayload, typename ResponseHeader>
inline auto ResponseMut<Service, ResponsePayload, ResponseHeader>::payload() const -> const ResponsePayload& {
    IOX_TODO();
}

template <ServiceType Service, typename ResponsePayload, typename ResponseHeader>
inline auto ResponseMut<Service, ResponsePayload, ResponseHeader>::payload_mut() -> ResponsePayload& {
    IOX_TODO();
}

template <ServiceType Service, typename ResponsePayload, typename ResponseHeader>
inline auto ResponseMut<Service, ResponsePayload, ResponseHeader>::send() -> iox::expected<void, SendError> {
    IOX_TODO();
}

template <ServiceType Service, typename ResponsePayload, typename ResponseHeader>
inline ResponseMut<Service, ResponsePayload, ResponseHeader>::ResponseMut() {
    IOX_TODO();
}

template <ServiceType Service, typename ResponsePayload, typename ResponseHeader>
inline void ResponseMut<Service, ResponsePayload, ResponseHeader>::drop() {
    IOX_TODO();
}

} // namespace iox2

#endif

