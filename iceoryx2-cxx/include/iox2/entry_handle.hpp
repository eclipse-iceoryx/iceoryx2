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

#ifndef IOX2_ENTRY_HANDLE_HPP
#define IOX2_ENTRY_HANDLE_HPP

#include "iox2/event_id.hpp"
#include "iox2/service_type.hpp"

namespace iox2 {
/// A handle for direct read access to a specific blackboard value.
template <ServiceType S, typename KeyType, typename ValueType>
class EntryHandle {
  public:
    EntryHandle(EntryHandle&& rhs) noexcept;
    auto operator=(EntryHandle&& rhs) noexcept -> EntryHandle&;
    ~EntryHandle();

    EntryHandle(const EntryHandle&) = delete;
    auto operator=(const EntryHandle&) -> EntryHandle& = delete;

    /// Returns a copy of the value.
    auto get() const -> ValueType;

    /// Returns an ID corresponding to the entry which can be used in an event based communication
    /// setup.
    auto entry_id() const -> EventId;

  private:
    template <ServiceType, typename>
    friend class Reader;

    explicit EntryHandle(iox2_entry_handle_h handle);
    void drop();

    iox2_entry_handle_h m_handle = nullptr;
};

template <ServiceType S, typename KeyType, typename ValueType>
inline EntryHandle<S, KeyType, ValueType>::EntryHandle(iox2_entry_handle_h handle)
    : m_handle { handle } {
}

template <ServiceType S, typename KeyType, typename ValueType>
inline void EntryHandle<S, KeyType, ValueType>::drop() {
    if (m_handle != nullptr) {
        iox2_entry_handle_drop(m_handle);
        m_handle = nullptr;
    }
}

template <ServiceType S, typename KeyType, typename ValueType>
inline EntryHandle<S, KeyType, ValueType>::EntryHandle(EntryHandle&& rhs) noexcept {
    *this = std::move(rhs);
}

template <ServiceType S, typename KeyType, typename ValueType>
inline auto EntryHandle<S, KeyType, ValueType>::operator=(EntryHandle&& rhs) noexcept -> EntryHandle& {
    if (this != &rhs) {
        drop();
        m_handle = std::move(rhs.m_handle);
        rhs.m_handle = nullptr;
    }

    return *this;
}

template <ServiceType S, typename KeyType, typename ValueType>
inline EntryHandle<S, KeyType, ValueType>::~EntryHandle() {
    drop();
}

template <ServiceType S, typename KeyType, typename ValueType>
inline auto EntryHandle<S, KeyType, ValueType>::entry_id() const -> EventId {
    iox2_event_id_t entry_id {};

    iox2_entry_handle_entry_id(&m_handle, &entry_id);

    return EventId { entry_id };
}

template <ServiceType S, typename KeyType, typename ValueType>
inline auto EntryHandle<S, KeyType, ValueType>::get() const -> ValueType {
    ValueType value;

    iox2_entry_handle_get(&m_handle, &value, sizeof(ValueType), alignof(ValueType));

    return value;
}
} // namespace iox2

#endif
