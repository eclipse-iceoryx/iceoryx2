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

#include "iox2/listener_details.hpp"

namespace iox2 {
ListenerDetailsView::ListenerDetailsView(iox2_listener_details_ptr handle)
    : m_handle { handle } {
}

ListenerDetailsView::ListenerDetailsView(ListenerDetailsView&& rhs) noexcept
    : m_handle { std::move(rhs.m_handle) } {
    rhs.m_handle = nullptr;
}

auto ListenerDetailsView::operator=(ListenerDetailsView&& rhs) noexcept -> ListenerDetailsView& {
    if (this != &rhs) {
        m_handle = rhs.m_handle;
        rhs.m_handle = nullptr;
    }

    return *this;
}

auto ListenerDetailsView::listener_id() const -> UniqueListenerId {
    iox2_unique_listener_id_h id_handle = nullptr;
    iox2_listener_details_listener_id(m_handle, nullptr, &id_handle);
    return UniqueListenerId { id_handle };
}

auto ListenerDetailsView::node_id() const -> NodeId {
    const auto* node_id_ptr = iox2_listener_details_node_id(m_handle);
    iox2_node_id_h id_handle = nullptr;
    iox2_node_id_clone_from_ptr(nullptr, node_id_ptr, &id_handle);
    return NodeId(id_handle);
}
} // namespace iox2
