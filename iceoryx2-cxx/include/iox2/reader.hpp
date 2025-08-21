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

#include "iox/assertions_addendum.hpp"
#include "iox/expected.hpp"
#include "iox2/reader_handle.hpp"
#include "iox2/reader_handle_error.hpp"
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

    /// Returns the [`UniqueReaderId`] of the [`Reader`]
    auto id() const -> UniqueReaderId;

    /// Creates a [`ReaderHandle`] for direct read access to the value.
    template <typename ValueType>
    auto entry(const KeyType& key) -> iox::expected<ReaderHandle<S, KeyType, ValueType>, ReaderHandleError>;

  private:
    template <ServiceType, typename>
    friend class PortFactoryReader;

    explicit Reader(/*iox2_reader_h handle*/);
    void drop();

    // iox2_reader_h m_handle = nullptr;
};

template <ServiceType S, typename KeyType>
inline Reader<S, KeyType>::Reader(/*iox2_reader_h handle*/) {
    IOX_TODO();
}

template <ServiceType S, typename KeyType>
inline void Reader<S, KeyType>::drop() {
    IOX_TODO();
}

template <ServiceType S, typename KeyType>
inline Reader<S, KeyType>::Reader(Reader&& rhs) noexcept {
    *this = std::move(rhs);
}

template <ServiceType S, typename KeyType>
inline auto Reader<S, KeyType>::operator=([[maybe_unused]] Reader&& rhs) noexcept -> Reader& {
    IOX_TODO();
}

template <ServiceType S, typename KeyType>
inline Reader<S, KeyType>::~Reader() {
    drop();
}

template <ServiceType S, typename KeyType>
inline auto Reader<S, KeyType>::id() const -> UniqueReaderId {
    IOX_TODO();
}

template <ServiceType S, typename KeyType>
template <typename ValueType>
inline auto Reader<S, KeyType>::entry([[maybe_unused]] const KeyType& key)
    -> iox::expected<ReaderHandle<S, KeyType, ValueType>, ReaderHandleError> {
    IOX_TODO();
}
} // namespace iox2

#endif
