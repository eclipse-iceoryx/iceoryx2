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
#if __cplusplus >= 201703L

#ifndef MY_OPTIONAL_FOR_ICEORYX2
#define MY_OPTIONAL_FOR_ICEORYX2

#include "my_optional.hpp"

namespace iox2 {
namespace bb {
namespace variation {

template <typename T>
using Optional = my::optional<T>;
using NulloptT = my::nullopt_t;

constexpr NulloptT NULLOPT = my::nullopt;

} // namespace variation
} // namespace bb
} // namespace iox2

#endif // MY_OPTIONAL_FOR_ICEORYX2

#endif
