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

#ifndef IOX2_INCLUDE_GUARD_VARIATION_OPTIONAL_ADAPTION_HPP
#define IOX2_INCLUDE_GUARD_VARIATION_OPTIONAL_ADAPTION_HPP

#include "iox2/bb/stl/optional.hpp"

namespace iox2 {
namespace bb {
namespace variation {

template <typename T>
using Optional = iox2::bb::stl::Optional<T>;
using NulloptT = iox2::bb::stl::NulloptT;

constexpr NulloptT NULLOPT = iox2::bb::stl::NULLOPT;

} // namespace variation
} // namespace bb
} // namespace iox2

#endif // IOX2_INCLUDE_GUARD_VARIATION_OPTIONAL_ADAPTION_HPP
