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

#include "iox2/unique_node_id.hpp"

#include <utility>

namespace iox2 {
UniqueNodeId::UniqueNodeId(iox2_unique_node_id_h handle)
    : m_handle { handle } {
}

UniqueNodeId::UniqueNodeId(const UniqueNodeId& rhs) {
    if (rhs.m_handle != nullptr) {
        iox2_unique_node_id_clone_from_handle(nullptr, &rhs.m_handle, &m_handle);
    }
}

UniqueNodeId::UniqueNodeId(UniqueNodeId&& rhs) noexcept
    : m_handle { rhs.m_handle } {
    rhs.m_handle = nullptr;
}

auto UniqueNodeId::operator=(UniqueNodeId&& rhs) noexcept -> UniqueNodeId& {
    if (this != &rhs) {
        drop();
        m_handle = rhs.m_handle;
        rhs.m_handle = nullptr;
    }

    return *this;
}

auto UniqueNodeId::operator=(const UniqueNodeId& rhs) -> UniqueNodeId& {
    if (this != &rhs) {
        drop();

        if (rhs.m_handle != nullptr) {
            iox2_unique_node_id_clone_from_handle(nullptr, &rhs.m_handle, &m_handle);
        }
    }

    return *this;
}

UniqueNodeId::~UniqueNodeId() {
    drop();
}

void UniqueNodeId::drop() {
    if (m_handle != nullptr) {
        iox2_unique_node_id_drop(m_handle);
        m_handle = nullptr;
    }
}

auto UniqueNodeId::value_high() const -> uint64_t {
    return iox2_unique_node_id_value_high(&m_handle);
}

auto UniqueNodeId::value_low() const -> uint64_t {
    return iox2_unique_node_id_value_low(&m_handle);
}

auto UniqueNodeId::pid() const -> int32_t {
    return iox2_unique_node_id_pid(&m_handle);
}

auto UniqueNodeId::creation_time() const -> timespec {
    uint64_t seconds = 0;
    uint32_t nanoseconds = 0;
    iox2_unique_node_id_creation_time(&m_handle, &seconds, &nanoseconds);

    return { static_cast<decltype(timespec::tv_sec)>(seconds), static_cast<decltype(timespec::tv_nsec)>(nanoseconds) };
}

auto operator<<(std::ostream& stream, const UniqueNodeId& node) -> std::ostream& {
    stream << "NodeId { value_high: " << node.value_high() << ", value_low: " << node.value_low()
           << ", pid: " << node.pid() << ", creation time: " << node.creation_time().tv_sec << "."
           << node.creation_time().tv_nsec << "s }";
    return stream;
}

auto operator==(const UniqueNodeId& lhs, const UniqueNodeId& rhs) -> bool {
    return lhs.value_high() == rhs.value_high() && lhs.value_low() == rhs.value_low();
}

auto operator!=(const UniqueNodeId& lhs, const UniqueNodeId& rhs) -> bool {
    return !(lhs == rhs);
}
} // namespace iox2
