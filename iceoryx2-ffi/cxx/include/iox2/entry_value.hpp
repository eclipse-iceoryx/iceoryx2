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
#include "iox2/service_type.hpp"
#include "iox2/writer_handle.hpp"

namespace iox2 {
/// Wrapper around an initialized entry value that can be used for a zero-copy update.
template <ServiceType S, typename KeyType, typename ValueType>
class EntryValue {
  public:
    EntryValue(EntryValue&&) noexcept = default;
    auto operator=(EntryValue&&) noexcept -> EntryValue& = default;
    ~EntryValue() = default;

    EntryValue(const EntryValue&) = delete;
    auto operator=(const EntryValue&) -> EntryValue& = delete;

    /// Makes new value readable for [`Reader`]s, consumes the
    /// [`EntryValue`] and returns the original [`WriterHandle`].
    template <ServiceType ST, typename KeyT, typename ValueT>
    friend auto update(EntryValue<ST, KeyT, ValueT>&& self) -> WriterHandle<S, KeyType, ValueType>;

    /// Discard the [`EntryValue`] and returns the original [`WriterHandle`].
    template <ServiceType ST, typename KeyT, typename ValueT>
    friend auto discard(EntryValue<ST, KeyT, ValueT>&& self) -> WriterHandle<S, KeyType, ValueType>;

  private:
    // The EntryValue is defaulted since the member is initialized in
    // EntryValueUninit::write()
    explicit EntryValue() = default;

    WriterHandle<S, KeyType, ValueType> m_writer_handle;
};

template <ServiceType S, typename KeyType, typename ValueType>
inline auto update([[maybe_unused]] EntryValue<S, KeyType, ValueType>&& self) -> WriterHandle<S, KeyType, ValueType> {
    IOX_TODO();
}

template <ServiceType S, typename KeyType, typename ValueType>
inline auto discard([[maybe_unused]] EntryValue<S, KeyType, ValueType>&& self) -> WriterHandle<S, KeyType, ValueType> {
    IOX_TODO();
}
} // namespace iox2

#endif
