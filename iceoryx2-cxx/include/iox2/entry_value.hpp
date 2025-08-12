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

#ifndef IOX2_ENTRY_VALUE_HPP
#define IOX2_ENTRY_VALUE_HPP

#include "iox/assertions_addendum.hpp"
// #include "iox2/entry_handle_mut.hpp"
#include "iox2/service_type.hpp"
#include <iostream>

namespace iox2 {
template <ServiceType, typename, typename>
class EntryHandleMut;
template <ServiceType, typename, typename>
class EntryValueUninit;

/// Wrapper around an initialized entry value that can be used for a zero-copy update.
template <ServiceType S, typename KeyType, typename ValueType>
class EntryValue {
  public:
    EntryValue(EntryValue&& rhs) noexcept;
    auto operator=(EntryValue&& rhs) noexcept -> EntryValue&;
    ~EntryValue() noexcept;

    EntryValue(const EntryValue&) = delete;
    auto operator=(const EntryValue&) -> EntryValue& = delete;

    /// Makes new value readable for [`Reader`]s, consumes the
    /// [`EntryValue`] and returns the original [`WriterHandle`].
    template <ServiceType ST, typename KeyT, typename ValueT>
    // friend auto update(EntryValue<ST, KeyT, ValueT>&& self) -> EntryHandleMut<S, KeyType, ValueType>;
    friend auto update(EntryValue<ST, KeyT, ValueT>&& self);

    /// Discard the [`EntryValue`] and returns the original [`WriterHandle`].
    template <ServiceType ST, typename KeyT, typename ValueT>
    // friend auto discard(EntryValue<ST, KeyT, ValueT>&& self) -> EntryHandleMut<S, KeyType, ValueType>;
    friend auto discard(EntryValue<ST, KeyT, ValueT>&& self);

    // TODO: make private
    auto value_mut() -> ValueType&;

  private:
    template <ServiceType, typename, typename>
    friend class EntryHandleMut;
    template <ServiceType, typename, typename>
    friend class EntryValueUninit;
    template <ServiceType ST, typename KeyT, typename ValueT>
    friend auto loan_uninit(EntryHandleMut<ST, KeyT, ValueT>&&) -> EntryValueUninit<ST, KeyT, ValueT>;
    // template <ServiceType ST, typename KeyT, typename ValueT>
    // friend auto write(EntryValueUninit<ST, KeyT, ValueT>&&, ValueT) -> EntryValue<ST, KeyT, ValueT>;

    // The EntryValue is defaulted since the member is initialized in
    // EntryHandleMut::loan_uninit()
    explicit EntryValue() = default;

    void drop();

    iox2_entry_value_t m_entry_value;
    iox2_entry_value_h m_handle = nullptr;
};

// NOLINTNEXTLINE(cppcoreguidelines-pro-type-member-init,hicpp-member-init) m_entry_value will be initialized in the move assignment operator
template <ServiceType S, typename KeyType, typename ValueType>
inline EntryValue<S, KeyType, ValueType>::EntryValue(EntryValue&& rhs) noexcept {
    *this = std::move(rhs);
}

namespace internal {
extern "C" {
void iox2_entry_value_move(iox2_entry_value_t*, iox2_entry_value_t*, iox2_entry_value_h*);
}
} // namespace internal

template <ServiceType S, typename KeyType, typename ValueType>
inline auto EntryValue<S, KeyType, ValueType>::operator=(EntryValue&& rhs) noexcept -> EntryValue& {
    if (this != &rhs) {
        drop();

        internal::iox2_entry_value_move(&rhs.m_entry_value, &m_entry_value, &m_handle);
        rhs.m_handle = nullptr;
    }

    return *this;
}

template <ServiceType S, typename KeyType, typename ValueType>
inline EntryValue<S, KeyType, ValueType>::~EntryValue() noexcept {
    drop();
}

template <ServiceType S, typename KeyType, typename ValueType>
inline auto update(EntryValue<S, KeyType, ValueType>&& self) {
    iox2_entry_value_update(self.m_handle);
}

template <ServiceType S, typename KeyType, typename ValueType>
inline auto discard([[maybe_unused]] EntryValue<S, KeyType, ValueType>&& self) {
    IOX_TODO();
}

template <ServiceType S, typename KeyType, typename ValueType>
inline auto EntryValue<S, KeyType, ValueType>::value_mut() -> ValueType& {
    void* value_ptr = nullptr;
    iox2_entry_value_mut(&m_handle, &value_ptr);
    if (value_ptr == nullptr) {
        std::cout << "OHA" << std::endl;
    }
    std::cout << "C++ value_ptr = " << value_ptr << std::endl;
    return *static_cast<ValueType*>(value_ptr);
}

template <ServiceType S, typename KeyType, typename ValueType>
inline void EntryValue<S, KeyType, ValueType>::drop() {
    if (m_handle != nullptr) {
        iox2_entry_value_drop(m_handle);
        m_handle = nullptr;
    }
}
} // namespace iox2

#endif
