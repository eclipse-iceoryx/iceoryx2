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

#include "iox2/node_id.hpp"

#include <utility>

namespace iox2 {
NodeId::NodeId(iox2_node_id_h handle)
    : m_handle { handle } {
}

NodeId::NodeId(const NodeId& rhs) {
    if (rhs.m_handle != nullptr) {
        iox2_node_id_clone_from_handle(nullptr, &rhs.m_handle, &m_handle);
    }
}

NodeId::NodeId(NodeId&& rhs) noexcept
    : m_handle { std::move(rhs.m_handle) } {
    rhs.m_handle = nullptr;
}

auto NodeId::operator=(NodeId&& rhs) noexcept -> NodeId& {
    if (this != &rhs) {
        drop();
        m_handle = std::move(rhs.m_handle);
        rhs.m_handle = nullptr;
    }

    return *this;
}

auto NodeId::operator=(const NodeId& rhs) -> NodeId& {
    if (this != &rhs) {
        drop();

        if (rhs.m_handle != nullptr) {
            iox2_node_id_clone_from_handle(nullptr, &rhs.m_handle, &m_handle);
        }
    }

    return *this;
}

NodeId::~NodeId() {
    drop();
}

void NodeId::drop() {
    if (m_handle != nullptr) {
        iox2_node_id_drop(m_handle);
        m_handle = nullptr;
    }
}

auto NodeId::value_high() const -> uint64_t {
    return iox2_node_id_value_high(&m_handle);
}

auto NodeId::value_low() const -> uint64_t {
    return iox2_node_id_value_low(&m_handle);
}

auto NodeId::pid() const -> int32_t {
    return iox2_node_id_pid(&m_handle);
}

auto NodeId::creation_time() const -> timespec {
    uint64_t seconds = 0;
    uint32_t nanoseconds = 0;
    iox2_node_id_creation_time(&m_handle, &seconds, &nanoseconds);

    return { static_cast<decltype(timespec::tv_sec)>(seconds), static_cast<decltype(timespec::tv_nsec)>(nanoseconds) };
}

auto operator<<(std::ostream& stream, const NodeId& node) -> std::ostream& {
    stream << "NodeId { value_high: " << node.value_high() << ", value_low: " << node.value_low()
           << ", pid: " << node.pid() << ", creation time: " << node.creation_time().tv_sec << "."
           << node.creation_time().tv_nsec << "s }";
    return stream;
}

auto operator==(const NodeId& lhs, const NodeId& rhs) -> bool {
    return lhs.value_high() == rhs.value_high() && lhs.value_low() == rhs.value_low();
}

auto operator!=(const NodeId& lhs, const NodeId& rhs) -> bool {
    return !(lhs == rhs);
}
} // namespace iox2
