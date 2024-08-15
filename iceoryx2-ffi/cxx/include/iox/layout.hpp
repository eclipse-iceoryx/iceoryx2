// Copyright (c) 2024 Contributors to the Eclipse Foundation
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

#ifndef IOX_LAYOUT_HPP
#define IOX_LAYOUT_HPP

#include "iox/expected.hpp"
#include <cstdint>
#include <type_traits>

namespace iox {
/// Defines all errors that can occur while creating a new [`Layout`].
enum class LayoutCreationError : uint8_t {
    /// The provided alignment was not a power of two.
    InvalidAlignment
};

/// Contains a valid [`Layout`], meaning the alignment is a power of two and
/// the size is zero or a multiple of the alignment.
class Layout {
  public:
    /// Creates a new [`Layout`] from the provided type `T` by using `sizeof(T)`
    /// and `alignof(T)`
    template <typename T>
    static auto from() -> std::enable_if_t<!std::is_same_v<T, void>, Layout>;

    /// Creates a new [`Layout`] with size == 0 and alignment == 1.
    template <typename T>
    static auto from() -> std::enable_if_t<std::is_same_v<T, void>, Layout>;

    /// Creates a new [`Layout`] from the given `size` and `align`.
    ///  * If the `size` is not a multiple of `align` it will be rounded up so that it
    ///      becomes a multiple of `align`.
    ///  * If `align` is not a power of two it fails.
    static auto create(uint64_t size, uint64_t align) -> iox::expected<Layout, LayoutCreationError>;

    /// Returns the stored size.
    auto size() const -> uint64_t;

    /// Returns the stored alignment
    auto alignment() const -> uint64_t;

  private:
    static auto is_power_of_two(uint64_t value) -> bool;

    static auto round_up_to(uint64_t value, uint64_t multiple) -> uint64_t;

    Layout(uint64_t size, uint64_t align);

    uint64_t m_size;
    uint64_t m_align;
};


template <typename T>
inline auto Layout::from() -> std::enable_if_t<!std::is_same_v<T, void>, Layout> {
    return Layout { sizeof(T), alignof(T) };
}

template <typename T>
inline auto Layout::from() -> std::enable_if_t<std::is_same_v<T, void>, Layout> {
    return Layout { 0, 1 };
}

inline auto Layout::create(const uint64_t size, const uint64_t align) -> iox::expected<Layout, LayoutCreationError> {
    if (!is_power_of_two(align)) {
        return iox::err(LayoutCreationError::InvalidAlignment);
    }

    return iox::ok(Layout(round_up_to(size, align), align));
}

inline auto Layout::size() const -> uint64_t {
    return m_size;
}

inline auto Layout::alignment() const -> uint64_t {
    return m_align;
}

inline auto Layout::is_power_of_two(const uint64_t value) -> bool {
    return (value != 0) && ((value & (value - 1)) == 0);
}

inline auto Layout::round_up_to(const uint64_t value, const uint64_t multiple) -> uint64_t {
    return (value % multiple == 0) ? value : multiple * (value / multiple + 1);
}

inline Layout::Layout(const uint64_t size, const uint64_t align)
    : m_size { size }
    , m_align { align } {
}


} // namespace iox

#endif
