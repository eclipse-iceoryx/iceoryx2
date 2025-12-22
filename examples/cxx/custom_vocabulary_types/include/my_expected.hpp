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

// NOLINTBEGIN(readability-identifier-naming) STL naming required for example

// required for clang-tidy
#if __cplusplus > 202002L

#ifndef MY_EXPECTED
#define MY_EXPECTED

#include <expected>

namespace my {

template <typename T, typename E>
using expected = std::expected<T, E>;
template <typename E>
using unexpected = std::unexpected<E>;

using in_place_t = std::in_place_t;
using unexpect_t = std::unexpect_t;

constexpr in_place_t in_place = std::in_place;
constexpr unexpect_t unexpect = std::unexpect;

} // namespace my

#endif // MY_EXPECTED

#endif

//NOLINTEND(readability-identifier-naming)
