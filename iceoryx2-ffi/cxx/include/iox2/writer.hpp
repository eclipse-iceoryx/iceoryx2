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
    Writer(Writer&& rhs) noexcept;
    auto operator=(Writer&& rhs) noexcept -> Writer&;
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
    template <ServiceType, typename>
    friend class PortFactoryWriter;

    explicit Writer(/*iox2_writer_h handle*/);
    void drop();

    // iox2_writer_h m_handle = nullptr;
};

template <ServiceType S, typename KeyType>
inline Writer<S, KeyType>::Writer(/*iox2_writer_h handle*/) {
    IOX_TODO();
}

template <ServiceType S, typename KeyType>
inline void Writer<S, KeyType>::drop() {
    IOX_TODO();
}

template <ServiceType S, typename KeyType>
inline Writer<S, KeyType>::Writer(Writer&& rhs) noexcept {
    *this = std::move(rhs);
}

template <ServiceType S, typename KeyType>
inline auto Writer<S, KeyType>::operator=([[maybe_unused]] Writer&& rhs) noexcept -> Writer& {
    IOX_TODO();
}

template <ServiceType S, typename KeyType>
inline Writer<S, KeyType>::~Writer() {
    drop();
}

template <ServiceType S, typename KeyType>
inline auto Writer<S, KeyType>::id() const -> UniqueWriterId {
    IOX_TODO();
}

template <ServiceType S, typename KeyType>
template <typename ValueType>
inline auto Writer<S, KeyType>::entry([[maybe_unused]] const KeyType& key)
    -> iox::expected<WriterHandle<S, KeyType, ValueType>, WriterHandleError> {
    IOX_TODO();
}
} // namespace iox2

#endif
