// Copyright (c) 2022 by Apex.AI Inc. All rights reserved.
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

#ifndef IOX2_BB_VOCABULARY_STRING_TYPE_TRAITS_HPP
#define IOX2_BB_VOCABULARY_STRING_TYPE_TRAITS_HPP

#include <cstdint>

#include "iox2/legacy/type_traits.hpp"

namespace iox2 {
namespace legacy {
template <uint64_t Capacity>
class string;

/// @brief struct to check whether an argument is a iox2::legacy::string
template <typename T>
struct is_iox_string : std::false_type { };

template <uint64_t N>
struct is_iox_string<::iox2::legacy::string<N>> : std::true_type { };

} // namespace legacy
} // namespace iox2

#endif // IOX2_BB_VOCABULARY_STRING_TYPE_TRAITS_HPP
