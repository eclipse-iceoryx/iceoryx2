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

#include "iox2/subscriber_details.hpp"

namespace iox2 {
SubscriberDetailsView::SubscriberDetailsView(iox2_subscriber_details_ptr handle)
    : m_handle { handle } {
}

SubscriberDetailsView::SubscriberDetailsView(SubscriberDetailsView&& rhs) noexcept
    : m_handle { std::move(rhs.m_handle) } {
    rhs.m_handle = nullptr;
}

auto SubscriberDetailsView::operator=(SubscriberDetailsView&& rhs) noexcept -> SubscriberDetailsView& {
    if (this != &rhs) {
        m_handle = rhs.m_handle;
        rhs.m_handle = nullptr;
    }

    return *this;
}

auto SubscriberDetailsView::subscriber_id() const -> UniqueSubscriberId {
    iox2_unique_subscriber_id_h id_handle = nullptr;
    iox2_subscriber_details_subscriber_id(m_handle, nullptr, &id_handle);
    return UniqueSubscriberId { id_handle };
}

auto SubscriberDetailsView::node_id() const -> NodeId {
    const auto* node_id_ptr = iox2_subscriber_details_node_id(m_handle);
    iox2_node_id_h id_handle = nullptr;
    iox2_node_id_clone_from_ptr(nullptr, node_id_ptr, &id_handle);
    return NodeId(id_handle);
}

auto SubscriberDetailsView::buffer_size() const -> uint64_t {
    return iox2_subscriber_details_buffer_size(m_handle);
}
} // namespace iox2
