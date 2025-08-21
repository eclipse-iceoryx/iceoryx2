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

#ifndef IOX2_HEADER_REQUEST_RESPONSE_HPP
#define IOX2_HEADER_REQUEST_RESPONSE_HPP

#include "iox2/unique_port_id.hpp"

namespace iox2 {
/// Request header used by [`MessagingPattern::RequestResponse`]
class RequestHeader {
  public:
    RequestHeader(const RequestHeader&) = delete;
    RequestHeader(RequestHeader&& rhs) noexcept;
    auto operator=(const RequestHeader&) -> RequestHeader& = delete;
    auto operator=(RequestHeader&& rhs) noexcept -> RequestHeader&;
    ~RequestHeader();

    /// Returns the [`UniqueClientId`] of the source [`Client`].
    auto client_port_id() -> UniqueClientId;

  private:
    template <ServiceType, typename, typename, typename, typename>
    friend class ActiveRequest;
    template <ServiceType, typename, typename, typename, typename>
    friend class PendingResponse;
    template <ServiceType, typename, typename, typename, typename>
    friend class RequestMut;

    explicit RequestHeader(iox2_request_header_h handle);
    void drop();

    iox2_request_header_h m_handle = nullptr;
};

/// Response header used by [`MessagingPattern::RequestResponse`]
class ResponseHeader {
  public:
    ResponseHeader(const ResponseHeader&) = delete;
    ResponseHeader(ResponseHeader&& rhs) noexcept;
    auto operator=(const ResponseHeader&) -> ResponseHeader& = delete;
    auto operator=(ResponseHeader&& rhs) noexcept -> ResponseHeader&;
    ~ResponseHeader();

    /// Returns the [`UniqueServerId`] of the source [`Server`].
    auto server_port_id() -> UniqueServerId;

  private:
    template <ServiceType, typename, typename>
    friend class Response;
    template <ServiceType, typename, typename>
    friend class ResponseMut;

    explicit ResponseHeader(iox2_response_header_h handle);
    void drop();

    iox2_response_header_h m_handle = nullptr;
};
} // namespace iox2
#endif
