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

#ifndef RESPONSE_MUT_UNINIT_HPP
#define RESPONSE_MUT_UNINIT_HPP

#include "iox2/response_mut.hpp"
#include "iox2/service_type.hpp"

namespace iox2 {

template <ServiceType Service, typename ResponsePayload, typename ResponseHeader>
class ResponseMutUninit {
  public:
    ResponseMutUninit(ResponseMutUninit&& rhs) noexcept = default;
    auto operator=(ResponseMutUninit&& rhs) noexcept -> ResponseMutUninit& = default;
    ~ResponseMutUninit() noexcept = default;

    ResponseMutUninit(const ResponseMutUninit&) = delete;
    auto operator=(const ResponseMutUninit&) -> ResponseMutUninit& = delete;

    auto header() const -> ResponseHeaderRequestResponse&;

    template <typename T = ResponseHeader, typename = std::enable_if_t<!std::is_same_v<void, ResponseHeader>, T>>
    auto user_header() const -> const T&;

    template <typename T = ResponseHeader, typename = std::enable_if_t<!std::is_same_v<void, ResponseHeader>, T>>
    auto user_header_mut() -> T&;

    auto payload() const -> const ResponsePayload&;
    auto payload_mut() -> ResponsePayload&;

    void write_payload(ResponsePayload& value);

  private:
    explicit ResponseMutUninit();

    auto assume_init() -> ResponseMut<Service, ResponsePayload, ResponseHeader>;

    ResponseMut<Service, ResponsePayload, ResponseHeader> m_response;
};

template <ServiceType Service, typename ResponsePayload, typename ResponseHeader>
inline auto ResponseMutUninit<Service, ResponsePayload, ResponseHeader>::header() const
    -> ResponseHeaderRequestResponse& {
    IOX_TODO();
}

template <ServiceType Service, typename ResponsePayload, typename ResponseHeader>
template <typename T, typename>
inline auto ResponseMutUninit<Service, ResponsePayload, ResponseHeader>::user_header() const -> const T& {
    IOX_TODO();
}

template <ServiceType Service, typename ResponsePayload, typename ResponseHeader>
template <typename T, typename>
inline auto ResponseMutUninit<Service, ResponsePayload, ResponseHeader>::user_header_mut() -> T& {
    IOX_TODO();
}

template <ServiceType Service, typename ResponsePayload, typename ResponseHeader>
inline auto ResponseMutUninit<Service, ResponsePayload, ResponseHeader>::payload() const -> const ResponsePayload& {
    IOX_TODO();
}

template <ServiceType Service, typename ResponsePayload, typename ResponseHeader>
inline auto ResponseMutUninit<Service, ResponsePayload, ResponseHeader>::payload_mut() -> ResponsePayload& {
    IOX_TODO();
}

template <ServiceType Service, typename ResponsePayload, typename ResponseHeader>
inline void ResponseMutUninit<Service, ResponsePayload, ResponseHeader>::write_payload(ResponsePayload& value) {
    IOX_TODO();
}

template <ServiceType Service, typename ResponsePayload, typename ResponseHeader>
inline ResponseMutUninit<Service, ResponsePayload, ResponseHeader>::ResponseMutUninit() {
    IOX_TODO();
}

template <ServiceType Service, typename ResponsePayload, typename ResponseHeader>
inline auto ResponseMutUninit<Service, ResponsePayload, ResponseHeader>::assume_init()
    -> ResponseMut<Service, ResponsePayload, ResponseHeader> {
    IOX_TODO();
}

} // namespace iox2

#endif

