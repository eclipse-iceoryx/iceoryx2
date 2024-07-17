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

#ifndef IOX2_ENUM_TRANSLATION_HPP
#define IOX2_ENUM_TRANSLATION_HPP

#include "iox/assertions.hpp"
#include "iox/into.hpp"
#include "iox2/internal/iceoryx2.hpp"
#include "iox2/semantic_string.hpp"

namespace iox {
template <>
constexpr auto from<int, iox2::SemanticStringError>(const int value) noexcept -> iox2::SemanticStringError {
    const auto error = static_cast<iox2_semantic_string_error_e>(value);
    switch (error) {
    case iox2_semantic_string_error_e_INVALID_CONTENT:
        return iox2::SemanticStringError::InvalidContent;
    case iox2_semantic_string_error_e_EXCEEDS_MAXIMUM_LENGTH:
        return iox2::SemanticStringError::ExceedsMaximumLength;
    }

    IOX_UNREACHABLE();
}
} // namespace iox

#endif
