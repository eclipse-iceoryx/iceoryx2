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

#include "iox2/server_details.hpp"

namespace iox2 {
ServerDetailsView::ServerDetailsView(iox2_server_details_ptr handle)
    : m_handle { handle } {
}

ServerDetailsView::ServerDetailsView(ServerDetailsView&& rhs) noexcept
    : m_handle { std::move(rhs.m_handle) } {
    rhs.m_handle = nullptr;
}

auto ServerDetailsView::operator=(ServerDetailsView&& rhs) noexcept -> ServerDetailsView& {
    if (this != &rhs) {
        m_handle = rhs.m_handle;
        rhs.m_handle = nullptr;
    }

    return *this;
}

auto ServerDetailsView::server_id() const -> UniqueServerId {
    iox2_unique_server_id_h id_handle = nullptr;
    iox2_server_details_server_id(m_handle, nullptr, &id_handle);
    return UniqueServerId { id_handle };
}

auto ServerDetailsView::node_id() const -> NodeId {
    const auto* node_id_ptr = iox2_server_details_node_id(m_handle);
    iox2_node_id_h id_handle = nullptr;
    iox2_node_id_clone_from_ptr(nullptr, node_id_ptr, &id_handle);
    return NodeId(id_handle);
}

auto ServerDetailsView::request_buffer_size() const -> uint64_t {
    return iox2_server_details_request_buffer_size(m_handle);
}

auto ServerDetailsView::number_of_responses() const -> uint64_t {
    return iox2_server_details_number_of_responses(m_handle);
}

auto ServerDetailsView::max_slice_len() const -> uint64_t {
    return iox2_server_details_max_slice_len(m_handle);
}
} // namespace iox2
