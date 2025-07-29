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

#include "iox2/reader_details.hpp"

namespace iox2 {
ReaderDetailsView::ReaderDetailsView(iox2_reader_details_ptr handle)
    : m_handle { handle } {
}

ReaderDetailsView::ReaderDetailsView(ReaderDetailsView&& rhs) noexcept
    : m_handle { std::move(rhs.m_handle) } {
    rhs.m_handle = nullptr;
}

auto ReaderDetailsView::operator=(ReaderDetailsView&& rhs) noexcept -> ReaderDetailsView& {
    if (this != &rhs) {
        m_handle = rhs.m_handle;
        rhs.m_handle = nullptr;
    }

    return *this;
}

auto ReaderDetailsView::reader_id() const -> UniqueReaderId {
    iox2_unique_reader_id_h id_handle = nullptr;
    iox2_reader_details_reader_id(m_handle, nullptr, &id_handle);
    return UniqueReaderId { id_handle };
}

auto ReaderDetailsView::node_id() const -> NodeId {
    const auto* node_id_ptr = iox2_reader_details_node_id(m_handle);
    iox2_node_id_h id_handle = nullptr;
    iox2_node_id_clone_from_ptr(nullptr, node_id_ptr, &id_handle);
    return NodeId(id_handle);
}
} // namespace iox2

