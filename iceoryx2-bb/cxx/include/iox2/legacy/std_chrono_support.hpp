// Copyright (c) 2023 by Apex.AI Inc. All rights reserved.
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

#ifndef IOX2_BB_UTILITY_STD_CHRONO_SUPPORT_HPP
#define IOX2_BB_UTILITY_STD_CHRONO_SUPPORT_HPP

#include "iox2/legacy/duration.hpp"
#include "iox2/legacy/into.hpp"

#include <chrono>

namespace iox2 {
namespace legacy {
/// @brief Construct a Duration object from std::chrono::nanoseconds
/// @param[in] value as nanoseconds
/// @attention Since negative durations are not allowed, the duration will be clamped to 0
template <>
struct FromImpl<std::chrono::nanoseconds, units::Duration> {
    static units::Duration fromImpl(const std::chrono::nanoseconds& value) noexcept;
};

/// @brief Construct a Duration object from std::chrono::microseconds
/// @param[in] value as microseconds
/// @attention Since negative durations are not allowed, the duration will be clamped to 0
template <>
struct FromImpl<std::chrono::microseconds, units::Duration> {
    static units::Duration fromImpl(const std::chrono::microseconds& value) noexcept;
};

/// @brief Construct a Duration object from std::chrono::milliseconds
/// @param[in] value as milliseconds
/// @attention Since negative durations are not allowed, the duration will be clamped to 0
template <>
struct FromImpl<std::chrono::milliseconds, units::Duration> {
    static units::Duration fromImpl(const std::chrono::milliseconds& value) noexcept;
};

/// @brief Construct a Duration object from std::chrono::seconds
/// @param[in] value as seconds
/// @attention Since negative durations are not allowed, the duration will be clamped to 0
template <>
struct FromImpl<std::chrono::seconds, units::Duration> {
    static units::Duration fromImpl(const std::chrono::seconds& value) noexcept;
};
} // namespace legacy
} // namespace iox2

#include "iox2/legacy/detail/std_chrono_support.inl"

#endif // IOX2_BB_UTILITY_STD_CHRONO_SUPPORT_HPP
