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
#include "iox2/internal/iceoryx2.hpp"

namespace iox2 {
RequestHeader::RequestHeader(RequestHeader&& rhs) noexcept {
    *this = std::move(rhs);
}

auto RequestHeader::operator=(RequestHeader&& rhs) noexcept -> RequestHeader& {
    if (this != &rhs) {
        drop();

        m_handle = rhs.m_handle;
        rhs.m_handle = nullptr;
    }

    return *this;
}

RequestHeader::~RequestHeader() {
    drop();
}

auto RequestHeader::client_port_id() -> UniqueClientId {
    iox2_unique_client_id_h id_handle = nullptr;
    iox2_request_header_client_id(&m_handle, nullptr, &id_handle);
    return UniqueClientId { id_handle };
}

RequestHeader::RequestHeader(iox2_request_header_h handle)
    : m_handle(handle) {
}

void RequestHeader::drop() {
    if (m_handle != nullptr) {
        iox2_request_header_drop(m_handle);
        m_handle = nullptr;
    }
}

auto RequestHeader::number_of_elements() const -> uint64_t {
    return iox2_request_header_number_of_elements(&m_handle);
}

auto RequestHeader::payload_offset() const -> uint64_t {
    return iox2_request_header_payload_offset(&m_handle);
}

ResponseHeader::ResponseHeader(ResponseHeader&& rhs) noexcept {
    *this = std::move(rhs);
}

auto ResponseHeader::operator=(ResponseHeader&& rhs) noexcept -> ResponseHeader& {
    if (this != &rhs) {
        drop();

        m_handle = rhs.m_handle;
        rhs.m_handle = nullptr;
    }

    return *this;
}

ResponseHeader::~ResponseHeader() {
    drop();
}

auto ResponseHeader::server_port_id() -> UniqueServerId {
    iox2_unique_server_id_h id_handle = nullptr;
    iox2_response_header_server_id(&m_handle, nullptr, &id_handle);
    return UniqueServerId { id_handle };
}

ResponseHeader::ResponseHeader(iox2_response_header_h handle)
    : m_handle(handle) {
}

void ResponseHeader::drop() {
    if (m_handle != nullptr) {
        iox2_response_header_drop(m_handle);
        m_handle = nullptr;
    }
}

auto ResponseHeader::payload_offset() const -> uint64_t {
    return iox2_response_header_payload_offset(&m_handle);
}

auto ResponseHeader::number_of_elements() const -> uint64_t {
    return iox2_response_header_number_of_elements(&m_handle);
}
} // namespace iox2
