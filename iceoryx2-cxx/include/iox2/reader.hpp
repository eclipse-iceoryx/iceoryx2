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

#ifndef IOX2_READER_HPP
#define IOX2_READER_HPP

#include "iox/expected.hpp"
#include "iox2/entry_handle.hpp"
#include "iox2/entry_handle_error.hpp"
#include "iox2/internal/service_builder_internal.hpp"
#include "iox2/service_type.hpp"
#include "iox2/unique_port_id.hpp"

namespace iox2 {
/// Reading endpoint of a blackboard based communication.
template <ServiceType S, typename KeyType>
class Reader {
  public:
    Reader(Reader&& rhs) noexcept;
    auto operator=(Reader&& rhs) noexcept -> Reader&;
    ~Reader();

    Reader(const Reader&) = delete;
    auto operator=(const Reader&) -> Reader& = delete;

    /// Returns the [`UniqueReaderId`] of the [`Reader`].
    auto id() const -> UniqueReaderId;

    /// Creates an [`EntryHandle`] for direct read access to the value.
    template <typename ValueType>
    auto entry(const KeyType& key) -> iox::expected<EntryHandle<S, KeyType, ValueType>, EntryHandleError>;

  private:
    template <ServiceType, typename>
    friend class PortFactoryReader;

    explicit Reader(iox2_reader_h handle);
    void drop();

    iox2_reader_h m_handle = nullptr;
};

template <ServiceType S, typename KeyType>
inline Reader<S, KeyType>::Reader(iox2_reader_h handle)
    : m_handle { handle } {
}

template <ServiceType S, typename KeyType>
inline void Reader<S, KeyType>::drop() {
    if (m_handle != nullptr) {
        iox2_reader_drop(m_handle);
        m_handle = nullptr;
    }
}

template <ServiceType S, typename KeyType>
inline Reader<S, KeyType>::Reader(Reader&& rhs) noexcept {
    *this = std::move(rhs);
}

template <ServiceType S, typename KeyType>
inline auto Reader<S, KeyType>::operator=(Reader&& rhs) noexcept -> Reader& {
    if (this != &rhs) {
        drop();
        m_handle = std::move(rhs.m_handle);
        rhs.m_handle = nullptr;
    }

    return *this;
}

template <ServiceType S, typename KeyType>
inline Reader<S, KeyType>::~Reader() {
    drop();
}

template <ServiceType S, typename KeyType>
inline auto Reader<S, KeyType>::id() const -> UniqueReaderId {
    iox2_unique_reader_id_h id_handle = nullptr;

    iox2_reader_id(&m_handle, nullptr, &id_handle);
    return UniqueReaderId { id_handle };
}

template <ServiceType S, typename KeyType>
template <typename ValueType>
inline auto Reader<S, KeyType>::entry(const KeyType& key)
    -> iox::expected<EntryHandle<S, KeyType, ValueType>, EntryHandleError> {
    iox2_entry_handle_h entry_handle {};
    const auto type_name = internal::get_type_name<ValueType>();

    auto result = iox2_reader_entry(&m_handle,
                                    nullptr,
                                    &entry_handle,
                                    &key,
                                    type_name.unchecked_access().c_str(),
                                    type_name.size(),
                                    sizeof(ValueType),
                                    alignof(ValueType));

    if (result == IOX2_OK) {
        return iox::ok(EntryHandle<S, KeyType, ValueType>(entry_handle));
    }

    return iox::err(iox::into<EntryHandleError>(result));
}
} // namespace iox2

#endif
