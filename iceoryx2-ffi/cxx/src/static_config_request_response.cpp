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

#include "iox2/static_config_request_response.hpp"
#include "iox/assertions_addendum.hpp"

namespace iox2 {
auto StaticConfigRequestResponse::request_message_type_details() const -> MessageTypeDetails {
    IOX_TODO();
}

auto StaticConfigRequestResponse::response_message_type_details() const -> MessageTypeDetails {
    IOX_TODO();
}

auto StaticConfigRequestResponse::has_safe_overflow_for_requests() const -> bool {
    IOX_TODO();
}

auto StaticConfigRequestResponse::has_safe_overflow_for_responses() const -> bool {
    IOX_TODO();
}

auto StaticConfigRequestResponse::max_borrowed_responses_per_pending_responses() const -> uint64_t {
    IOX_TODO();
}

auto StaticConfigRequestResponse::max_active_requests_per_client() const -> uint64_t {
    IOX_TODO();
}

auto StaticConfigRequestResponse::max_response_buffer_size() const -> uint64_t {
    IOX_TODO();
}

auto StaticConfigRequestResponse::max_servers() const -> uint64_t {
    IOX_TODO();
}

auto StaticConfigRequestResponse::max_clients() const -> uint64_t {
    IOX_TODO();
}

auto StaticConfigRequestResponse::max_nodes() const -> uint64_t {
    IOX_TODO();
}

StaticConfigRequestResponse::StaticConfigRequestResponse(/*iox2_static_config_request_response_t value*/) {
    IOX_TODO();
}
} // namespace iox2
