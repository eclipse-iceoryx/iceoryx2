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

#include "iox2/legacy/file_path.hpp"
#include "iox2/legacy/detail/path_and_file_verifier.hpp"
#include "iox2/legacy/string.hpp"

namespace iox2 {
namespace legacy {
namespace detail {
auto file_path_does_contain_invalid_characters(const string<platform::IOX2_MAX_PATH_LENGTH>& value) noexcept -> bool {
    const auto value_size = value.size();

    for (uint64_t i { 0 }; i < value_size; ++i) {
        // AXIVION Next Construct AutosarC++19_03-A3.9.1: Not used as an integer but as actual character
        // NOLINTNEXTLINE(readability-identifier-length)
        const char c { value.unchecked_at(i) };

        const bool is_small_letter { detail::ASCII_A <= c && c <= detail::ASCII_Z };
        const bool is_capital_letter { detail::ASCII_CAPITAL_A <= c && c <= detail::ASCII_CAPITAL_Z };
        const bool is_number { detail::ASCII_0 <= c && c <= detail::ASCII_9 };
        const bool is_special_character { c == detail::ASCII_DASH || c == detail::ASCII_DOT || c == detail::ASCII_COLON
                                          || c == detail::ASCII_UNDERSCORE };

        const bool is_path_separator { [&]() -> bool {
            // NOLINTNEXTLINE(readability-use-anyofallof)
            for (const auto separator : platform::IOX2_PATH_SEPARATORS) {
                if (c == separator) {
                    return true;
                }
            }
            return false;
        }() };

        if ((!is_small_letter && !is_capital_letter) && (!is_number && !is_special_character) && !is_path_separator) {
            return true;
        }
    }

    return false;
}

auto file_path_does_contain_invalid_content(const string<platform::IOX2_MAX_PATH_LENGTH>& value) noexcept -> bool {
    return !is_valid_path_to_file(value);
}
} // namespace detail
} // namespace legacy
} // namespace iox2
