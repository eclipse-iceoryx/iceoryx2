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

#include "iox2/client_details.hpp"

namespace iox2 {
ClientDetailsView::ClientDetailsView(iox2_client_details_ptr handle)
    : m_handle { handle } {
}

ClientDetailsView::ClientDetailsView(ClientDetailsView&& rhs) noexcept
    : m_handle { std::move(rhs.m_handle) } {
    rhs.m_handle = nullptr;
}

auto ClientDetailsView::operator=(ClientDetailsView&& rhs) noexcept -> ClientDetailsView& {
    if (this != &rhs) {
        m_handle = rhs.m_handle;
        rhs.m_handle = nullptr;
    }

    return *this;
}

auto ClientDetailsView::client_id() const -> UniqueClientId {
    iox2_unique_client_id_h id_handle = nullptr;
    iox2_client_details_client_id(m_handle, nullptr, &id_handle);
    return UniqueClientId { id_handle };
}

auto ClientDetailsView::node_id() const -> NodeId {
    const auto* node_id_ptr = iox2_client_details_node_id(m_handle);
    iox2_node_id_h id_handle = nullptr;
    iox2_node_id_clone_from_ptr(nullptr, node_id_ptr, &id_handle);
    return NodeId(id_handle);
}

auto ClientDetailsView::response_buffer_size() const -> uint64_t {
    return iox2_client_details_response_buffer_size(m_handle);
}

auto ClientDetailsView::number_of_requests() const -> uint64_t {
    return iox2_client_details_number_of_requests(m_handle);
}

auto ClientDetailsView::max_slice_len() const -> uint64_t {
    return iox2_client_details_max_slice_len(m_handle);
}
} // namespace iox2
