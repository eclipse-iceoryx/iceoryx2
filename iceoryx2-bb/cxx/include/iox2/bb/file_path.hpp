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

#ifndef IOX2_BB_FILE_PATH_HPP
#define IOX2_BB_FILE_PATH_HPP

#include "iox2/bb/detail/path_and_file_verifier.hpp"
#include "iox2/bb/semantic_string.hpp"
#include "iox2/bb/static_string.hpp"

namespace iox2 {
namespace bb {
namespace platform {
#ifdef _WIN32
constexpr uint64_t IOX2_MAX_PATH_LENGTH = 255U;
#else
constexpr uint64_t IOX2_MAX_PATH_LENGTH = 1023U;
#endif
} // namespace platform

namespace detail {
auto file_path_does_contain_invalid_characters(const bb::StaticString<platform::IOX2_MAX_PATH_LENGTH>& value) noexcept
    -> bool;
auto file_path_does_contain_invalid_content(const bb::StaticString<platform::IOX2_MAX_PATH_LENGTH>& value) noexcept
    -> bool;
} // namespace detail

/// @brief Represents a path to a file. It is not allowed to end with a path separator
///        since this would then be a path to a directory. A valid file path is for
///        instance "path/to/file" but not "path/to/file/".
class FilePath : public SemanticString<FilePath,
                                       platform::IOX2_MAX_PATH_LENGTH,
                                       detail::file_path_does_contain_invalid_content,
                                       detail::file_path_does_contain_invalid_characters> {
    using Parent = SemanticString<FilePath,
                                  platform::IOX2_MAX_PATH_LENGTH,
                                  detail::file_path_does_contain_invalid_content,
                                  detail::file_path_does_contain_invalid_characters>;
    using Parent::Parent;
};

namespace detail {
inline auto
file_path_does_contain_invalid_characters(const bb::StaticString<platform::IOX2_MAX_PATH_LENGTH>& value) noexcept
    -> bool {
    // NOLINTNEXTLINE(readability-identifier-length)
    for (const char c : value.unchecked_access()) {
        const bool is_small_letter { ASCII_A <= c && c <= ASCII_Z };
        const bool is_capital_letter { ASCII_CAPITAL_A <= c && c <= ASCII_CAPITAL_Z };
        const bool is_number { ASCII_0 <= c && c <= ASCII_9 };
        const bool is_special_character { c == ASCII_DASH || c == ASCII_DOT || c == ASCII_COLON
                                          || c == ASCII_UNDERSCORE };

        const bool is_path_separator { [&]() -> bool {
            // NOLINTNEXTLINE(readability-use-anyofallof) not yet supported in all compilers
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

inline auto
file_path_does_contain_invalid_content(const bb::StaticString<platform::IOX2_MAX_PATH_LENGTH>& value) noexcept -> bool {
    return !is_valid_path_to_file(value);
}
} // namespace detail
} // namespace bb
} // namespace iox2

#endif // IOX2_BB_FILE_PATH_HPP
