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

#ifndef IOX2_INCLUDE_GUARD_BB_DETAIL_STRING_INTERNAL_HPP
#define IOX2_INCLUDE_GUARD_BB_DETAIL_STRING_INTERNAL_HPP

#include <cstdint>
#include <cstring>

namespace iox2 {
namespace bb {
template <uint64_t>
class StaticString;

namespace detail {
template <uint64_t N>
// C array not used here, it is a type alias for easier access
// NOLINTNEXTLINE(hicpp-avoid-c-arrays, cppcoreguidelines-avoid-c-arrays, modernize-avoid-c-arrays)
using CharArray = char[N];

template <uint64_t N>
auto get_size(const StaticString<N>& data) -> uint64_t {
    return data.size();
}

template <uint64_t N>
auto get_size(const CharArray<N>& data) -> uint64_t {
    return strnlen(&data[0], N);
}

template <uint64_t N>
auto get_data(const StaticString<N>& data) -> const char* {
    return data.unchecked_access().c_str();
}

template <uint64_t N>
auto get_data(const CharArray<N>& data) -> const char* {
    return &data[0];
}
} // namespace detail
} // namespace bb
} // namespace iox2

#endif // IOX2_INCLUDE_GUARD_BB_DETAIL_STRING_INTERNAL_HPP
