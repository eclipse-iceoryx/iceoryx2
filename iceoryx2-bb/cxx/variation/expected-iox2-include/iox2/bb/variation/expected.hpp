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

#ifndef IOX2_INCLUDE_GUARD_VARIATION_EXPECTED_HPP
#define IOX2_INCLUDE_GUARD_VARIATION_EXPECTED_HPP

#include "iox2/bb/detail/expected.hpp"

namespace iox2 {
namespace bb {

template <typename T, typename E>
using Expected = bb::detail::Expected<T, E>;
template <typename E>
using Unexpected = bb::detail::Unexpected<E>;

using InPlaceT = bb::detail::InPlaceT;
using UnexpectT = bb::detail::UnexpectT;

constexpr InPlaceT in_place = bb::detail::in_place;
constexpr UnexpectT UNEXPECT = bb::detail::UNEXPECT;

} // namespace bb
} // namespace iox2

#endif // IOX2_INCLUDE_GUARD_VARIATION_EXPECTED_HPP
