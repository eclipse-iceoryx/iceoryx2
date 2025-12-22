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

#ifndef MY_OPTIONAL
#define MY_OPTIONAL

// required for clang-tidy
#if __cplusplus >= 201703L

#include <optional>

namespace my {

template <typename T>
using optional = std::optional<T>;
using nullopt_t = std::nullopt_t;

constexpr nullopt_t nullopt = std::nullopt;

} // namespace my

#endif

#endif // MY_OPTIONAL

// NOLINTEND(readability-identifier-naming) STL naming required for example
