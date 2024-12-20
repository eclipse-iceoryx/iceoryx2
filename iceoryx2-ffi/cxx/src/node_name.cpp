// Copyright (c) 2024 Contributors to the Eclipse Foundation
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

#include "iox2/node_name.hpp"
#include "iox/assertions.hpp"

#include <cstring>

namespace iox2 {
auto NodeNameView::to_string() const -> iox::string<IOX2_NODE_NAME_LENGTH> {
    size_t len = 0;
    const auto* chars = iox2_node_name_as_chars(m_ptr, &len);
    return { iox::TruncateToCapacity, chars, len };
}

auto NodeNameView::to_owned() const -> NodeName {
    size_t len = 0;
    const auto* chars = iox2_node_name_as_chars(m_ptr, &len);
    return NodeName::create_impl(chars, len).expect("NodeNameView contains always valid NodeName");
}

NodeNameView::NodeNameView(iox2_node_name_ptr ptr)
    : m_ptr { ptr } {
}

NodeName::NodeName(iox2_node_name_h handle)
    : m_handle { handle } {
}

NodeName::~NodeName() {
    drop();
}

NodeName::NodeName(NodeName&& rhs) noexcept {
    *this = std::move(rhs);
}

auto NodeName::operator=(NodeName&& rhs) noexcept -> NodeName& {
    if (this != &rhs) {
        drop();
        m_handle = std::move(rhs.m_handle);
        rhs.m_handle = nullptr;
    }

    return *this;
}

NodeName::NodeName(const NodeName& rhs) {
    *this = rhs;
}

auto NodeName::operator=(const NodeName& rhs) -> NodeName& {
    if (this != &rhs) {
        drop();

        const auto* ptr = iox2_cast_node_name_ptr(rhs.m_handle);
        size_t len = 0;
        const auto* chars = iox2_node_name_as_chars(ptr, &len);
        IOX_ASSERT(iox2_node_name_new(nullptr, chars, len, &m_handle) == IOX2_OK,
                   "NodeName shall always contain a valid value.");
    }

    return *this;
}

void NodeName::drop() noexcept {
    if (m_handle != nullptr) {
        iox2_node_name_drop(m_handle);
        m_handle = nullptr;
    }
}

auto NodeName::create(const char* value) -> iox::expected<NodeName, SemanticStringError> {
    return NodeName::create_impl(value, strnlen(value, IOX2_NODE_NAME_LENGTH + 1));
}

auto NodeName::create_impl(const char* value, size_t value_len) -> iox::expected<NodeName, SemanticStringError> {
    if (value_len > IOX2_NODE_NAME_LENGTH) {
        return iox::err(SemanticStringError::ExceedsMaximumLength);
    }

    iox2_node_name_h handle {};
    const auto ret_val = iox2_node_name_new(nullptr, value, value_len, &handle);
    if (ret_val == IOX2_OK) {
        return iox::ok(NodeName { handle });
    }

    return iox::err(iox::into<SemanticStringError>(ret_val));
}

auto NodeName::to_string() const -> iox::string<IOX2_NODE_NAME_LENGTH> {
    return as_view().to_string();
}

auto NodeName::as_view() const -> NodeNameView {
    return NodeNameView(iox2_cast_node_name_ptr(m_handle));
}

} // namespace iox2
