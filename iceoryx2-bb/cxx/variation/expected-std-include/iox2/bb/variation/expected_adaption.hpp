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

#ifndef IOX2_INCLUDE_GUARD_VARIATION_EXPECTED_ADAPTION_HPP
#define IOX2_INCLUDE_GUARD_VARIATION_EXPECTED_ADAPTION_HPP

// required for clang-tidy
#if __cplusplus > 202002L

#include <expected>

namespace iox2 {
namespace bb {
namespace variation {

template <typename T, typename E>
using Expected = std::expected<T, E>;
template <typename E>
using Unexpected = std::unexpected<E>;

using InPlaceT = std::in_place_t;
using UnexpectT = std::unexpect_t;

constexpr InPlaceT IN_PLACE = std::in_place;
constexpr UnexpectT UNEXPECT = std::unexpect;

} // namespace variation
} // namespace bb
} // namespace iox2

#endif

#endif // IOX2_INCLUDE_GUARD_VARIATION_EXPECTED_ADAPTION_HPP
