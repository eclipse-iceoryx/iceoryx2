// Copyright (c) 2022 by Apex.AI Inc. All rights reserved.
// Copyright (c) 2023 by ekxide IO GmbH. All rights reserved.
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

#ifndef IOX2_BB_DETAIL_PATH_AND_FILE_VERIFIER_HPP
#define IOX2_BB_DETAIL_PATH_AND_FILE_VERIFIER_HPP

#include "iox2/bb/static_string.hpp"

#include <cstdint>

namespace iox2 {
namespace bb {
namespace platform {
#ifdef _WIN32
constexpr uint64_t IOX2_NUMBER_OF_PATH_SEPARATORS = 2U;
// NOLINTNEXTLINE(hicpp-avoid-c-arrays, cppcoreguidelines-avoid-c-arrays, hicpp-explicit-conversions, modernize-avoid-c-arrays)
constexpr const char IOX2_PATH_SEPARATORS[IOX2_NUMBER_OF_PATH_SEPARATORS] = { '/', '\\' };
#else
constexpr uint64_t IOX2_NUMBER_OF_PATH_SEPARATORS = 1U;
// NOLINTNEXTLINE(hicpp-avoid-c-arrays, cppcoreguidelines-avoid-c-arrays, hicpp-explicit-conversions, modernize-avoid-c-arrays)
constexpr const char IOX2_PATH_SEPARATORS[IOX2_NUMBER_OF_PATH_SEPARATORS] = { '/' };
#endif
} // namespace platform

namespace detail {
// AXIVION DISABLE STYLE AutosarC++19_03-A3.9.1: Not used as an integer but as actual character.
constexpr char ASCII_A { 'a' };
constexpr char ASCII_Z { 'z' };
constexpr char ASCII_CAPITAL_A { 'A' };
constexpr char ASCII_CAPITAL_Z { 'Z' };
constexpr char ASCII_0 { '0' };
constexpr char ASCII_9 { '9' };
constexpr char ASCII_DASH { '-' };
constexpr char ASCII_DOT { '.' };
constexpr char ASCII_COLON { ':' };
constexpr char ASCII_UNDERSCORE { '_' };
// AXIVION ENABLE STYLE AutosarC++19_03-A3.9.1

enum class RelativePathComponents : uint8_t {
    Reject,
    Accept
};

/// @brief checks if the given string is a valid path entry. A path entry is the string between
///        two path separators.
/// @note A valid path entry for iceoryx must be platform independent and also supported
///       by various file systems. The file systems we intend to support are
///         * linux: ext3, ext4, btrfs
///         * windows: ntfs, exfat, fat
///         * freebsd: ufs, ffs
///         * apple: apfs
///         * qnx: etfs
///         * android: ext3, ext4, fat
///
///       Sometimes it is also possible that a certain file character is supported by the filesystem
///       itself but not by the platforms SDK. One example are files which end with a dot like "myFile."
///       which are supported by ntfs but not by the Windows SDK.
/// @param[in] name the path entry in question
/// @param[in] relativePathComponents are relative path components are allowed for this path entry
/// @return true if it is valid, otherwise false
template <uint64_t StringCapacity>
auto is_valid_path_entry(const bb::StaticString<StringCapacity>& name,
                         RelativePathComponents relative_path_components) noexcept -> bool;

/// @brief checks if the given string is a valid filename. It must fulfill the
///        requirements of a valid path entry (see, isValidPathEntry) and is not allowed
///        to contain relative path components
/// @param[in] name the string to verify
/// @return true if the string is a filename, otherwise false
template <uint64_t StringCapacity>
auto is_valid_file_name(const bb::StaticString<StringCapacity>& name) noexcept -> bool;

/// @brief returns true if the provided name ends with a path separator, otherwise false
/// @param[in] name the string which may contain a path separator at the end
template <uint64_t StringCapacity>
auto does_end_with_path_separator(const bb::StaticString<StringCapacity>& name) noexcept -> bool;

template <uint64_t StringCapacity>
inline auto is_valid_path_entry(const bb::StaticString<StringCapacity>& name,
                                RelativePathComponents relative_path_components) noexcept -> bool {
    const auto current_directory = bb::StaticString<StringCapacity>::from_utf8_unchecked(".");
    const auto parent_directory = bb::StaticString<StringCapacity>::from_utf8_unchecked("..");

    if ((name == current_directory) || (name == parent_directory)) {
        return relative_path_components == RelativePathComponents::Accept;
    }

    const auto name_size = name.size();

    for (uint64_t i { 0 }; i < name_size; ++i) {
        // AXIVION Next Construct AutosarC++19_03-A3.9.1: Not used as an integer but as actual character
        // NOLINTNEXTLINE(readability-identifier-length)
        const char c { name.unchecked_access()[i] };

        // AXIVION DISABLE STYLE FaultDetection-UnusedAssignments : False positive, variable IS used
        // AXIVION DISABLE STYLE AutosarC++19_03-A0.1.1 : False positive, variable IS used
        // AXIVION DISABLE STYLE AutosarC++19_03-M4.5.3 : We are explicitly checking for ASCII characters which have defined consecutive values
        const bool is_small_letter { (ASCII_A <= c) && (c <= ASCII_Z) };
        const bool is_capital_letter { (ASCII_CAPITAL_A <= c) && (c <= ASCII_CAPITAL_Z) };
        const bool is_number { (ASCII_0 <= c) && (c <= ASCII_9) };
        const bool is_special_character { ((c == ASCII_DASH) || (c == ASCII_DOT))
                                          || ((c == ASCII_COLON) || (c == ASCII_UNDERSCORE)) };
        // AXIVION ENABLE STYLE AutosarC++19_03-M4.5.3
        // AXIVION ENABLE STYLE AutosarC++19_03-A0.1.1
        // AXIVION ENABLE STYLE FaultDetection-UnusedAssignments

        if ((!is_small_letter && !is_capital_letter) && (!is_number && !is_special_character)) {
            return false;
        }
    }

    if (name_size == 0) {
        return true;
    }

    // dot at the end is invalid to be compatible with windows api
    return !(name.unchecked_access()[name_size - 1] == '.');
}

template <uint64_t StringCapacity>
inline auto is_valid_file_name(const bb::StaticString<StringCapacity>& name) noexcept -> bool {
    if (name.empty()) {
        return false;
    }

    // check if the file contains only valid characters
    return is_valid_path_entry(name, RelativePathComponents::Reject);
}

template <uint64_t StringCapacity>
inline auto is_valid_path_to_file(const bb::StaticString<StringCapacity>& name) noexcept -> bool {
    if (does_end_with_path_separator(name)) {
        return false;
    }

    auto maybe_separator = name.code_units().find_last_of(platform::IOX2_PATH_SEPARATORS);
    if (!maybe_separator.has_value()) {
        return is_valid_file_name(name);
    }

    const auto& position = maybe_separator.value();

    bool is_file_name_valid { false };
    auto sub_str = name.code_units().substr(position + 1, name.size());
    if (sub_str.has_value()) {
        is_file_name_valid = is_valid_file_name(*sub_str);
    }

    bool is_path_valid { false };
    sub_str = name.code_units().substr(0, position);
    if (sub_str.has_value()) {
        const bool is_empty_path { sub_str->empty() };
        const bool is_path_to_directory_valid { is_valid_path_to_directory(*sub_str) };
        is_path_valid = is_empty_path || is_path_to_directory_valid;
    }

    // AXIVION Next Construct AutosarC++19_03-M0.1.2, AutosarC++19_03-M0.1.9, FaultDetection-DeadBranches : False positive! Branching depends on input parameter
    return is_path_valid && is_file_name_valid;
}

template <uint64_t StringCapacity>
inline auto is_valid_path_to_directory(const bb::StaticString<StringCapacity>& name) noexcept -> bool {
    if (name.empty()) {
        return false;
    }

    auto const current_directory = bb::StaticString<StringCapacity>::from_utf8_unchecked(".");
    auto const parent_directory = bb::StaticString<StringCapacity>::from_utf8_unchecked("..");

    auto remaining = name;
    while (!remaining.empty()) {
        const auto separator_position = remaining.code_units().find_first_of(platform::IOX2_PATH_SEPARATORS);

        if (separator_position.has_value()) {
            const uint64_t position { separator_position.value() };

            // multiple slashes are explicitly allowed. the following paths
            // are equivalent:
            // /some/fuu/bar
            // //some///fuu////bar

            // verify if the entry between two path separators is a valid directory
            // name, e.g. either it has the relative component . or .. or conforms
            // with a valid file name
            if (position != 0) {
                const auto guaranteed_substr = remaining.code_units().substr(0, position);
                const auto& filename_to_verify = guaranteed_substr.value();
                const bool is_valid_directory { (is_valid_file_name(filename_to_verify))
                                                || ((filename_to_verify == current_directory)
                                                    || (filename_to_verify == parent_directory)) };
                if (!is_valid_directory) {
                    return false;
                }
            }

            auto sub_str = remaining.code_units().substr(position + 1, remaining.size());
            if (sub_str.has_value()) {
                remaining = *sub_str;
            }
        } else // we reached the last entry, if its a valid file name the path is valid
        {
            return is_valid_path_entry(remaining, RelativePathComponents::Accept);
        }
    }

    return true;
}

// AXIVION Next Construct AutosarC++19_03-A5.2.5, AutosarC++19_03-M5.0.16, FaultDetection-OutOfBounds : IOX2_PATH_SEPARATORS is not a string but an array of chars without a null termination and all elements are valid characters
template <uint64_t StringCapacity>
inline auto does_end_with_path_separator(const bb::StaticString<StringCapacity>& name) noexcept -> bool {
    if (name.empty()) {
        return false;
    }
    // AXIVION Next Construct AutosarC++19_03-A3.9.1: Not used as an integer but as actual character
    const char last_character { *name.code_units().back_element() };

    // NOLINTNEXTLINE(readability-use-anyofallof)
    for (const auto separator : platform::IOX2_PATH_SEPARATORS) {
        if (last_character == separator) {
            return true;
        }
    }
    return false;
}

} // namespace detail
} // namespace bb
} // namespace iox2

#endif // IOX2_BB_DETAIL_PATH_AND_FILE_VERIFIER_HPP
