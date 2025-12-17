// Copyright (c) 2019, 2021 by Robert Bosch GmbH. All rights reserved.
// Copyright (c) 2021 by Apex.AI Inc. All rights reserved.
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

#ifndef IOX2_BB_UTILITY_CONVERT_HPP
#define IOX2_BB_UTILITY_CONVERT_HPP

#include "iox2/bb/optional.hpp"
#include "iox2/legacy/posix_call.hpp"

#include <climits>
#include <cmath>
#include <cstdint>
#include <cstdlib>
#include <cstring>
#include <iostream>
#include <limits>
#include <sstream>
#include <string>

namespace iox2 {
namespace legacy {
/// @brief Collection of static methods for conversion from and to string.
/// @code
///     std::string number      = iox2::legacy::convert::toString(123);
///     std::string someClass   = iox2::legacy::convert::toString(someToStringConvertableObject);
///
///     int i;
///     unsigned int a;
///     if ( iox2::legacy::convert::from_string("123", i) ) {}  // will succeed
///     if ( iox2::legacy::convert::from_string("-123", a) ) {} // will fail since -123 is not unsigned
/// @endcode
/// @todo iox-#260 Refactor 'convert' so that one can use 'into' to directly to convert numbers to strings:
/// 'ClassExpectingAString(iox2::bb::into<iox2::bb::StaticString<100>>(42)'
class convert {
  public:
    enum class NumberType : uint8_t {
        INTEGER,
        UNSIGNED_INTEGER,
        FLOAT
    };

    static constexpr int32_t STRTOULL_BASE { 10 };
    static constexpr int32_t STRTOUL_BASE { 10 };
    static constexpr int32_t STRTOLL_BASE { 10 };
    static constexpr int32_t STRTOL_BASE { 10 };

    static constexpr uint32_t FLOAT_SIGNALING_NAN_MASK { static_cast<uint32_t>(1) << static_cast<uint32_t>(22) };
    static constexpr uint64_t DOUBLE_SIGNALING_NAN_MASK { static_cast<uint64_t>(1) << static_cast<uint64_t>(51) };

    /// @brief Converts every type which is either a pod (plain old data) type or is convertable
    ///         to a string (this means that the operator std::string() is defined)
    /// @param Source type of the value which should be converted to a string
    /// @param[in] t value which should be converted to a string
    /// @return string representation of t
    template <typename Source>
    static typename std::enable_if<!std::is_convertible<Source, std::string>::value, std::string>::type
    toString(const Source& t) noexcept;

    /// @brief Converts every type which is either a pod (plain old data) type or is convertable
    ///         to a string (this means that the operator std::string() is defined)
    /// @param Source type of the value which should be converted to a string
    /// @param[in] t value which should be converted to a string
    /// @return string representation of t
    template <typename Source>
    static typename std::enable_if<std::is_convertible<Source, std::string>::value, std::string>::type
    toString(const Source& t) noexcept;

    /// @brief  convert the input based on the 'TargetType', allowing only numeric types as valid destination
    /// types
    /// @note   for the 'TargetType' equal to 'std::string,' please include 'iox/std_string_support.hpp'
    /// @tparam TargetType the desired target type for converting text
    /// @param v the input string in c type
    /// @return an iox2::bb::Optional<TargetType> where, if the return value is iox2::bb::NULLOPT, it indicates
    /// a failed conversion process
    template <typename TargetType>
    static bb::Optional<TargetType> from_string(const char* v) noexcept;

  private:
    template <typename TargetType, typename CallType>
    static bb::Optional<TargetType> evaluate_return_value(CallType& call, const char* end_ptr, const char* v) noexcept;

    template <typename TargetType, typename SourceType>
    static bool check_edge_case(decltype(errno) errno_cache,
                                const char* end_ptr,
                                const char* v,
                                const SourceType& source_val) noexcept;

    template <typename SourceType>
    static bool is_valid_input(const char* end_ptr, const char* v, const SourceType& source_val) noexcept;

    template <typename TargetType, typename SourceType>
    static bool is_within_range(const SourceType& source_val) noexcept;

    static bool is_valid_errno(decltype(errno) errno_cache, const char* v) noexcept;
    static bool start_with_neg_sign(const char* v) noexcept;
};

} // namespace legacy
} // namespace iox2

#include "iox2/legacy/detail/convert.inl"

#endif // IOX2_BB_UTILITY_CONVERT_HPP
