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

#include "iox2/header_request_response.hpp"
#include "iox/assertions_addendum.hpp"

namespace iox2 {
RequestHeaderRequestResponse::RequestHeaderRequestResponse(RequestHeaderRequestResponse&& rhs) noexcept {
    *this = std::move(rhs);
}

auto RequestHeaderRequestResponse::operator=(RequestHeaderRequestResponse&& rhs) noexcept
    -> RequestHeaderRequestResponse& {
    IOX_TODO();
}

RequestHeaderRequestResponse::~RequestHeaderRequestResponse() {
    drop();
}

auto RequestHeaderRequestResponse::client_port_id() -> UniqueClientId {
    IOX_TODO();
}

RequestHeaderRequestResponse::RequestHeaderRequestResponse(/*iox2_request_header_h handle*/) {
    IOX_TODO();
}

void RequestHeaderRequestResponse::drop() {
    IOX_TODO();
}

ResponseHeaderRequestResponse::ResponseHeaderRequestResponse(ResponseHeaderRequestResponse&& rhs) noexcept {
    *this = std::move(rhs);
}

auto ResponseHeaderRequestResponse::operator=(ResponseHeaderRequestResponse&& rhs) noexcept
    -> ResponseHeaderRequestResponse& {
    IOX_TODO();
}

ResponseHeaderRequestResponse::~ResponseHeaderRequestResponse() {
    drop();
}

auto ResponseHeaderRequestResponse::server_port_id() -> UniqueServerId {
    IOX_TODO();
}

ResponseHeaderRequestResponse::ResponseHeaderRequestResponse(/*iox2_response_header_h handle*/) {
    IOX_TODO();
}

void ResponseHeaderRequestResponse::drop() {
    IOX_TODO();
}

} // namespace iox2

