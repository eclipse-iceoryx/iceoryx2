// Copyright (c) 2019, 2021 by Robert Bosch GmbH. All rights reserved.
// Copyright (c) 2021 - 2022 by Apex.AI Inc. All rights reserved.
// Copyright (c) 2022 by NXP. All rights reserved.
// Copyright (c) 2023 by Dennis Liu. All rights reserved.
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

#ifndef IOX2_BB_UTILITY_CONVERT_INL
#define IOX2_BB_UTILITY_CONVERT_INL

#include "iox2/legacy/detail/convert.hpp"
#include "iox2/legacy/logging.hpp"

namespace iox2 {
namespace legacy {
///@brief specialization for  uint8_t and int8_t is required  since uint8_t is unsigned char and int8_t is signed char
/// and stringstream will not convert these to string as it is already a character.
template <>
inline typename std::enable_if<!std::is_convertible<uint8_t, std::string>::value, std::string>::type
convert::toString(const uint8_t& t) noexcept {
    return toString(static_cast<uint16_t>(t));
}

template <>
inline typename std::enable_if<!std::is_convertible<int8_t, std::string>::value, std::string>::type
convert::toString(const int8_t& t) noexcept {
    return toString(static_cast<int16_t>(t));
}


template <typename Source>
inline typename std::enable_if<!std::is_convertible<Source, std::string>::value, std::string>::type
convert::toString(const Source& t) noexcept {
    std::stringstream ss;
    ss << t;
    return ss.str();
}

template <typename Source>
inline typename std::enable_if<std::is_convertible<Source, std::string>::value, std::string>::type
convert::toString(const Source& t) noexcept {
    return t;
}

template <typename TargetType>
inline bb::Optional<TargetType> convert::from_string(const char* v IOX2_MAYBE_UNUSED) noexcept {
    static_assert(always_false_v<TargetType>,
                  "For a conversion to 'std::string' please include 'iox/std_string_support.hpp'!\nConversion not "
                  "supported!");
}

template <>
inline bb::Optional<char> convert::from_string<char>(const char* v) noexcept {
    if (strlen(v) != 1U) {
        IOX2_LOG(Debug, v << " is not a char");
        return bb::NULLOPT;
    }

    // NOLINTJUSTIFICATION encapsulated in abstraction
    // NOLINTNEXTLINE(cppcoreguidelines-pro-bounds-pointer-arithmetic)
    return v[0];
}

template <>
inline bb::Optional<bool> convert::from_string<bool>(const char* v) noexcept {
    char* end_ptr = nullptr;

    if (start_with_neg_sign(v)) {
        return bb::NULLOPT;
    }

    auto call = IOX2_POSIX_CALL(strtoul)(v, &end_ptr, STRTOUL_BASE)
                    .failureReturnValue(ULONG_MAX)
                    .ignoreErrnos(0, EINVAL, ERANGE)
                    .evaluate();

    // we assume that in the IOX2_POSIX_CALL procedure, no other POSIX call will change errno,
    // except for the target function 'f'.
    return evaluate_return_value<bool>(call, end_ptr, v);
}

template <>
inline bb::Optional<float> convert::from_string<float>(const char* v) noexcept {
    char* end_ptr = nullptr;

    auto call = IOX2_POSIX_CALL(strtof)(v, &end_ptr)
                    .failureReturnValue(HUGE_VALF, -HUGE_VALF)
                    .ignoreErrnos(0, EINVAL, ERANGE)
                    .evaluate();

    return evaluate_return_value<float>(call, end_ptr, v);
}

template <>
inline bb::Optional<double> convert::from_string<double>(const char* v) noexcept {
    char* end_ptr = nullptr;

    auto call = IOX2_POSIX_CALL(strtod)(v, &end_ptr)
                    .failureReturnValue(HUGE_VAL, -HUGE_VAL)
                    .ignoreErrnos(0, EINVAL, ERANGE)
                    .evaluate();

    return evaluate_return_value<double>(call, end_ptr, v);
}

template <>
inline bb::Optional<long double> convert::from_string<long double>(const char* v) noexcept {
    char* end_ptr = nullptr;

    auto call = IOX2_POSIX_CALL(strtold)(v, &end_ptr)
                    .failureReturnValue(HUGE_VALL, -HUGE_VALL)
                    .ignoreErrnos(0, EINVAL, ERANGE)
                    .evaluate();

    return evaluate_return_value<long double>(call, end_ptr, v);
}

template <>
inline bb::Optional<unsigned long long> convert::from_string<unsigned long long>(const char* v) noexcept {
    char* end_ptr = nullptr;

    if (start_with_neg_sign(v)) {
        return bb::NULLOPT;
    }

    auto call = IOX2_POSIX_CALL(strtoull)(v, &end_ptr, STRTOULL_BASE)
                    .failureReturnValue(ULLONG_MAX)
                    .ignoreErrnos(0, EINVAL, ERANGE)
                    .evaluate();

    return evaluate_return_value<unsigned long long>(call, end_ptr, v);
}

template <>
inline bb::Optional<unsigned long> convert::from_string<unsigned long>(const char* v) noexcept {
    char* end_ptr = nullptr;

    if (start_with_neg_sign(v)) {
        return bb::NULLOPT;
    }

    auto call = IOX2_POSIX_CALL(strtoul)(v, &end_ptr, STRTOUL_BASE)
                    .failureReturnValue(ULONG_MAX)
                    .ignoreErrnos(0, EINVAL, ERANGE)
                    .evaluate();

    return evaluate_return_value<unsigned long>(call, end_ptr, v);
}

template <>
inline bb::Optional<unsigned int> convert::from_string<unsigned int>(const char* v) noexcept {
    char* end_ptr = nullptr;

    if (start_with_neg_sign(v)) {
        return bb::NULLOPT;
    }

    // use alwaysSuccess for the conversion edge cases in 32-bit system?
    auto call = IOX2_POSIX_CALL(strtoul)(v, &end_ptr, STRTOUL_BASE)
                    .failureReturnValue(ULONG_MAX)
                    .ignoreErrnos(0, EINVAL, ERANGE)
                    .evaluate();

    return evaluate_return_value<unsigned int>(call, end_ptr, v);
}

template <>
inline bb::Optional<unsigned short> convert::from_string<unsigned short>(const char* v) noexcept {
    char* end_ptr = nullptr;

    if (start_with_neg_sign(v)) {
        return bb::NULLOPT;
    }

    auto call = IOX2_POSIX_CALL(strtoul)(v, &end_ptr, STRTOUL_BASE)
                    .failureReturnValue(ULONG_MAX)
                    .ignoreErrnos(0, EINVAL, ERANGE)
                    .evaluate();

    return evaluate_return_value<unsigned short>(call, end_ptr, v);
}

template <>
inline bb::Optional<unsigned char> convert::from_string<unsigned char>(const char* v) noexcept {
    char* end_ptr = nullptr;

    if (start_with_neg_sign(v)) {
        return bb::NULLOPT;
    }

    auto call = IOX2_POSIX_CALL(strtoul)(v, &end_ptr, STRTOULL_BASE)
                    .failureReturnValue(ULONG_MAX)
                    .ignoreErrnos(0, EINVAL, ERANGE)
                    .evaluate();

    return evaluate_return_value<unsigned char>(call, end_ptr, v);
}

template <>
inline bb::Optional<long long> convert::from_string<long long>(const char* v) noexcept {
    char* end_ptr = nullptr;

    auto call = IOX2_POSIX_CALL(strtoll)(v, &end_ptr, STRTOLL_BASE)
                    .failureReturnValue(LLONG_MAX, LLONG_MIN)
                    .ignoreErrnos(0, EINVAL, ERANGE)
                    .evaluate();

    return evaluate_return_value<long long>(call, end_ptr, v);
}

template <>
inline bb::Optional<long> convert::from_string<long>(const char* v) noexcept {
    char* end_ptr = nullptr;

    auto call = IOX2_POSIX_CALL(strtol)(v, &end_ptr, STRTOL_BASE)
                    .failureReturnValue(LONG_MAX, LONG_MIN)
                    .ignoreErrnos(0, EINVAL, ERANGE)
                    .evaluate();

    return evaluate_return_value<long>(call, end_ptr, v);
}

template <>
inline bb::Optional<int> convert::from_string<int>(const char* v) noexcept {
    char* end_ptr = nullptr;

    // use alwaysSuccess for the conversion edge cases in 32-bit system?
    auto call = IOX2_POSIX_CALL(strtol)(v, &end_ptr, STRTOL_BASE)
                    .failureReturnValue(LONG_MAX, LONG_MIN)
                    .ignoreErrnos(0, EINVAL, ERANGE)
                    .evaluate();

    return evaluate_return_value<int>(call, end_ptr, v);
}

template <>
inline bb::Optional<short> convert::from_string<short>(const char* v) noexcept {
    char* end_ptr = nullptr;

    auto call = IOX2_POSIX_CALL(strtol)(v, &end_ptr, STRTOL_BASE)
                    .failureReturnValue(LONG_MAX, LONG_MIN)
                    .ignoreErrnos(0, EINVAL, ERANGE)
                    .evaluate();

    return evaluate_return_value<short>(call, end_ptr, v);
}

template <>
inline bb::Optional<signed char> convert::from_string<signed char>(const char* v) noexcept {
    char* end_ptr = nullptr;

    auto call = IOX2_POSIX_CALL(strtol)(v, &end_ptr, STRTOL_BASE)
                    .failureReturnValue(LONG_MAX, LONG_MIN)
                    .ignoreErrnos(0, EINVAL, ERANGE)
                    .evaluate();

    return evaluate_return_value<signed char>(call, end_ptr, v);
}

template <typename TargetType, typename SourceType>
inline bool convert::check_edge_case(decltype(errno) errno_cache,
                                     const char* end_ptr,
                                     const char* v,
                                     const SourceType& source_val) noexcept {
    return is_valid_input(end_ptr, v, source_val) && is_valid_errno(errno_cache, v)
           && is_within_range<TargetType>(source_val);
}

template <typename TargetType, typename CallType>
inline bb::Optional<TargetType>
convert::evaluate_return_value(CallType& call, const char* end_ptr, const char* v) noexcept {
    if (call.has_error()) {
        return bb::NULLOPT;
    }

    if (!check_edge_case<TargetType>(call->errnum, end_ptr, v, call->value)) {
        return bb::NULLOPT;
    }

    return static_cast<TargetType>(call->value);
}

template <typename SourceType>
inline bool convert::is_valid_input(const char* end_ptr, const char* v, const SourceType& source_val) noexcept {
    // invalid string
    if (v == end_ptr && source_val == 0) {
        IOX2_LOG(Debug, "invalid input");
        return false;
    }

    // end_ptr is not '\0' which means conversion failure at end_ptr
    if (end_ptr != nullptr && v != end_ptr && *end_ptr != '\0') {
        IOX2_LOG(Debug, "conversion failed at " << end_ptr - v << " : " << *end_ptr);
        return false;
    }

    return true;
}

template <typename TargetType, typename SourceType>
inline bool convert::is_within_range(const SourceType& source_val) noexcept {
#if __cplusplus >= 201703L
    if constexpr (std::is_arithmetic_v<TargetType> == false)
#else
    if (std::is_arithmetic<TargetType>::value == false)
#endif
    {
        return true;
    }

#if __cplusplus >= 201703L
    if constexpr (std::is_floating_point_v<SourceType>) {
        auto source_val_fp = source_val;
#else
    if (std::is_floating_point<SourceType>::value) {
        auto source_val_fp = static_cast<double>(source_val);
#endif
        // special cases for floating point
        // can be nan or inf
        if (std::isnan(source_val) || std::isinf(source_val)) {
            return true;
        }
        // should be normal or zero
        if (!std::isnormal(source_val) && (source_val_fp != 0.0F)) {
            return false;
        }
    }
    // out of range (upper bound)
    if (source_val > std::numeric_limits<TargetType>::max()) {
        IOX2_LOG(Debug,
                 source_val << " is out of range (upper bound), should be less than "
                            << std::numeric_limits<TargetType>::max());
        return false;
    }
    // out of range (lower bound)
    if (source_val < std::numeric_limits<TargetType>::lowest()) {
        IOX2_LOG(Debug,
                 source_val << " is out of range (lower bound), should be larger than "
                            << std::numeric_limits<TargetType>::lowest());
        return false;
    }
    return true;
}

inline bool convert::start_with_neg_sign(const char* v) noexcept {
    if (v == nullptr) {
        return false;
    }

    // remove space
    while (*v != '\0' && (isspace((unsigned char) *v) != 0)) {
        // NOLINTNEXTLINE(cppcoreguidelines-pro-bounds-pointer-arithmetic)
        ++v;
    }

    return (*v == '-');
}

inline bool convert::is_valid_errno(decltype(errno) errno_cache, const char* v) noexcept {
    if (errno_cache == ERANGE) {
        IOX2_LOG(Debug, "ERANGE triggered during conversion of string: '" << v << "'");
        return false;
    }

    if (errno_cache == EINVAL) {
        IOX2_LOG(Debug, "EINVAL triggered during conversion of string: " << v);
        return false;
    }

    if (errno_cache != 0) {
        IOX2_LOG(Debug, "Unexpected errno: " << errno_cache << ". The input string is: " << v);
        return false;
    }

    return true;
}

} // namespace legacy
} // namespace iox2

#endif // IOX2_BB_UTILITY_CONVERT_INL
