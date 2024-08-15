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
enum class LayoutCreationError : uint8_t {
    InvalidAlignment
};

class Layout {
  public:
    template <typename T>
    static auto from() -> std::enable_if_t<!std::is_same_v<T, void>, Layout> {
        return Layout { sizeof(T), alignof(T) };
    }

    template <typename T>
    static auto from() -> std::enable_if_t<std::is_same_v<T, void>, Layout> {
        return Layout { 0, 1 };
    }

    static auto create(const uint64_t size, const uint64_t align) -> iox::expected<Layout, LayoutCreationError> {
        if (!is_power_of_two(align)) {
            return iox::err(LayoutCreationError::InvalidAlignment);
        }

        return iox::ok(Layout(round_up_to(size, align), align));
    }

    auto size() const -> uint64_t {
        return m_size;
    }
    auto alignment() const -> uint64_t {
        return m_align;
    }

  private:
    static auto is_power_of_two(const uint64_t value) -> bool {
        return (value != 0) && ((value & (value - 1)) == 0);
    }

    static auto round_up_to(const uint64_t value, const uint64_t multiple) -> uint64_t {
        auto remainder = value % multiple;
        return value + ((remainder == 0U) ? 0U : value - remainder);
    }

    Layout(const uint64_t size, const uint64_t align)
        : m_size { size }
        , m_align { align } {
    }

    uint64_t m_size;
    uint64_t m_align;
};
} // namespace iox

#endif
