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

#ifndef IOX2_ENTRY_VALUE_UNINIT_HPP
#define IOX2_ENTRY_VALUE_UNINIT_HPP

#include "iox/assertions_addendum.hpp"
#include "iox2/entry_value.hpp"
#include "iox2/service_type.hpp"

namespace iox2 {
template <ServiceType, typename, typename>
class EntryHandleMut;

/// Wrapper around an uninitiaized entry value that can be used for a zero-copy update.
template <ServiceType S, typename KeyType, typename ValueType>
class EntryValueUninit {
  public:
    EntryValueUninit(EntryValueUninit&&) noexcept = default;
    auto operator=(EntryValueUninit&&) noexcept -> EntryValueUninit& = default;
    ~EntryValueUninit() noexcept = default;

    EntryValueUninit(const EntryValueUninit&) = delete;
    auto operator=(const EntryValueUninit&) -> EntryValueUninit& = delete;

    /// Consumes the [`EntryValueUninit`], writes value to the entry value and returns the
    /// initialized [`EntryValue`].
    // TODO: check ValueType with enable_if?
    template <ServiceType ST, typename KeyT, typename ValueT>
    friend auto write(EntryValueUninit<ST, KeyT, ValueT>&& self, ValueT value) -> EntryValue<ST, KeyT, ValueT>;

    /// Discard the [`EntryValueUninit`] and returns the original [`EntryHandleMut`].
    template <ServiceType ST, typename KeyT, typename ValueT>
    friend auto discard(EntryValueUninit<ST, KeyT, ValueT>&& self) -> EntryHandleMut<ST, KeyT, ValueT>;

  private:
    template <ServiceType, typename, typename>
    friend class EntryHandleMut;

    template <ServiceType ST, typename KeyT, typename ValueT>
    friend auto loan_uninit(EntryHandleMut<ST, KeyT, ValueT>&&) -> EntryValueUninit<ST, KeyT, ValueT>;

    // The EntryValueUninit is defaulted since the member is initialized in
    // EntryHandleMut::loan_uninit().
    explicit EntryValueUninit() = default;

    EntryValue<S, KeyType, ValueType> m_entry_value;
    // iox2_entry_value_uninit_h m_handle = nullptr;
};

template <ServiceType S, typename KeyType, typename ValueType>
inline auto write(EntryValueUninit<S, KeyType, ValueType>&& self, ValueType value)
    -> EntryValue<S, KeyType, ValueType> {
    std::cout << "write" << std::endl;
    new (&self.m_entry_value.value_mut()) ValueType(std::forward<ValueType>(value));
    return std::move(self.m_entry_value);
}

template <ServiceType S, typename KeyType, typename ValueType>
inline auto discard(EntryValueUninit<S, KeyType, ValueType>&& self) -> EntryHandleMut<S, KeyType, ValueType> {
    iox2_entry_handle_mut_h entry_handle_mut_handle = nullptr;

    iox2_entry_value_discard(self.m_entry_value.m_handle, nullptr, &entry_handle_mut_handle);

    EntryHandleMut<S, KeyType, ValueType> entry_handle_mut(entry_handle_mut_handle);
    return std::move(entry_handle_mut);
}
} // namespace iox2

#endif
