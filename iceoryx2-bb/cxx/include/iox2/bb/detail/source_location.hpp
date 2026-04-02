// Copyright (c) 2023 by Apex.AI Inc. All rights reserved.
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

#ifndef IOX2_BB_DETAIL_SOURCE_LOCATION_HPP
#define IOX2_BB_DETAIL_SOURCE_LOCATION_HPP

#include <cstdint>

namespace iox2 {
namespace bb {
namespace detail {
struct SourceLocation {
  private:
    const char* m_file { nullptr };
    const char* m_function { nullptr };
    uint32_t m_line { 0 };

  public:
    static constexpr auto current(const char* file = __builtin_FILE(),
                                  const uint32_t line = __builtin_LINE(),
                                  const char* function = __builtin_FUNCTION()) noexcept -> SourceLocation {
        return SourceLocation { file, line, function };
    }

    constexpr auto file_name() const noexcept -> const char* {
        return m_file;
    }
    constexpr auto line() const noexcept -> uint32_t {
        return m_line;
    }
    constexpr auto function_name() const noexcept -> const char* {
        return m_function;
    }

  private:
    constexpr SourceLocation(const char* file, uint32_t line, const char* function) noexcept
        : m_file(file)
        , m_function(function)
        , m_line(line) {
    }
};

} // namespace detail
} // namespace bb
} // namespace iox2

#endif // IOX2_BB_DETAIL_SOURCE_LOCATION_HPP
