// Copyright (c) 2022 - 2023 by Apex.AI Inc. All rights reserved.
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

#ifndef IOX2_BB_UTILITY_STD_STRING_SUPPORT_HPP
#define IOX2_BB_UTILITY_STD_STRING_SUPPORT_HPP

#include "iox2/bb/into.hpp"
#include "iox2/legacy/detail/convert.hpp"
#include "iox2/legacy/optional.hpp"
#include "iox2/legacy/string.hpp"

#include <cstdint>
#include <ostream>
#include <string>

namespace iox2 {

namespace bb {
template <uint64_t N>
struct FromImpl<legacy::string<N>, std::string> {
    static std::string fromImpl(const legacy::string<N>& value) noexcept;
};

template <uint64_t N>
struct FromImpl<std::string, legacy::string<N>> {
    static legacy::string<N> fromImpl(const std::string& value) noexcept;
};

template <uint64_t N>
struct FromImpl<std::string, legacy::optional<legacy::string<N>>> {
    static legacy::optional<legacy::string<N>> fromImpl(const std::string& value) noexcept;
};

template <uint64_t N>
struct FromImpl<std::string, bb::lossy<legacy::string<N>>> {
    static legacy::string<N> fromImpl(const std::string& value) noexcept;
};
} // namespace bb

namespace legacy {

template <>
struct is_custom_string<std::string> : public std::true_type { };

namespace internal {
/// @brief struct to get a pointer to the char array of the std::string
template <>
struct GetData<std::string> {
    static const char* call(const std::string& data) noexcept {
        return data.data();
    }
};

/// @brief struct to get size of a std::string
template <>
struct GetSize<std::string> {
    static uint64_t call(const std::string& data) noexcept {
        return data.size();
    }
};
} // namespace internal

/// @brief outputs the fixed string on stream
///
/// @param [in] stream is the output stream
/// @param [in] str is the fixed string
///
/// @return the stream output of the fixed string
template <uint64_t Capacity>
std::ostream& operator<<(std::ostream& stream, const string<Capacity>& str) noexcept;

/// @brief A specialization function of convert::from_string for std::string
/// @param v the input string in c type
/// @return an iox2::legacy::optional<Destination> where, if the return value is iox2::legacy::nullopt, it indicates a
/// failed conversion process
template <>
inline iox2::legacy::optional<std::string> convert::from_string(const char* v) noexcept {
    return iox2::legacy::optional<std::string>(v);
}

} // namespace legacy
} // namespace iox2

#include "iox2/legacy/detail/std_string_support.inl"

#endif // IOX2_BB_UTILITY_STD_STRING_SUPPORT_HPP
