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

#ifndef IOX2_ENTRY_HANDLE_MUT_HPP
#define IOX2_ENTRY_HANDLE_MUT_HPP

#include "iox/assertions_addendum.hpp"
#include "iox2/entry_value_uninit.hpp"
#include "iox2/event_id.hpp"
#include "iox2/service_type.hpp"

namespace iox2 {
// template <ServiceType, typename, typename>
// class EntryValueUninit;

/// A handle for direct write access to a specific blackboard value.
template <ServiceType S, typename KeyType, typename ValueType>
class EntryHandleMut {
  public:
    EntryHandleMut(EntryHandleMut&& rhs) noexcept;
    auto operator=(EntryHandleMut&& rhs) noexcept -> EntryHandleMut&;
    ~EntryHandleMut() noexcept;

    EntryHandleMut(const EntryHandleMut&) = delete;
    auto operator=(const EntryHandleMut&) -> EntryHandleMut& = delete;

    /// Updates the value by copying the passed value into it.
    void update_with_copy(ValueType value);

    /// Consumes the [`EntryHandleMut`] and loans an uninitialized entry value that can be used to update without copy.
    template <ServiceType ST, typename KeyT, typename ValueT>
    friend auto loan_uninit(EntryHandleMut<ST, KeyT, ValueT>&& self) -> EntryValueUninit<ST, KeyT, ValueT>;

    /// Returns an ID corresponding to the entry which can be used in an event based communication
    /// setup.
    auto entry_id() const -> EventId;

  private:
    // template <ServiceType, typename, typename>
    // friend class EntryValueUninit;
    // template <ServiceType, typename, typename>
    // friend class EntryValue;
    template <ServiceType, typename>
    friend class Writer;
    template <ServiceType ST, typename KeyT, typename ValueT>
    friend auto update(EntryValue<ST, KeyT, ValueT>&& self) -> EntryHandleMut<ST, KeyT, ValueT>;
    template <ServiceType ST, typename KeyT, typename ValueT>
    friend auto discard(EntryValue<ST, KeyT, ValueT>&& self) -> EntryHandleMut<ST, KeyT, ValueT>;
    template <ServiceType ST, typename KeyT, typename ValueT>
    friend auto discard(EntryValueUninit<ST, KeyT, ValueT>&& self) -> EntryHandleMut<ST, KeyT, ValueT>;

    // TODO: explain default
    explicit EntryHandleMut(iox2_entry_handle_mut_h handle);
    void drop();

    iox2_entry_handle_mut_h m_handle = nullptr;
    // EntryValueUninit<S, KeyType, ValueType> m_entry_value_uninit;
};

template <ServiceType S, typename KeyType, typename ValueType>
inline EntryHandleMut<S, KeyType, ValueType>::EntryHandleMut(iox2_entry_handle_mut_h handle)
    : m_handle { handle } {
}

template <ServiceType S, typename KeyType, typename ValueType>
inline EntryHandleMut<S, KeyType, ValueType>::EntryHandleMut(EntryHandleMut&& rhs) noexcept {
    *this = std::move(rhs);
}

template <ServiceType S, typename KeyType, typename ValueType>
inline auto EntryHandleMut<S, KeyType, ValueType>::operator=(EntryHandleMut&& rhs) noexcept -> EntryHandleMut& {
    if (this != &rhs) {
        drop();
        m_handle = std::move(rhs.m_handle);
        rhs.m_handle = nullptr;
    }

    return *this;
}

template <ServiceType S, typename KeyType, typename ValueType>
inline EntryHandleMut<S, KeyType, ValueType>::~EntryHandleMut() noexcept {
    std::cout << "~EntryHandleMut" << std::endl;
    drop();
}

template <ServiceType S, typename KeyType, typename ValueType>
inline void EntryHandleMut<S, KeyType, ValueType>::update_with_copy(ValueType value) {
    // only C++, not C
    auto value_ptr = new ValueType(value);
    iox2_entry_handle_mut_update_with_copy(&m_handle, value_ptr, sizeof(ValueType), alignof(ValueType));
    delete value_ptr;
}

template <ServiceType S, typename KeyType, typename ValueType>
inline auto loan_uninit(EntryHandleMut<S, KeyType, ValueType>&& self) -> EntryValueUninit<S, KeyType, ValueType> {
    // C++ and C
    EntryValueUninit<S, KeyType, ValueType> entry_value_uninit;

    iox2_entry_handle_mut_loan_uninit(self.m_handle,
                                      &entry_value_uninit.m_entry_value.m_entry_value,
                                      &entry_value_uninit.m_entry_value.m_handle,
                                      sizeof(ValueType),
                                      alignof(ValueType));

    return std::move(entry_value_uninit);
}

template <ServiceType S, typename KeyType, typename ValueType>
inline auto EntryHandleMut<S, KeyType, ValueType>::entry_id() const -> EventId {
    IOX_TODO();
}

template <ServiceType S, typename KeyType, typename ValueType>
inline void EntryHandleMut<S, KeyType, ValueType>::drop() {
    if (m_handle == nullptr) {
        std::cout << "EntryHandleMut::drop(), m_handle == nullptr" << std::endl;
    } else {
        std::cout << "EntryHandleMut::drop(), m_handle != nullptr" << std::endl;
    }
    if (m_handle != nullptr) {
        iox2_entry_handle_mut_drop(m_handle);
        m_handle = nullptr;
    }
}
} // namespace iox2

#endif
