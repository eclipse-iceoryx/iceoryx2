// Copyright (c) 2026 Contributors to the Eclipse Foundation
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

#include "iox2/listener_key.hpp"

#include <utility>

namespace iox2 {
ListenerKey::ListenerKey(iox2_listener_key_h handle)
    : m_handle { handle } {
}

ListenerKey::ListenerKey(const ListenerKey& rhs) {
    if (rhs.m_handle != nullptr) {
        iox2_listener_key_clone(&rhs.m_handle, nullptr, &m_handle);
    }
}

ListenerKey::ListenerKey(ListenerKey&& rhs) noexcept
    : m_handle { rhs.m_handle } {
    rhs.m_handle = nullptr;
}

auto ListenerKey::operator=(const ListenerKey& rhs) -> ListenerKey& {
    if (this != &rhs) {
        iox2_listener_key_h copy = nullptr;
        if (rhs.m_handle != nullptr) {
            iox2_listener_key_clone(&rhs.m_handle, nullptr, &copy);
        }
        drop();
        m_handle = copy;
    }
    return *this;
}

auto ListenerKey::operator=(ListenerKey&& rhs) noexcept -> ListenerKey& {
    if (this != &rhs) {
        drop();
        m_handle = rhs.m_handle;
        rhs.m_handle = nullptr;
    }
    return *this;
}

ListenerKey::~ListenerKey() noexcept {
    drop();
}

void ListenerKey::drop() noexcept {
    if (m_handle != nullptr) {
        iox2_listener_key_drop(m_handle);
        m_handle = nullptr;
    }
}
} // namespace iox2
