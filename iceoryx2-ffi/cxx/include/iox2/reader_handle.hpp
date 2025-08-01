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

#ifndef IOX2_READER_HANDLE_HPP
#define IOX2_READER_HANDLE_HPP

#include "iox/assertions_addendum.hpp"
#include "iox2/event_id.hpp"
#include "iox2/service_type.hpp"

namespace iox2 {
/// A handle for direct read access to a specific blackboard value.
template <ServiceType S, typename KeyType, typename ValueType>
class ReaderHandle {
  public:
    ReaderHandle(ReaderHandle&& rhs) noexcept;
    auto operator=(ReaderHandle&& rhs) noexcept -> ReaderHandle&;
    ~ReaderHandle();

    ReaderHandle(const ReaderHandle&) = delete;
    auto operator=(const ReaderHandle&) -> ReaderHandle& = delete;

    /// Returns a copy of the value.
    auto get() const -> ValueType;

    /// Returns an ID corresponding to the entry which can be used in an event based communication
    /// setup.
    auto entry_id() const -> EventId;

  private:
    explicit ReaderHandle(/*iox2_reader_handle_h handle*/);
    void drop();

    // iox2_reader_handle_h m_handle = nullptr;
};

template <ServiceType S, typename KeyType, typename ValueType>
inline ReaderHandle<S, KeyType, ValueType>::ReaderHandle(/*iox2_reader_handle_h handle*/) {
    IOX_TODO();
}

template <ServiceType S, typename KeyType, typename ValueType>
inline void ReaderHandle<S, KeyType, ValueType>::drop() {
    IOX_TODO();
}

template <ServiceType S, typename KeyType, typename ValueType>
inline ReaderHandle<S, KeyType, ValueType>::ReaderHandle(ReaderHandle&& rhs) noexcept {
    *this = std::move(rhs);
}

template <ServiceType S, typename KeyType, typename ValueType>
inline auto ReaderHandle<S, KeyType, ValueType>::operator=([[maybe_unused]] ReaderHandle&& rhs) noexcept
    -> ReaderHandle& {
    IOX_TODO();
}

template <ServiceType S, typename KeyType, typename ValueType>
inline ReaderHandle<S, KeyType, ValueType>::~ReaderHandle() {
    drop();
}

template <ServiceType S, typename KeyType, typename ValueType>
inline auto ReaderHandle<S, KeyType, ValueType>::entry_id() const -> EventId {
    IOX_TODO();
}

template <ServiceType S, typename KeyType, typename ValueType>
inline auto ReaderHandle<S, KeyType, ValueType>::get() const -> ValueType {
    IOX_TODO();
}
} // namespace iox2

#endif
