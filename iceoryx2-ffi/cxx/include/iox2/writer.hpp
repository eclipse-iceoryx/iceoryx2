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

#ifndef IOX2_WRITER_HPP
#define IOX2_WRITER_HPP

#include "iox/assertions_addendum.hpp"
#include "iox/expected.hpp"
#include "iox2/service_type.hpp"
#include "iox2/unique_port_id.hpp"
#include "iox2/writer_handle.hpp"
#include "iox2/writer_handle_error.hpp"

namespace iox2 {
/// Producing endpoint of a blackboard based communication.
template <ServiceType S, typename KeyType>
class Writer {
  public:
    Writer(Writer&&) noexcept;
    auto operator=(Writer&&) noexcept -> Writer&;
    ~Writer();

    Writer(const Writer&) = delete;
    auto operator=(const Writer&) -> Writer& = delete;

    /// Returns the [`UniqueWriterId`] of the [`Writer`]
    auto id() const -> UniqueWriterId;

    /// Creates a [`WriterHandle`] for direct write access to the value. There can be only one
    /// [`WriterHandle`] per value.
    template <typename ValueType>
    auto entry(const KeyType& key) -> iox::expected<WriterHandle<S, KeyType, ValueType>, WriterHandleError>;

  private:
    template <ServiceType, KeyType>
    friend class PortFactoryWriter;

    explicit Writer(iox2_writer_h handle);
    void drop();

    iox2_writer_h m_handle = nullptr;
};

template <ServiceType S, typename KeyType>
inline Writer<S, KeyType>::Writer(iox2_writer_h handle)
    : m_handle { handle } {
}

template <ServiceType S, typename KeyType>
inline void Writer<S, KeyType>::drop() {
    if (m_handle != nullptr) {
        iox2_writer_drop(m_handle);
        m_handle = nullptr;
    }
}

template <ServiceType S, typename KeyType>
inline Writer<S, KeyType>::Writer(Writer&& rhs) noexcept {
    *this = std::move(rhs);
}

template <ServiceType S, typename KeyType>
inline auto Writer<S, KeyType>::operator=(Writer&& rhs) noexcept -> Writer& {
    if (this != &rhs) {
        drop();
        m_handle = std::move(rhs.m_handle);
        rhs.m_handle = nullptr;
    }

    return *this;
}

template <ServiceType S, typename KeyType>
inline Writer<S, KeyType>::~Writer() {
    drop();
}

template <ServiceType S, typename KeyType>
inline auto Writer<S, KeyType>::id() const -> UniqueWriterId {
    iox2_unique_writer_id_h id_handle = nullptr;

    iox2_writer_id(&m_handle, nullptr, &id_handle);
    return UniqueWriterId { id_handle };
}

template <ServiceType S, typename KeyType>
template <typename ValueType>
inline auto Writer<S, KeyType>::entry(const KeyType& key)
    -> iox::expected<WriterHandle<S, KeyType, ValueType>, WriterHandleError> {
    IOX_TODO();
}
} // namespace iox2

#endif
