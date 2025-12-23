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

#ifndef IOX2_INCLUDE_GUARD_BB_EXPECTED_HPP
#define IOX2_INCLUDE_GUARD_BB_EXPECTED_HPP

#include "iox2/bb/variation/expected_adaption.hpp"

namespace iox2 {
namespace bb {

template <typename T, typename E>
using Expected = iox2::bb::variation::Expected<T, E>;
template <typename E>
using Unexpected = iox2::bb::variation::Unexpected<E>;

using InPlaceT = iox2::bb::variation::InPlaceT;
using UnexpectT = iox2::bb::variation::UnexpectT;

constexpr InPlaceT IN_PLACE = iox2::bb::variation::IN_PLACE;
constexpr UnexpectT UNEXPECT = iox2::bb::variation::UNEXPECT;

template <typename E>
constexpr auto err(const E& error) -> Unexpected<E> {
    return Unexpected<E>(error);
}

template <typename E, std::enable_if_t<!std::is_lvalue_reference<E>::value, bool> = true>
constexpr auto err(E&& error) -> Unexpected<E> {
    return Unexpected<E>(std::forward<E>(error));
}

} // namespace bb
} // namespace iox2

#endif // IOX2_INCLUDE_GUARD_BB_EXPECTED_HPP
