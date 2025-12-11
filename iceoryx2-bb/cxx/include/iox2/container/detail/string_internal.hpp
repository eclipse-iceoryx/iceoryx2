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

#ifndef IOX2_INCLUDE_GUARD_CONTAINER_DETAILSTRING_INTERNAL_HPP
#define IOX2_INCLUDE_GUARD_CONTAINER_DETAILSTRING_INTERNAL_HPP

#include <cstdint>
#include <cstring>
#include <type_traits>

namespace iox2 {
namespace container {
template <uint64_t>
class StaticString;

namespace detail {
template <uint64_t N>
// C array not used here, it is a type alias for easier access
// NOLINTNEXTLINE(hicpp-avoid-c-arrays, cppcoreguidelines-avoid-c-arrays, modernize-avoid-c-arrays)
using CharArray = char[N];

/// Generic empty implementation of the struct to get the size of a string
template <typename>
struct GetSize {
    static_assert(false, "GetSize is not implemented for the specified type!");
};

/// Struct to get size of iox2::container::StaticString
template <uint64_t N>
struct GetSize<StaticString<N>> {
    static auto call(const StaticString<N>& data) -> uint64_t {
        return data.size();
    }
};

/// Struct to get the size of a char array
template <uint64_t N>
// Used to acquire the size of a C array safely, strnlen only accesses N elements which is the maximum
// capacity of the array where N is a compile time constant
// NOLINTNEXTLINE(hicpp-avoid-c-arrays, cppcoreguidelines-avoid-c-arrays, modernize-avoid-c-arrays)
struct GetSize<char[N]> {
    static auto call(const CharArray<N>& data) -> uint64_t {
        return strnlen(&data[0], N);
    }
};

/// Generic empty implementation of the struct to get the data of a string
template <typename T>
struct GetData {
    static_assert(false, "GetData is not implemented for the specified type!");
};

/// Struct to get a pointer to the char array of the iox2::container::string
template <uint64_t N>
struct GetData<StaticString<N>> {
    static auto call(const StaticString<N>& data) -> const char* {
        return data.unchecked_access().c_str();
    }
};

/// Struct to get a pointer to the char array of the string literal
template <uint64_t N>
// Provides uniform and safe access (in combination with GetSize) to string like constructs
// NOLINTNEXTLINE(hicpp-avoid-c-arrays, cppcoreguidelines-avoid-c-arrays, modernize-avoid-c-arrays)
struct GetData<char[N]> {
    static auto call(const CharArray<N>& data) -> const char* {
        return &data[0];
    }
};

} // namespace detail
} // namespace container
} // namespace iox2
#endif // IOX2_INCLUDE_GUARD_CONTAINER_DETAILSTRING_INTERNAL_HPP
