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

#include "iox2/publisher_details.hpp"

namespace iox2 {
PublisherDetailsView::PublisherDetailsView(iox2_publisher_details_ptr handle)
    : m_handle { handle } {
}

PublisherDetailsView::PublisherDetailsView(PublisherDetailsView&& rhs) noexcept
    : m_handle { std::move(rhs.m_handle) } {
    rhs.m_handle = nullptr;
}

auto PublisherDetailsView::operator=(PublisherDetailsView&& rhs) noexcept -> PublisherDetailsView& {
    if (this != &rhs) {
        m_handle = rhs.m_handle;
        rhs.m_handle = nullptr;
    }

    return *this;
}

auto PublisherDetailsView::publisher_id() const -> UniquePublisherId {
    iox2_unique_publisher_id_h id_handle = nullptr;
    iox2_publisher_details_publisher_id(m_handle, nullptr, &id_handle);
    return UniquePublisherId { id_handle };
}

auto PublisherDetailsView::node_id() const -> NodeId {
    const auto* node_id_ptr = iox2_publisher_details_node_id(m_handle);
    iox2_node_id_h id_handle = nullptr;
    iox2_node_id_clone_from_ptr(nullptr, node_id_ptr, &id_handle);
    return NodeId(id_handle);
}

auto PublisherDetailsView::number_of_samples() const -> uint64_t {
    return iox2_publisher_details_number_of_samples(m_handle);
}

auto PublisherDetailsView::max_slice_len() const -> uint64_t {
    return iox2_publisher_details_max_slice_len(m_handle);
}
} // namespace iox2
