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

#ifndef IOX2_BB_POSIX_VOCABULARY_FILE_NAME_HPP
#define IOX2_BB_POSIX_VOCABULARY_FILE_NAME_HPP

#include "iox2/legacy/semantic_string.hpp"

namespace iox2 {
namespace legacy {
namespace platform {
#if defined(_WIN32)
constexpr uint64_t IOX2_MAX_FILENAME_LENGTH = 128U;
#else
constexpr uint64_t IOX2_MAX_FILENAME_LENGTH = 255U;
#endif
} // namespace platform

namespace detail {
auto file_name_does_contain_invalid_characters(const string<platform::IOX2_MAX_FILENAME_LENGTH>& value) noexcept
    -> bool;
auto file_name_does_contain_invalid_content(const string<platform::IOX2_MAX_FILENAME_LENGTH>& value) noexcept -> bool;
} // namespace detail

/// @brief Represents a single file name. It is not allowed to contain any path elements
///        like "./some_file" or "path/to/file". Just a plain old simple "my_file.bla".
class FileName : public SemanticString<FileName,
                                       platform::IOX2_MAX_FILENAME_LENGTH,
                                       detail::file_name_does_contain_invalid_content,
                                       detail::file_name_does_contain_invalid_characters> {
    using Parent = SemanticString<FileName,
                                  platform::IOX2_MAX_FILENAME_LENGTH,
                                  detail::file_name_does_contain_invalid_content,
                                  detail::file_name_does_contain_invalid_characters>;
    using Parent::Parent;
};
} // namespace legacy
} // namespace iox2

#endif
