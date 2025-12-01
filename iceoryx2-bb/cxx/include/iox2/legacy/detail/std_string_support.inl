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

#ifndef IOX2_BB_UTILITY_STD_STRING_SUPPORT_INL
#define IOX2_BB_UTILITY_STD_STRING_SUPPORT_INL

#include "iox2/legacy/std_string_support.hpp"

namespace iox2 {
namespace legacy {
template <uint64_t N>
inline std::string FromImpl<string<N>, std::string>::fromImpl(const string<N>& value) noexcept {
    return std::string(value.c_str(), static_cast<size_t>(value.size()));
}

template <uint64_t N>
inline string<N> FromImpl<std::string, string<N>>::fromImpl(const std::string&) noexcept {
    static_assert(always_false_v<std::string> && always_false_v<string<N>>, "\n \
        The conversion from 'std::string' to 'iox2::legacy::string<N>' is potentially lossy!\n \
        This happens when the size of source string exceeds the capacity of the destination string!\n \
        Please use either: \n \
          - 'iox2::legacy::into<iox2::legacy::optional<iox2::legacy::string<N>>>' which returns a 'iox2::legacy::optional<iox2::legacy::string<N>>'\n \
            with a 'nullopt' if the size of the source string exceeds the capacity of the destination string\n \
          - 'iox2::legacy::into<iox2::legacy::lossy<iox2::legacy::string<N>>>' which returns a 'iox2::legacy::string<N>' and truncates the\n \
            source string if its size exceeds the capacity of the destination string");
}

template <uint64_t N>
inline optional<string<N>> FromImpl<std::string, optional<string<N>>>::fromImpl(const std::string& value) noexcept {
    const auto stringLength = value.size();
    if (stringLength <= N) {
        return string<N>(TruncateToCapacity, value.c_str(), stringLength);
    }
    return nullopt;
}

template <uint64_t N>
inline string<N> FromImpl<std::string, lossy<string<N>>>::fromImpl(const std::string& value) noexcept {
    return string<N>(TruncateToCapacity, value.c_str(), value.size());
}

// AXIVION Next Construct AutosarC++19_03-M5.17.1: This is not used as shift operator but as stream operator and does
// not require to implement '<<='
template <uint64_t Capacity>
inline std::ostream& operator<<(std::ostream& stream, const string<Capacity>& str) noexcept {
    stream << str.c_str();
    return stream;
}
} // namespace legacy
} // namespace iox2

#endif // IOX2_BB_UTILITY_STD_STRING_SUPPORT_INL
