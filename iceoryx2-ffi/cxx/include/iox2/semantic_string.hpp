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

#ifndef IOX2_SEMANTIC_STRING_HPP
#define IOX2_SEMANTIC_STRING_HPP

#include <cstdint>

namespace iox2 {
/// @brief Failures that can occur when a [`SemanticString`] is created or modified
enum class SemanticStringError : uint8_t {
    /// @brief The modification would lead to a [`SemanticString`] with invalid content.
    InvalidContent,
    /// @brief The added content would exceed the maximum capacity of the [`SemanticString`]
    ExceedsMaximumLength
};

} // namespace iox2

#endif
