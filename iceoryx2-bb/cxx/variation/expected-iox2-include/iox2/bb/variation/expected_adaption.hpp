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

#include "iox2/bb/stl/expected.hpp"

namespace iox2 {
namespace bb {
namespace variation {

template <typename T, typename E>
using Expected = iox2::bb::stl::Expected<T, E>;
template <typename E>
using Unexpected = iox2::bb::stl::Unexpected<E>;

using InPlaceT = iox2::bb::stl::InPlaceT;
using UnexpectT = iox2::bb::stl::UnexpectT;

constexpr InPlaceT IN_PLACE = iox2::bb::stl::IN_PLACE;
constexpr UnexpectT UNEXPECT = iox2::bb::stl::UNEXPECT;

} // namespace variation
} // namespace bb
} // namespace iox2

#endif // IOX2_INCLUDE_GUARD_VARIATION_EXPECTED_ADAPTION_HPP
