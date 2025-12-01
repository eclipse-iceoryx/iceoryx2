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

#ifndef IOX2_BB_POSIX_VOCABULARY_DETAIL_PATH_AND_FILE_VERIFIER_INL
#define IOX2_BB_POSIX_VOCABULARY_DETAIL_PATH_AND_FILE_VERIFIER_INL

#include "iox2/legacy/detail/path_and_file_verifier.hpp"

namespace iox2 {
namespace legacy {
namespace detail {

template <uint64_t StringCapacity>
inline auto is_valid_path_entry(const iox2::legacy::string<StringCapacity>& name,
                             const RelativePathComponents relative_path_components) noexcept -> bool {
    const iox2::legacy::string<StringCapacity> current_directory { "." };
    const iox2::legacy::string<StringCapacity> parent_directory { ".." };

    if ((name == current_directory) || (name == parent_directory)) {
        return relative_path_components == RelativePathComponents::Accept;
    }

    const auto name_size = name.size();

    for (uint64_t i { 0 }; i < name_size; ++i) {
        // AXIVION Next Construct AutosarC++19_03-A3.9.1: Not used as an integer but as actual character
        // NOLINTNEXTLINE(readability-identifier-length)
        const char c { name[i] };

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
    return !(name[name_size - 1] == '.');
}

template <uint64_t StringCapacity>
inline auto is_valid_file_name(const iox2::legacy::string<StringCapacity>& name) noexcept -> bool {
    if (name.empty()) {
        return false;
    }

    // check if the file contains only valid characters
    return is_valid_path_entry(name, RelativePathComponents::Reject);
}

template <uint64_t StringCapacity>
inline auto is_valid_path_to_file(const iox2::legacy::string<StringCapacity>& name) noexcept -> bool {
    if (does_end_with_path_separator(name)) {
        return false;
    }

    auto maybe_separator = name.find_last_of(iox2::legacy::string<platform::IOX2_NUMBER_OF_PATH_SEPARATORS>(
        TruncateToCapacity, &platform::IOX2_PATH_SEPARATORS[0], platform::IOX2_NUMBER_OF_PATH_SEPARATORS));

    if (!maybe_separator.has_value()) {
        return is_valid_file_name(name);
    }

    const auto& position = maybe_separator.value();

    bool is_file_name_valid { false };
    name.substr(position + 1).and_then([&is_file_name_valid](const auto& str) noexcept -> void {
        is_file_name_valid = is_valid_file_name(str);
    });

    bool is_path_valid { false };
    name.substr(0, position).and_then([&is_path_valid](const auto& str) noexcept -> void {
        const bool is_empty_path { str.empty() };
        const bool is_path_to_directory_valid { is_valid_path_to_directory(str) };
        is_path_valid = is_empty_path || is_path_to_directory_valid;
    });

    // AXIVION Next Construct AutosarC++19_03-M0.1.2, AutosarC++19_03-M0.1.9, FaultDetection-DeadBranches : False positive! Branching depends on input parameter
    return is_path_valid && is_file_name_valid;
}

template <uint64_t StringCapacity>
inline auto is_valid_path_to_directory(const iox2::legacy::string<StringCapacity>& name) noexcept -> bool {
    if (name.empty()) {
        return false;
    }

    const iox2::legacy::string<StringCapacity> current_directory { "." };
    const iox2::legacy::string<StringCapacity> parent_directory { ".." };

    const iox2::legacy::string<platform::IOX2_NUMBER_OF_PATH_SEPARATORS> path_separators {
        TruncateToCapacity, &platform::IOX2_PATH_SEPARATORS[0], platform::IOX2_NUMBER_OF_PATH_SEPARATORS
    };

    auto remaining = name;
    while (!remaining.empty()) {
        const auto separator_position = remaining.find_first_of(path_separators);

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
                const auto guaranteed_substr = remaining.substr(0, position);
                const auto& filename_to_verify = guaranteed_substr.value();
                const bool is_valid_directory { (is_valid_file_name(filename_to_verify))
                                                || ((filename_to_verify == current_directory)
                                                    || (filename_to_verify == parent_directory)) };
                if (!is_valid_directory) {
                    return false;
                }
            }

            remaining.substr(position + 1).and_then([&remaining](const auto& str) noexcept -> void {
                remaining = str;
            });
        } else // we reached the last entry, if its a valid file name the path is valid
        {
            return is_valid_path_entry(remaining, RelativePathComponents::Accept);
        }
    }

    return true;
}

// AXIVION Next Construct AutosarC++19_03-A5.2.5, AutosarC++19_03-M5.0.16, FaultDetection-OutOfBounds : IOX2_PATH_SEPARATORS is not a string but an array of chars without a null termination and all elements are valid characters
template <uint64_t StringCapacity>
inline auto does_end_with_path_separator(const iox2::legacy::string<StringCapacity>& name) noexcept -> bool {
    if (name.empty()) {
        return false;
    }
    // AXIVION Next Construct AutosarC++19_03-A3.9.1: Not used as an integer but as actual character
    const char last_character { name[name.size() - 1U] };

    // NOLINTNEXTLINE(readability-use-anyofallof)
    for (const auto separator : iox2::legacy::platform::IOX2_PATH_SEPARATORS) {
        if (last_character == separator) {
            return true;
        }
    }
    return false;
}

} // namespace detail
} // namespace legacy
} // namespace iox2

#endif // IOX2_BB_POSIX_VOCABULARY_DETAIL_PATH_AND_FILE_VERIFIER_INL
