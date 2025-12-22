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

// required for clang-tidy
#if __cplusplus > 202002L

#ifndef MY_EXPECTED_FOR_ICEORYX2
#define MY_EXPECTED_FOR_ICEORYX2

#include "my_expected.hpp"

namespace iox2 {
namespace bb {
namespace variation {

template <typename T, typename E>
using Expected = my::expected<T, E>;
template <typename E>
using Unexpected = my::unexpected<E>;

using InPlaceT = my::in_place_t;
using UnexpectT = my::unexpect_t;

constexpr InPlaceT IN_PLACE = my::in_place;
constexpr UnexpectT UNEXPECT = my::unexpect;

} // namespace variation
} // namespace bb
} // namespace iox2

#endif // MY_EXPECTED_FOR_ICEORYX2

#endif
