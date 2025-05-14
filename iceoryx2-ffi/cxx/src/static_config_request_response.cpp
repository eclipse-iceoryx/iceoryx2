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

namespace iox2 {
auto StaticConfigRequestResponse::request_message_type_details() const -> MessageTypeDetails {
    return MessageTypeDetails(m_value.request_message_type_details);
}

auto StaticConfigRequestResponse::response_message_type_details() const -> MessageTypeDetails {
    return MessageTypeDetails(m_value.response_message_type_details);
}

auto StaticConfigRequestResponse::has_safe_overflow_for_requests() const -> bool {
    return m_value.enable_safe_overflow_for_requests;
}

auto StaticConfigRequestResponse::has_safe_overflow_for_responses() const -> bool {
    return m_value.enable_safe_overflow_for_responses;
}

auto StaticConfigRequestResponse::max_borrowed_responses_per_pending_responses() const -> uint64_t {
    return m_value.max_borrowed_responses_per_pending_response;
}

auto StaticConfigRequestResponse::max_active_requests_per_client() const -> uint64_t {
    return m_value.max_active_requests_per_client;
}

auto StaticConfigRequestResponse::max_response_buffer_size() const -> uint64_t {
    return m_value.max_response_buffer_size;
}

auto StaticConfigRequestResponse::max_loaned_requests() const -> uint64_t {
    return m_value.max_loaned_requests;
}

auto StaticConfigRequestResponse::does_support_fire_and_forget_requests() const -> bool {
    return m_value.enable_fire_and_forget_requests;
}

auto StaticConfigRequestResponse::max_servers() const -> uint64_t {
    return m_value.max_servers;
}

auto StaticConfigRequestResponse::max_clients() const -> uint64_t {
    return m_value.max_clients;
}

auto StaticConfigRequestResponse::max_nodes() const -> uint64_t {
    return m_value.max_nodes;
}

StaticConfigRequestResponse::StaticConfigRequestResponse(iox2_static_config_request_response_t value)
    : m_value { value } {
}
} // namespace iox2
