// Copyright (c) 2026 Contributors to the Eclipse Foundation
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

#ifndef IOX2_PORT_NAME_HPP
#define IOX2_PORT_NAME_HPP

#include "iox2/bb/expected.hpp"
#include "iox2/bb/semantic_string.hpp"
#include "iox2/internal/iceoryx2.hpp"

namespace iox2 {

class PortName;

/// Non-owning view of a [`PortName`].
class PortNameView {
  public:
    PortNameView(PortNameView&&) = default;
    PortNameView(const PortNameView&) = default;
    auto operator=(PortNameView&&) -> PortNameView& = default;
    auto operator=(const PortNameView&) -> PortNameView& = default;
    ~PortNameView() = default;

    /// Returns a [`iox2::bb::StaticString`] containing the [`PortName`].
    auto to_string() const -> iox2::bb::StaticString<IOX2_PORT_NAME_LENGTH>;

    /// Creates a copy of the corresponding [`PortName`] and returns it.
    auto to_owned() const -> PortName;

  private:
    friend class PortName;

    explicit PortNameView(iox2_port_name_ptr ptr);
    iox2_port_name_ptr m_ptr = nullptr;
};

/// Represent the name for a [`Port`].
class PortName {
  public:
    PortName(PortName&&) noexcept;
    auto operator=(PortName&&) noexcept -> PortName&;
    PortName(const PortName&);
    auto operator=(const PortName&) -> PortName&;
    ~PortName();

    /// Creates a [`PortNameView`]
    auto as_view() const -> PortNameView;

    /// Creates a new [`PortName`].
    /// If the provided name does not contain a valid [`PortName`] it will return a
    /// [`SemanticStringError`] otherwise the [`PortName`].
    static auto create(const char* value) -> iox2::bb::Expected<PortName, bb::SemanticStringError>;

    /// Returns a [`iox2::bb::StaticString`] containing the [`PortName`].
    auto to_string() const -> iox2::bb::StaticString<IOX2_PORT_NAME_LENGTH>;

  private:
    friend class PortNameView;

    explicit PortName(iox2_port_name_h handle);
    void drop() noexcept;
    static auto create_impl(const char* value, size_t value_len)
        -> iox2::bb::Expected<PortName, bb::SemanticStringError>;

    iox2_port_name_h m_handle = nullptr;
};
} // namespace iox2

#endif
