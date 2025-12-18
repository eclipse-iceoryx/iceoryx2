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

#ifndef IOX2_BB_PATH_HPP
#define IOX2_BB_PATH_HPP

#include "iox2/bb/detail/attributes.hpp"
#include "iox2/bb/file_path.hpp"
#include "iox2/bb/semantic_string.hpp"

namespace iox2 {
namespace bb {
namespace detail {
inline auto path_does_contain_invalid_content(
    const bb::StaticString<platform::IOX2_MAX_PATH_LENGTH>& value IOX2_MAYBE_UNUSED) noexcept -> bool {
    return false;
}
} // namespace detail

/// @brief Represents a path to a file or a directory.
class Path : public SemanticString<Path,
                                   platform::IOX2_MAX_PATH_LENGTH,
                                   detail::path_does_contain_invalid_content,
                                   detail::file_path_does_contain_invalid_characters> {
    using Parent = SemanticString<Path,
                                  platform::IOX2_MAX_PATH_LENGTH,
                                  detail::path_does_contain_invalid_content,
                                  detail::file_path_does_contain_invalid_characters>;
    using Parent::Parent;
};
} // namespace bb
} // namespace iox2

#endif // IOX2_BB_PATH_HPP
