// Copyright (c) 2019 by Robert Bosch GmbH. All rights reserved.
// Copyright (c) 2021 - 2022 by Apex.AI Inc. All rights reserved.
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

#ifndef IOX2_BB_DURATION_HPP
#define IOX2_BB_DURATION_HPP

#include "iox2/bb/detail/attributes.hpp"
#include "iox2/legacy/log/logstream.hpp"
#include "iox2/legacy/type_traits.hpp"

#include <cmath>

namespace iox2 {
namespace bb {
class Duration;

namespace duration_literals {
/// @brief Constructs a new Duration object from nanoseconds
// AXIVION Next Line AutosarC++19_03-A3.9.1 : Use of unsigned long long int in user-defined literals is enforced by the standard
constexpr auto operator""_ns(unsigned long long int value) noexcept -> Duration;

/// @brief Constructs a new Duration object from microseconds
// AXIVION Next Line AutosarC++19_03-A3.9.1 : Use of unsigned long long int in user-defined literals is enforced by the standard
constexpr auto operator""_us(unsigned long long int value) noexcept -> Duration;

/// @brief Constructs a new Duration object from milliseconds
// AXIVION Next Line AutosarC++19_03-A3.9.1 : Use of unsigned long long int in user-defined literals is enforced by the standard
constexpr auto operator""_ms(unsigned long long int value) noexcept -> Duration;

/// @brief Constructs a new Duration object from seconds
// AXIVION Next Line AutosarC++19_03-A3.9.1 : Use of unsigned long long int in user-defined literals is enforced by the standard
constexpr auto operator""_s(unsigned long long int value) noexcept -> Duration;

/// @brief Constructs a new Duration object from minutes
// AXIVION Next Line AutosarC++19_03-A3.9.1 : Use of unsigned long long int in user-defined literals is enforced by the standard
constexpr auto operator""_m(unsigned long long int value) noexcept -> Duration;

/// @brief Constructs a new Duration object from hours
// AXIVION Next Line AutosarC++19_03-A3.9.1 : Use of unsigned long long int in user-defined literals is enforced by the standard
constexpr auto operator""_h(unsigned long long int value) noexcept -> Duration;

/// @brief Constructs a new Duration object from days
// AXIVION Next Line AutosarC++19_03-A3.9.1 : Use of unsigned long long int in user-defined literals is enforced by the standard
constexpr auto operator""_d(unsigned long long int value) noexcept -> Duration;
} // namespace duration_literals

/// @code
///   #include <iostream>
///   // ...
///   using namespace units;
///   using namespace units::duration_literals;
///   auto someDays = 2 * 7_d + 5_ns;
///   auto someSeconds = 42_s + 500_ms;
///   IOX2_LOG(Info, someDays);
///   IOX2_LOG(Info, someDays.nanoSeconds<uint64_t>() << " ns");
///   IOX2_LOG(Info, someSeconds.milliSeconds<int64_t>() << " ms");
/// @endcode
class Duration {
  protected:
    using SecondsT = uint64_t;
    using NanosecondsT = uint32_t;

  private:
    SecondsT m_seconds { 0U };
    NanosecondsT m_nanoseconds { 0U };

  public:
    // BEGIN CREATION FROM STATIC FUNCTIONS

    /// @brief Constructs a new Duration object from nanoseconds
    /// @tparam T is an integer type for the value
    /// @param[in] value as nanoseconds
    /// @return a new Duration object
    /// @attention Since negative durations are not allowed, the duration will be clamped to 0
    template <typename T>
    static constexpr auto from_nanos(T value) noexcept -> Duration;

    /// @brief Constructs a new Duration object from microseconds
    /// @tparam T is an integer type for the value
    /// @param[in] value as microseconds
    /// @return a new Duration object
    /// @attention Since negative durations are not allowed, the duration will be clamped to 0
    template <typename T>
    static constexpr auto from_micros(T value) noexcept -> Duration;

    /// @brief Constructs a new Duration object from milliseconds
    /// @tparam T is an integer type for the value
    /// @param[in] value as milliseconds
    /// @return a new Duration object
    /// @attention Since negative durations are not allowed, the duration will be clamped to 0
    template <typename T>
    static constexpr auto from_millis(T value) noexcept -> Duration;

    /// @brief Constructs a new Duration object from seconds
    /// @tparam T is an integer type for the value
    /// @param[in] value as seconds
    /// @return a new Duration object
    /// @attention Since negative durations are not allowed, the duration will be clamped to 0
    template <typename T>
    static constexpr auto from_secs(T value) noexcept -> Duration;

    /// @brief Constructs a new Duration object from minutes
    /// @tparam T is an integer type for the value
    /// @param[in] value as minutes
    /// @return a new Duration object
    /// @attention Since negative durations are not allowed, the duration will be clamped to 0
    template <typename T>
    static constexpr auto from_mins(T value) noexcept -> Duration;

    /// @brief Constructs a new Duration object from hours
    /// @tparam T is an integer type for the value
    /// @param[in] value as hours
    /// @return a new Duration object
    /// @attention Since negative durations are not allowed, the duration will be clamped to 0
    template <typename T>
    static constexpr auto from_hours(T value) noexcept -> Duration;

    /// @brief Constructs a new Duration object from days
    /// @tparam T is an integer type for the value
    /// @param[in] value as days
    /// @return a new Duration object
    /// @attention Since negative durations are not allowed, the duration will be clamped to 0
    template <typename T>
    static constexpr auto from_days(T value) noexcept -> Duration;

    /// @brief Constructs a new Duration object of maximum allowed length. Useful for functions which should have an
    /// "infinite" timeout.
    static constexpr auto max() noexcept -> Duration;

    /// @brief Constructs a new Duration object with a duration of zero
    static constexpr auto zero() noexcept -> Duration;
    // END CREATION FROM STATIC FUNCTIONS

    // BEGIN COMPARISON
    // AXIVION DISABLE STYLE AutosarC++19_03-A8.4.7 : Each argument is larger than two words
    friend constexpr auto operator==(const Duration& lhs, const Duration& rhs) noexcept -> bool;
    friend constexpr auto operator!=(const Duration& lhs, const Duration& rhs) noexcept -> bool;
    friend constexpr auto operator<(const Duration& lhs, const Duration& rhs) noexcept -> bool;
    friend constexpr auto operator<=(const Duration& lhs, const Duration& rhs) noexcept -> bool;
    friend constexpr auto operator>(const Duration& lhs, const Duration& rhs) noexcept -> bool;
    friend constexpr auto operator>=(const Duration& lhs, const Duration& rhs) noexcept -> bool;
    // AXIVION ENABLE STYLE AutosarC++19_03-A8.4.7
    // END COMPARISON

    // BEGIN ARITHMETIC

    /// @brief Creates Duration object by addition. On overflow duration saturates to Duration::max().
    /// @param[in] rhs is the second summand
    /// @return a new Duration object
    // AXIVION Next Line AutosarC++19_03-A8.4.7 : Argument is larger than two words
    constexpr auto operator+(const Duration& rhs) const noexcept -> Duration;

    /// @brief Adds a Duration to itself. On overflow duration saturates to Duration::max().
    /// @param[in] rhs is the second summand
    /// @return a reference to itself
    // AXIVION Next Line AutosarC++19_03-A8.4.7 : Argument is larger than two words
    constexpr auto operator+=(const Duration& rhs) noexcept -> Duration&;

    /// @brief Creates Duration object by subtraction. On underflow duration saturates to Duration::zero().
    /// @param[in] rhs is the subtrahend
    /// @return a new Duration object
    /// @attention Since negative durations are not allowed, the duration will be clamped to 0
    // AXIVION Next Line AutosarC++19_03-A8.4.7 : Each argument is larger than two words
    constexpr auto operator-(const Duration& rhs) const noexcept -> Duration;

    /// @brief Subtracts a Duration from itself. On underflow duration saturates to Duration::zero().
    /// @param[in] rhs is the subtrahend
    /// @return a reference to itself
    /// @attention Since negative durations are not allowed, the duration will be clamped to 0
    // AXIVION Next Line AutosarC++19_03-A8.4.7 : Argument is larger than two words
    constexpr auto operator-=(const Duration& rhs) noexcept -> Duration&;

    /// @brief Creates Duration object by multiplication.
    /// @tparam T is an arithmetic type for the multiplicator
    /// @param[in] rhs is the multiplicator
    /// @return a new Duration object
    /// @attention Since negative durations are not allowed, the duration will be clamped to 0
    /// @note A duration of 0 will always result in 0, no matter if multiplied with NaN or +Inf
    /// @note There is no explicit division operator! This can be achieved by multiplication with the inverse of the
    /// divisor.
    /// @note Multiplication of a non-zero duration with NaN and +Inf results in a saturated max duration
    template <typename T>
    constexpr auto operator*(const T& rhs) const noexcept -> Duration;

    /// @brief Multiplies a Duration with an arithmetic type and assigns the result to itself.
    /// @tparam T is an arithmetic type for the multiplicator
    /// @param[in] rhs is the multiplicator
    /// @return a reference to itself
    /// @attention Since negative durations are not allowed, the duration will be clamped to 0
    /// @note A duration of 0 will always result in 0, no matter if multiplied with NaN or +Inf
    /// @note There is no explicit division operator! This can be achieved by multiplication with the inverse of the
    /// divisor.
    /// @note Multiplication of a non-zero duration with NaN and +Inf results in a saturated max duration
    template <typename T>
    constexpr auto operator*=(const T& rhs) noexcept -> Duration&;

    // END ARITHMETIC

    // BEGIN CONVERSION

    /// @brief returns the duration in nanoseconds
    /// @note If the duration in nanoseconds is larger than an uint64_t can represent, it will be clamped to the
    /// uint64_t max value.
    constexpr auto as_nanos() const noexcept -> uint64_t;

    /// @brief returns the duration in microseconds
    /// @note If the duration in microseconds is larger than an uint64_t can represent, it will be clamped to the
    /// uint64_t max value.
    /// @note The remaining nanoseconds are truncated, similar to the casting behavior of a float to an int.
    constexpr auto as_micros() const noexcept -> uint64_t;

    /// @brief returns the duration in milliseconds
    /// @note If the duration in milliseconds is larger than an uint64_t can represent, it will be clamped to the
    /// uint64_t max value.
    /// @note The remaining microseconds are truncated, similar to the casting behavior of a float to an int.
    constexpr auto as_millis() const noexcept -> uint64_t;

    /// @brief returns the duration in seconds
    /// @note The remaining milliseconds are truncated, similar to the casting behavior of a float to an int.
    constexpr auto as_secs() const noexcept -> uint64_t;

    /// @brief returns the duration in minutes
    /// @note The remaining seconds are truncated, similar to the casting behavior of a float to an int.
    constexpr auto as_mins() const noexcept -> uint64_t;

    /// @brief returns the duration in hours
    /// @note The remaining minutes are truncated, similar to the casting behavior of a float to an int.
    constexpr auto as_hours() const noexcept -> uint64_t;

    /// @brief returns the duration in days
    /// @note The remaining hours are truncated, similar to the casting behavior of a float to an int.
    constexpr auto as_days() const noexcept -> uint64_t;

    /// @brief returns the subsecond part of the duration in nanoseconds
    constexpr auto subsec_nanos() const noexcept -> uint32_t;

    /// @brief returns the subsecond part of the duration in microseconds
    /// @note The remaining nanoseconds are truncated, similar to the casting behavior of a float to an int.
    constexpr auto subsec_micros() const noexcept -> uint32_t;

    /// @brief returns the subsecond part of the duration in milliseconds
    /// @note The remaining microseconds are truncated, similar to the casting behavior of a float to an int.
    constexpr auto subsec_millis() const noexcept -> uint32_t;

    // END CONVERSION

    // AXIVION DISABLE STYLE AutosarC++19_03-A3.9.1 : Use of unsigned long long int in user-defined literals is enforced by the standard
    friend constexpr auto duration_literals::operator""_ns(unsigned long long int value) noexcept -> Duration;
    friend constexpr auto duration_literals::operator""_us(unsigned long long int value) noexcept -> Duration;
    friend constexpr auto duration_literals::operator""_ms(unsigned long long int value) noexcept -> Duration;
    friend constexpr auto duration_literals::operator""_s(unsigned long long int value) noexcept -> Duration;
    friend constexpr auto duration_literals::operator""_m(unsigned long long int value) noexcept -> Duration;
    friend constexpr auto duration_literals::operator""_h(unsigned long long int value) noexcept -> Duration;
    friend constexpr auto duration_literals::operator""_d(unsigned long long int value) noexcept -> Duration;
    // AXIVION ENABLE STYLE AutosarC++19_03-A3.9.1

    // AXIVION Next Construct AutosarC++19_03-A8.4.7 : Argument is larger than two words
    template <typename T>
    friend constexpr auto operator*(const T& lhs, const Duration& rhs) noexcept -> Duration;

    static constexpr uint32_t SECS_PER_MINUTE { 60U };
    static constexpr uint32_t SECS_PER_HOUR { 3600U };
    static constexpr uint32_t HOURS_PER_DAY { 24U };

    static constexpr uint32_t MILLISECS_PER_SEC { 1000U };
    static constexpr uint32_t MICROSECS_PER_SEC { MILLISECS_PER_SEC * 1000U };

    static constexpr uint32_t NANOSECS_PER_MICROSEC { 1000U };
    static constexpr uint32_t NANOSECS_PER_MILLISEC { NANOSECS_PER_MICROSEC * 1000U };
    static constexpr uint32_t NANOSECS_PER_SEC { NANOSECS_PER_MILLISEC * 1000U };

    /// @brief Constructs a Duration from seconds and nanoseconds
    /// @param[in] seconds portion of the duration
    /// @param[in] nanoseconds portion of the duration
    /// @note this is protected to be able to use it in unit tests
    constexpr Duration(SecondsT seconds, NanosecondsT nanoseconds) noexcept;

    /// @note this is factory method is necessary to build with msvc due to issues calling a protected constexpr ctor
    /// from public methods
    static constexpr auto create_duration(SecondsT seconds, NanosecondsT nanoseconds) noexcept -> Duration;

  private:
    template <typename T>
    static constexpr auto positive_value_or_clamp_to_zero(T value) noexcept -> uint64_t;

    template <typename T>
    constexpr auto from_floating_point_seconds(T floating_point_seconds) const noexcept -> Duration;
    template <typename From, typename To>
    constexpr auto would_cast_from_floating_point_probably_overflow(From floating_point) const noexcept -> bool;

    template <typename T>
    constexpr auto multiply_with(const std::enable_if_t<!std::is_floating_point<T>::value, T>& rhs) const noexcept
        -> Duration;

    template <typename T>
    constexpr auto multiply_with(const std::enable_if_t<std::is_floating_point<T>::value, T>& rhs) const noexcept
        -> Duration;
};

/// @brief creates Duration object by multiplying object T with a duration. On overflow
///        duration will saturate to Duration::max()
/// @tparam T is an arithmetic type for the multiplicator
/// @param[in] lhs is the multiplicator
/// @param[in] rhs is the multiplicant
/// @return a new Duration object
/// @attention Since negative durations are not allowed, the duration will be clamped to 0
// AXIVION Next Construct AutosarC++19_03-A8.4.7 : Each argument is larger than two words
template <typename T>
constexpr auto operator*(const T& lhs, const Duration& rhs) noexcept -> Duration {
    return rhs * lhs;
}

/// @brief Dummy implementation with a static assert. Assigning the result of a Duration
/// multiplication with 'operator*=' to an arithmetic type is not supported
// AXIVION Next Construct AutosarC++19_03-A8.4.7 : Each argument is larger than two words
template <typename T>
constexpr auto operator*=(T& lhs IOX2_MAYBE_UNUSED, const Duration& rhs IOX2_MAYBE_UNUSED) noexcept -> T& {
    static_assert(
        legacy::always_false_v<T>,
        "Assigning the result of a Duration multiplication with 'operator*=' to an arithmetic type is not supported");
    return T();
}

/// @brief Equal to operator
/// @param[in] lhs is the left hand side of the comparison
/// @param[in] rhs is the right hand side of the comparison
/// @return true if duration equal to rhs
// AXIVION Next Line AutosarC++19_03-A8.4.7 : Each argument is larger than two words
constexpr auto operator==(const Duration& lhs, const Duration& rhs) noexcept -> bool {
    return (lhs.m_seconds == rhs.m_seconds) && (lhs.m_nanoseconds == rhs.m_nanoseconds);
}

/// @brief Not equal to operator
/// @param[in] lhs is the left hand side of the comparison
/// @param[in] rhs is the right hand side of the comparison
/// @return true if duration not equal to rhs
// AXIVION Next Line AutosarC++19_03-A8.4.7 : Each argument is larger than two words
constexpr auto operator!=(const Duration& lhs, const Duration& rhs) noexcept -> bool {
    return !(lhs == rhs);
}

/// @brief Less than operator
/// @param[in] lhs is the left hand side of the comparison
/// @param[in] rhs is the right hand side of the comparison
/// @return true if duration is less than rhs
// AXIVION Next Line AutosarC++19_03-A8.4.7 : Each argument is larger than two words
constexpr auto operator<(const Duration& lhs, const Duration& rhs) noexcept -> bool {
    return (lhs.m_seconds < rhs.m_seconds)
           || ((lhs.m_seconds == rhs.m_seconds) && (lhs.m_nanoseconds < rhs.m_nanoseconds));
}

/// @brief Greater than operator
/// @param[in] lhs is the left hand side of the comparison
/// @param[in] rhs is the right hand side of the comparison
/// @return true if duration is greater than rhs
// AXIVION Next Line AutosarC++19_03-A8.4.7 : Each argument is larger than two words
constexpr auto operator>(const Duration& lhs, const Duration& rhs) noexcept -> bool {
    return (lhs.m_seconds > rhs.m_seconds)
           || ((lhs.m_seconds == rhs.m_seconds) && (lhs.m_nanoseconds > rhs.m_nanoseconds));
}

/// @brief Less than or equal to operator
/// @param[in] lhs is the left hand side of the comparison
/// @param[in] rhs is the right hand side of the comparison
/// @return true if duration is less than or equal to rhs
// AXIVION Next Line AutosarC++19_03-A8.4.7 : Each argument is larger than two words
constexpr auto operator<=(const Duration& lhs, const Duration& rhs) noexcept -> bool {
    return !(lhs > rhs);
}

/// @brief Greater than or equal to operator
/// @param[in] lhs is the left hand side of the comparison
/// @param[in] rhs is the right hand side of the comparison
/// @return true if duration is greater than or equal to rhs
// AXIVION Next Line AutosarC++19_03-A8.4.7 : Each argument is larger than two words
constexpr auto operator>=(const Duration& lhs, const Duration& rhs) noexcept -> bool {
    return !(lhs < rhs);
}

// NOLINTJUSTIFICATION @todo iox-#1617 Seconds_t and Nanoseconds_t should use Newtype pattern to solve this issue
// NOLINTNEXTLINE(bugprone-easily-swappable-parameters)
constexpr Duration::Duration(const SecondsT seconds, const NanosecondsT nanoseconds) noexcept
    : m_seconds(seconds)
    , m_nanoseconds(nanoseconds) {
    if (nanoseconds >= NANOSECS_PER_SEC) {
        const SecondsT additional_seconds { static_cast<SecondsT>(nanoseconds)
                                            / static_cast<SecondsT>(NANOSECS_PER_SEC) };
        if ((std::numeric_limits<SecondsT>::max() - additional_seconds) < m_seconds) {
            m_seconds = std::numeric_limits<SecondsT>::max();
            m_nanoseconds = NANOSECS_PER_SEC - 1U;
        } else {
            m_seconds += additional_seconds;
            m_nanoseconds = m_nanoseconds % NANOSECS_PER_SEC;
        }
    }
}

constexpr auto Duration::create_duration(const SecondsT seconds, const NanosecondsT nanoseconds) noexcept -> Duration {
    return { seconds, nanoseconds };
}

constexpr auto Duration::max() noexcept -> Duration {
    return Duration { std::numeric_limits<SecondsT>::max(), NANOSECS_PER_SEC - 1U };
}

constexpr auto Duration::zero() noexcept -> Duration {
    return Duration { 0U, 0U };
}

template <typename T>
constexpr auto Duration::positive_value_or_clamp_to_zero(const T value) noexcept -> uint64_t {
    static_assert(std::numeric_limits<T>::is_integer, "only integer types are supported");

    // AXIVION Next Construct AutosarC++19_03-A1.4.3 : Value is a templated an arbitrary integer
    // type that is not necessarily unsigned
    if (value < 0) {
        return 0U;
    }

    return static_cast<uint64_t>(value);
}

template <typename T>
constexpr auto Duration::from_nanos(const T value) noexcept -> Duration {
    const auto clamped_value = positive_value_or_clamp_to_zero(value);
    const auto seconds = static_cast<Duration::SecondsT>(clamped_value / Duration::NANOSECS_PER_SEC);
    const auto nanoseconds = static_cast<Duration::NanosecondsT>(clamped_value % Duration::NANOSECS_PER_SEC);
    return create_duration(seconds, nanoseconds);
}
template <typename T>
constexpr auto Duration::from_micros(const T value) noexcept -> Duration {
    const auto clamped_value = positive_value_or_clamp_to_zero(value);
    const auto seconds = static_cast<Duration::SecondsT>(clamped_value / Duration::MICROSECS_PER_SEC);
    const auto nanoseconds = static_cast<Duration::NanosecondsT>((clamped_value % Duration::MICROSECS_PER_SEC)
                                                                 * Duration::NANOSECS_PER_MICROSEC);
    return create_duration(seconds, nanoseconds);
}
template <typename T>
constexpr auto Duration::from_millis(const T value) noexcept -> Duration {
    const auto clamped_value = positive_value_or_clamp_to_zero(value);
    const auto seconds = static_cast<Duration::SecondsT>(clamped_value / Duration::MILLISECS_PER_SEC);
    const auto nanoseconds = static_cast<Duration::NanosecondsT>((clamped_value % Duration::MILLISECS_PER_SEC)
                                                                 * Duration::NANOSECS_PER_MILLISEC);
    return create_duration(seconds, nanoseconds);
}
template <typename T>
constexpr auto Duration::from_secs(const T value) noexcept -> Duration {
    const auto clamped_value = positive_value_or_clamp_to_zero(value);
    constexpr Duration::SecondsT MAX_SECONDS_BEFORE_OVERFLOW { std::numeric_limits<Duration::SecondsT>::max() };

    // AXIVION Next Construct AutosarC++19_03-M0.1.2, AutosarC++19_03-M0.1.9, FaultDetection-DeadBranches : False positive, platform-dependent
    if (clamped_value > MAX_SECONDS_BEFORE_OVERFLOW) {
        return Duration::max();
    }
    return Duration { static_cast<Duration::SecondsT>(clamped_value), 0U };
}
template <typename T>
constexpr auto Duration::from_mins(const T value) noexcept -> Duration {
    const auto clamped_value = positive_value_or_clamp_to_zero(value);
    constexpr uint64_t MAX_MINUTES_BEFORE_OVERFLOW { std::numeric_limits<uint64_t>::max() / Duration::SECS_PER_MINUTE };
    if (clamped_value > MAX_MINUTES_BEFORE_OVERFLOW) {
        return Duration::max();
    }
    return Duration { static_cast<Duration::SecondsT>(clamped_value * Duration::SECS_PER_MINUTE), 0U };
}
template <typename T>
constexpr auto Duration::from_hours(const T value) noexcept -> Duration {
    const auto clamped_value = positive_value_or_clamp_to_zero(value);
    constexpr uint64_t MAX_HOURS_BEFORE_OVERFLOW { std::numeric_limits<uint64_t>::max() / Duration::SECS_PER_HOUR };
    if (clamped_value > MAX_HOURS_BEFORE_OVERFLOW) {
        return Duration::max();
    }
    return Duration { static_cast<Duration::SecondsT>(clamped_value * Duration::SECS_PER_HOUR), 0U };
}
template <typename T>
constexpr auto Duration::from_days(const T value) noexcept -> Duration {
    const auto clamped_value = positive_value_or_clamp_to_zero(value);
    constexpr uint64_t SECS_PER_DAY { static_cast<uint64_t>(Duration::HOURS_PER_DAY * Duration::SECS_PER_HOUR) };
    constexpr uint64_t MAX_DAYS_BEFORE_OVERFLOW { std::numeric_limits<uint64_t>::max() / SECS_PER_DAY };
    if (clamped_value > MAX_DAYS_BEFORE_OVERFLOW) {
        return Duration::max();
    }
    return Duration { static_cast<Duration::SecondsT>(clamped_value * SECS_PER_DAY), 0U };
}

constexpr auto Duration::as_nanos() const noexcept -> uint64_t {
    constexpr SecondsT MAX_SECONDS_BEFORE_OVERFLOW { std::numeric_limits<uint64_t>::max()
                                                     / static_cast<uint64_t>(NANOSECS_PER_SEC) };
    constexpr NanosecondsT MAX_NANOSECONDS_BEFORE_OVERFLOW { static_cast<NanosecondsT>(
        std::numeric_limits<uint64_t>::max() % static_cast<uint64_t>(NANOSECS_PER_SEC)) };
    constexpr Duration MAX_DURATION_BEFORE_OVERFLOW { create_duration(MAX_SECONDS_BEFORE_OVERFLOW,
                                                                      MAX_NANOSECONDS_BEFORE_OVERFLOW) };

    if (*this > MAX_DURATION_BEFORE_OVERFLOW) {
        return std::numeric_limits<uint64_t>::max();
    }

    return (m_seconds * NANOSECS_PER_SEC) + m_nanoseconds;
}

constexpr auto Duration::as_micros() const noexcept -> uint64_t {
    constexpr SecondsT MAX_SECONDS_BEFORE_OVERFLOW { std::numeric_limits<uint64_t>::max() / MICROSECS_PER_SEC };
    constexpr NanosecondsT MAX_NANOSECONDS_BEFORE_OVERFLOW {
        static_cast<NanosecondsT>(std::numeric_limits<uint64_t>::max() % MICROSECS_PER_SEC) * NANOSECS_PER_MICROSEC
    };
    constexpr Duration MAX_DURATION_BEFORE_OVERFLOW { create_duration(MAX_SECONDS_BEFORE_OVERFLOW,
                                                                      MAX_NANOSECONDS_BEFORE_OVERFLOW) };

    if (*this > MAX_DURATION_BEFORE_OVERFLOW) {
        return std::numeric_limits<uint64_t>::max();
    }

    return (m_seconds * MICROSECS_PER_SEC)
           + (static_cast<SecondsT>(m_nanoseconds) / static_cast<SecondsT>(NANOSECS_PER_MICROSEC));
}

constexpr auto Duration::as_millis() const noexcept -> uint64_t {
    constexpr SecondsT MAX_SECONDS_BEFORE_OVERFLOW { std::numeric_limits<uint64_t>::max() / MILLISECS_PER_SEC };
    constexpr NanosecondsT MAX_NANOSECONDS_BEFORE_OVERFLOW {
        static_cast<NanosecondsT>(std::numeric_limits<uint64_t>::max() % MILLISECS_PER_SEC) * NANOSECS_PER_MILLISEC
    };
    constexpr Duration MAX_DURATION_BEFORE_OVERFLOW { create_duration(MAX_SECONDS_BEFORE_OVERFLOW,
                                                                      MAX_NANOSECONDS_BEFORE_OVERFLOW) };

    if (*this > MAX_DURATION_BEFORE_OVERFLOW) {
        return std::numeric_limits<uint64_t>::max();
    }

    return (m_seconds * MILLISECS_PER_SEC)
           + (static_cast<SecondsT>(m_nanoseconds) / static_cast<SecondsT>(NANOSECS_PER_MILLISEC));
}

constexpr auto Duration::as_secs() const noexcept -> uint64_t {
    return m_seconds;
}

constexpr auto Duration::as_mins() const noexcept -> uint64_t {
    return m_seconds / SECS_PER_MINUTE;
}

constexpr auto Duration::as_hours() const noexcept -> uint64_t {
    return m_seconds / SECS_PER_HOUR;
}

constexpr auto Duration::as_days() const noexcept -> uint64_t {
    return m_seconds / static_cast<uint64_t>(HOURS_PER_DAY * SECS_PER_HOUR);
}

constexpr auto Duration::subsec_nanos() const noexcept -> uint32_t {
    return m_nanoseconds;
}

constexpr auto Duration::subsec_micros() const noexcept -> uint32_t {
    return m_nanoseconds / NANOSECS_PER_MICROSEC;
}

constexpr auto Duration::subsec_millis() const noexcept -> uint32_t {
    return m_nanoseconds / NANOSECS_PER_MILLISEC;
}

// AXIVION Next Construct AutosarC++19_03-A8.4.7 : Argument is larger than two words
constexpr auto Duration::operator+(const Duration& rhs) const noexcept -> Duration {
    SecondsT seconds { m_seconds + rhs.m_seconds };
    NanosecondsT nanoseconds { m_nanoseconds + rhs.m_nanoseconds };
    if (nanoseconds >= NANOSECS_PER_SEC) {
        ++seconds;
        nanoseconds -= NANOSECS_PER_SEC;
    }

    const auto sum = create_duration(seconds, nanoseconds);
    if (sum < *this) {
        return Duration::max();
    }
    return sum;
}

// AXIVION Next Construct AutosarC++19_03-A8.4.7 : Argument is larger than two words
constexpr auto Duration::operator+=(const Duration& rhs) noexcept -> Duration& {
    *this = *this + rhs;
    return *this;
}

// AXIVION Next Construct AutosarC++19_03-A8.4.7 : Argument is larger than two words
constexpr auto Duration::operator-(const Duration& rhs) const noexcept -> Duration {
    if (*this < rhs) {
        return Duration::zero();
    }
    SecondsT seconds { m_seconds - rhs.m_seconds };
    // AXIVION Next Construct AutosarC++19_03-M0.1.9, AutosarC++19_03-A0.1.1, FaultDetection-UnusedAssignments : False positive, variable IS used
    NanosecondsT nanoseconds { 0U };
    if (m_nanoseconds >= rhs.m_nanoseconds) {
        nanoseconds = m_nanoseconds - rhs.m_nanoseconds;
    } else {
        // AXIVION Next Construct AutosarC++19_03-A4.7.1, AutosarC++19_03-M0.3.1, FaultDetection-IntegerOverflow : It is ensured that m_nanoseconds is never larger than NANOSECS_PER_SEC
        nanoseconds = (NANOSECS_PER_SEC - rhs.m_nanoseconds) + m_nanoseconds;
        --seconds;
    }
    return create_duration(seconds, nanoseconds);
}

// AXIVION Next Construct AutosarC++19_03-A8.4.7 : Argument is larger than two words
constexpr auto Duration::operator-=(const Duration& rhs) noexcept -> Duration& {
    *this = *this - rhs;
    return *this;
}

template <typename T>
constexpr auto Duration::multiply_with(const std::enable_if_t<!std::is_floating_point<T>::value, T>& rhs) const noexcept
    -> Duration {
    if ((rhs <= static_cast<T>(0)) || (*this == Duration::zero())) {
        return Duration::zero();
    }

    static_assert(sizeof(T) <= sizeof(SecondsT),
                  "only integer types with less or equal to size of uint64_t are allowed for multiplication");
    const auto multiplicator = static_cast<SecondsT>(rhs);

    const SecondsT max_before_overflow { std::numeric_limits<SecondsT>::max() / multiplicator };

    // check if the result of the m_seconds multiplication would already overflow
    if (m_seconds > max_before_overflow) {
        return Duration::max();
    }
    const auto duration_from_seconds = Duration(m_seconds * multiplicator, 0U);

    // the m_nanoseconds multiplication cannot exceed the limits of a Duration, since m_nanoseconds is always less than
    // a second and m_seconds can hold 64 bits and the multiplicator is at max 64 bits

    // check if the result of the m_nanoseconds multiplication can easily be converted into a Duration
    // AXIVION Next Construct AutosarC++19_03-M0.1.2, AutosarC++19_03-M0.1.9, FaultDetection-DeadBranches : False positive! Branching depends on input parameter
    if (m_nanoseconds <= max_before_overflow) {
        return duration_from_seconds + Duration::from_nanos(m_nanoseconds * multiplicator);
    }

    // when we reach this, the multiplicator must be larger than 2^32, since smaller values multiplied with the
    // m_nanoseconds(uint32_t) would fit into 64 bits;
    // to accurately determine the result, the calculation is split into a multiplication with the lower 32 bits of the
    // multiplicator and another one with the upper 32 bits;

    // this is the easy part with the lower 32 bits
    const uint64_t multiplicator_low { static_cast<uint32_t>(multiplicator) };
    const Duration duration_from_nanos_low { Duration::from_nanos(m_nanoseconds * multiplicator_low) };

    // this is the complicated part with the upper 32 bits;
    // the m_nanoseconds are multiplied with the upper 32 bits of the multiplicator shifted by 32 bit to the right, thus
    // having again a multiplication of two 32 bit values whose result fits into a 64 bit variable;
    // one bit of the result represents 2^32 nanoseconds;
    // just shifting left by 32 bits would result in an overflow, therefore blocks of full seconds must be extracted of
    // the result;
    // this cannot be done by dividing through NANOSECS_PER_SEC, since that one is base 1_000_000_000 and the result is
    // base 2^32, therefore the least common multiple can be used to get blocks of full seconds represented with the LSB
    // representing 2^32 nanoseconds;
    // this can then safely be converted to seconds as well as nanoseconds without loosing precision

    // least common multiple of 2^32 and NANOSECONDS_PER_SECOND;
    // for the following calculation it is not important to be the least common multiple, any common multiple will do
    constexpr uint64_t LEAST_COMMON_MULTIPLE { 8388608000000000 };
    constexpr uint64_t NUMBER_OF_BITS_IN_UINT32 { 32 };
    static_assert((LEAST_COMMON_MULTIPLE % (1ULL << NUMBER_OF_BITS_IN_UINT32)) == 0, "invalid multiple");
    static_assert((LEAST_COMMON_MULTIPLE % NANOSECS_PER_SEC) == 0, "invalid multiple");

    constexpr uint64_t ONE_FULL_BLOCK_OF_SECONDS_ONLY { LEAST_COMMON_MULTIPLE >> NUMBER_OF_BITS_IN_UINT32 };
    constexpr uint64_t SECONDS_PER_FULL_BLOCK { LEAST_COMMON_MULTIPLE / NANOSECS_PER_SEC };

    const uint64_t multiplicator_high { static_cast<uint32_t>(multiplicator >> NUMBER_OF_BITS_IN_UINT32) };
    const uint64_t nanoseconds_from_high { m_nanoseconds * multiplicator_high };
    const uint64_t full_blocks_of_seconds_only { nanoseconds_from_high / ONE_FULL_BLOCK_OF_SECONDS_ONLY };
    const uint64_t remaining_block_with_full_and_fractional_seconds { nanoseconds_from_high
                                                                      % ONE_FULL_BLOCK_OF_SECONDS_ONLY };

    // AXIVION Next Construct AutosarC++19_03-A4.7.1, AutosarC++19_03-M0.3.1, FaultDetection-IntegerOverflow : The logic from above prevents overflows
    const auto duration_from_nanos_high =
        Duration { full_blocks_of_seconds_only * SECONDS_PER_FULL_BLOCK, 0U }
        + Duration::from_nanos(remaining_block_with_full_and_fractional_seconds << NUMBER_OF_BITS_IN_UINT32);

    return duration_from_seconds + duration_from_nanos_low + duration_from_nanos_high;
}


template <typename From, typename To>
constexpr auto Duration::would_cast_from_floating_point_probably_overflow(const From floating_point) const noexcept
    -> bool {
    static_assert(std::is_floating_point<From>::value, "only floating point is allowed");

    // depending on the internal representation this could be either the last value to not cause an overflow
    // or the first one which causes an overflow;
    // to be safe, this is handled like causing an overflow which would result in undefined behavior when casting to
    // Seconds_t
    constexpr From SECONDS_BEFORE_LIKELY_OVERFLOW { static_cast<From>(std::numeric_limits<To>::max()) };
    return floating_point >= SECONDS_BEFORE_LIKELY_OVERFLOW;
}

template <typename T>
constexpr auto Duration::from_floating_point_seconds(const T floating_point_seconds) const noexcept -> Duration {
    static_assert(std::is_floating_point<T>::value, "only floating point is allowed");

    if (std::isinf(floating_point_seconds)) {
        return Duration::max();
    }

    T seconds_full { 0 };
    T seconds_fraction { std::modf(floating_point_seconds, &seconds_full) };

    if (would_cast_from_floating_point_probably_overflow<T, SecondsT>(seconds_full)) {
        return Duration::max();
    }

    return Duration { static_cast<SecondsT>(seconds_full),
                      static_cast<NanosecondsT>(seconds_fraction * NANOSECS_PER_SEC) };
}

template <typename T>
constexpr auto Duration::multiply_with(const std::enable_if_t<std::is_floating_point<T>::value, T>& rhs) const noexcept
    -> Duration {
    if (std::isnan(rhs)) {
        return (*this == Duration::zero()) ? Duration::zero() : Duration::max();
    }

    // this must be done after the NAN check in order to prevent to access a signaling NAN
    if ((rhs <= static_cast<T>(0)) || (*this == Duration::zero())) {
        return Duration::zero();
    }

    auto duration_from_seconds = from_floating_point_seconds<T>(static_cast<T>(m_seconds) * rhs);

    auto result_nanoseconds = static_cast<T>(m_nanoseconds) * rhs;

    if (!would_cast_from_floating_point_probably_overflow<T, uint64_t>(result_nanoseconds)) {
        return duration_from_seconds + Duration::from_nanos(static_cast<uint64_t>(result_nanoseconds));
    }

    // the multiplication result of nanoseconds would exceed the value an uint64_t can represent
    // -> convert result to seconds and and calculate duration
    auto floating_point_seconds = result_nanoseconds / NANOSECS_PER_SEC;
    auto duration_from_nanos = from_floating_point_seconds<T>(floating_point_seconds);

    return duration_from_seconds + duration_from_nanos;
}

// AXIVION Next Construct AutosarC++19_03-M5.17.1 : False positive! Corresponding assignment operator is implemented below
template <typename T>
constexpr auto Duration::operator*(const T& rhs) const noexcept -> Duration {
    static_assert(std::is_arithmetic<T>::value, "non arithmetic types are not supported for multiplication");

    return multiply_with<T>(rhs);
}

template <typename T>
constexpr auto Duration::operator*=(const T& rhs) noexcept -> Duration& {
    static_assert(std::is_arithmetic<T>::value, "non arithmetic types are not supported for multiplication");

    *this = multiply_with<T>(rhs);

    return *this;
}

namespace duration_literals {
// AXIVION Next Construct AutosarC++19_03-A3.9.1 : Use of unsigned long long int in user-defined literals is enforced by the standard
constexpr auto operator""_ns(unsigned long long int value) noexcept -> Duration {
    return Duration::from_nanos(value);
}

// AXIVION Next Construct AutosarC++19_03-A3.9.1 : Use of unsigned long long int in user-defined literals is enforced by the standard
constexpr auto operator""_us(unsigned long long int value) noexcept -> Duration {
    return Duration::from_micros(value);
}

// AXIVION Next Construct AutosarC++19_03-A3.9.1 : Use of unsigned long long int in user-defined literals is enforced by the standard
constexpr auto operator""_ms(unsigned long long int value) noexcept -> Duration {
    return Duration::from_millis(value);
}

// AXIVION Next Construct AutosarC++19_03-A3.9.1 : Use of unsigned long long int in user-defined literals is enforced by the standard
constexpr auto operator""_s(unsigned long long int value) noexcept -> Duration {
    return Duration::from_secs(value);
}

// AXIVION Next Construct AutosarC++19_03-A3.9.1 : Use of unsigned long long int in user-defined literals is enforced by the standard
constexpr auto operator""_m(unsigned long long int value) noexcept -> Duration {
    return Duration::from_mins(value);
}

// AXIVION Next Construct AutosarC++19_03-A3.9.1 : Use of unsigned long long int in user-defined literals is enforced by the standard
constexpr auto operator""_h(unsigned long long int value) noexcept -> Duration {
    return Duration::from_hours(value);
}

// AXIVION Next Construct AutosarC++19_03-A3.9.1 : Use of unsigned long long int in user-defined literals is enforced by the standard
constexpr auto operator""_d(unsigned long long int value) noexcept -> Duration {
    return Duration::from_days(value);
}
} // namespace duration_literals

} // namespace bb
} // namespace iox2

// AXIVION Next Construct AutosarC++19_03-M5.17.1 : This is not used as shift operator but as stream operator and does not require to implement '<<='
inline auto operator<<(iox2::legacy::log::LogStream& stream, const iox2::bb::Duration duration) noexcept
    -> iox2::legacy::log::LogStream& {
    stream << duration.as_secs() << "s " << duration.subsec_nanos() << "ns";
    return stream;
}

// AXIVION Next Construct AutosarC++19_03-M5.17.1 : This is not used as shift operator but as stream operator and does not require to implement '<<='
inline auto operator<<(std::ostream& stream, const iox2::bb::Duration duration) -> std::ostream& {
    stream << duration.as_secs() << "s " << duration.subsec_nanos() << "ns";
    return stream;
}

#endif // IOX2_BB_DURATION_HPP
