// Copyright (c) 2022 - 2023 by Apex.AI Inc. All rights reserved.
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

#ifndef IOX2_BB_UTILITY_STD_STRING_SUPPORT_HPP
#define IOX2_BB_UTILITY_STD_STRING_SUPPORT_HPP

#include "iox2/bb/into.hpp"
#include "iox2/legacy/detail/convert.hpp"
#include "iox2/legacy/optional.hpp"

#include <string>

namespace iox2 {
namespace legacy {

/// @brief A specialization function of convert::from_string for std::string
/// @param v the input string in c type
/// @return an iox2::legacy::optional<Destination> where, if the return value is iox2::legacy::nullopt, it indicates a
/// failed conversion process
template <>
inline iox2::legacy::optional<std::string> convert::from_string(const char* v) noexcept {
    return iox2::legacy::optional<std::string>(v);
}

} // namespace legacy
} // namespace iox2

#endif // IOX2_BB_UTILITY_STD_STRING_SUPPORT_HPP
