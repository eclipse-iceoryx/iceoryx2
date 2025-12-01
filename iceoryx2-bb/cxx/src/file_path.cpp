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
bool file_path_does_contain_invalid_characters(const string<platform::IOX2_MAX_PATH_LENGTH>& value) noexcept {
    const auto valueSize = value.size();

    for (uint64_t i { 0 }; i < valueSize; ++i) {
        // AXIVION Next Construct AutosarC++19_03-A3.9.1: Not used as an integer but as actual character
        const char c { value.unchecked_at(i) };

        const bool isSmallLetter { detail::ASCII_A <= c && c <= detail::ASCII_Z };
        const bool isCapitalLetter { detail::ASCII_CAPITAL_A <= c && c <= detail::ASCII_CAPITAL_Z };
        const bool isNumber { detail::ASCII_0 <= c && c <= detail::ASCII_9 };
        const bool isSpecialCharacter { c == detail::ASCII_DASH || c == detail::ASCII_DOT || c == detail::ASCII_COLON
                                        || c == detail::ASCII_UNDERSCORE };

        const bool isPathSeparator { [&] {
            for (const auto separator : platform::IOX2_PATH_SEPARATORS) {
                if (c == separator) {
                    return true;
                }
            }
            return false;
        }() };

        if ((!isSmallLetter && !isCapitalLetter) && (!isNumber && !isSpecialCharacter) && !isPathSeparator) {
            return true;
        }
    }

    return false;
}

bool file_path_does_contain_invalid_content(const string<platform::IOX2_MAX_PATH_LENGTH>& value) noexcept {
    return !isValidPathToFile(value);
}
} // namespace detail
} // namespace legacy
} // namespace iox2
