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

#ifndef IOX2_WRITER_HANDLE_HPP
#define IOX2_WRITER_HANDLE_HPP

#include "iox/assertions_addendum.hpp"
#include "iox2/event_id.hpp"
#include "iox2/service_type.hpp"

namespace iox2 {
template <ServiceType, typename, typename>
class EntryValueUninit;

/// A handle for direct write access to a specific blackboard value.
template <ServiceType S, typename KeyType, typename ValueType>
class WriterHandle {
  public:
    WriterHandle(WriterHandle&& rhs) noexcept;
    auto operator=(WriterHandle&& rhs) noexcept -> WriterHandle&;
    ~WriterHandle();

    WriterHandle(const WriterHandle&) = delete;
    auto operator=(const WriterHandle&) -> WriterHandle& = delete;

    /// Updates the value by copying the passed value into it.
    void update_with_copy(ValueType value);

    /// Consumes the [`WriterHandle`] and loans an uninitialized entry value that can be used to update without copy.
    auto loan_uninit() -> EntryValueUninit<S, KeyType, ValueType>;

    /// Returns an ID corresponding to the entry which can be used in an event based communication
    /// setup.
    auto entry_id() const -> EventId;

  private:
    template <ServiceType, typename, typename>
    friend class EntryValueUninit;
    template <ServiceType, typename, typename>
    friend class EntryValue;

    explicit WriterHandle(/*iox2_writer_handle_h handle*/);
    void drop();

    // iox2_writer_handle_h m_handle = nullptr;
};

template <ServiceType S, typename KeyType, typename ValueType>
inline WriterHandle<S, KeyType, ValueType>::WriterHandle(/*iox2_writer_handle_h handle*/) {
    IOX_TODO();
}

template <ServiceType S, typename KeyType, typename ValueType>
inline void WriterHandle<S, KeyType, ValueType>::drop() {
    IOX_TODO();
}

template <ServiceType S, typename KeyType, typename ValueType>
inline WriterHandle<S, KeyType, ValueType>::WriterHandle(WriterHandle&& rhs) noexcept {
    *this = std::move(rhs);
}

template <ServiceType S, typename KeyType, typename ValueType>
inline auto WriterHandle<S, KeyType, ValueType>::operator=([[maybe_unused]] WriterHandle&& rhs) noexcept
    -> WriterHandle& {
    IOX_TODO();
}

template <ServiceType S, typename KeyType, typename ValueType>
inline WriterHandle<S, KeyType, ValueType>::~WriterHandle() {
    drop();
}

template <ServiceType S, typename KeyType, typename ValueType>
inline void WriterHandle<S, KeyType, ValueType>::update_with_copy([[maybe_unused]] ValueType value) {
    IOX_TODO();
}

template <ServiceType S, typename KeyType, typename ValueType>
inline auto WriterHandle<S, KeyType, ValueType>::loan_uninit() -> EntryValueUninit<S, KeyType, ValueType> {
    IOX_TODO();
}

template <ServiceType S, typename KeyType, typename ValueType>
inline auto WriterHandle<S, KeyType, ValueType>::entry_id() const -> EventId {
    IOX_TODO();
}
} // namespace iox2

#endif
